//! Delimitacao conservadora de comandos literais para a receita de padrao C.
//! Nao avalia `CMake`. Scripts externos, controle de fluxo e sintaxe ambigua
//! exigem edicao manual; o contexto efetivo continua vindo do file-api/CDB.

use std::{iter::Peekable, str::Chars};

use super::error::ConfigActionError;

pub(super) struct Command {
    pub(super) name: String,
    pub(super) args: Vec<String>,
}

pub(super) fn manual(reason: impl AsRef<str>) -> ConfigActionError {
    ConfigActionError::NotApplicable {
        reason: format!("{}; edite o CMakeLists.txt manualmente", reason.as_ref()),
    }
}

/// Le apenas comandos com argumentos simples ou entre aspas, sem escapes.
pub(super) fn commands(text: &str) -> Result<Vec<Command>, ConfigActionError> {
    let mut input = text.chars().peekable();
    let mut result = Vec::new();
    loop {
        skip_layout(&mut input)?;
        if input.peek().is_none() {
            return Ok(result);
        }
        let mut name = String::new();
        while let Some(c) = input.next_if(|c| c.is_ascii_alphanumeric() || *c == '_') {
            name.push(c);
        }
        skip_layout(&mut input)?;
        if name.is_empty() || input.next() != Some('(') {
            return Err(manual("sintaxe CMake não delimitável"));
        }
        let args = arguments(&mut input)?;
        result.push(Command {
            name: name.to_ascii_lowercase(),
            args,
        });
    }
}

fn skip_layout(input: &mut Peekable<Chars<'_>>) -> Result<(), ConfigActionError> {
    loop {
        while input.peek().is_some_and(|c| c.is_whitespace()) {
            input.next();
        }
        if input.peek() != Some(&'#') {
            return Ok(());
        }
        input.next();
        if input.peek() == Some(&'[') {
            return Err(manual("comentário CMake em bloco fora do recorte"));
        }
        for c in input.by_ref() {
            if c == '\n' {
                break;
            }
        }
    }
}

fn arguments(input: &mut Peekable<Chars<'_>>) -> Result<Vec<String>, ConfigActionError> {
    let mut args = Vec::new();
    loop {
        skip_layout(input)?;
        if input.peek() == Some(&')') {
            input.next();
            return Ok(args);
        }
        let quoted = input.peek() == Some(&'"');
        if quoted {
            input.next();
        }
        let mut token = String::new();
        loop {
            let Some(&c) = input.peek() else {
                return Err(manual("comando CMake incompleto"));
            };
            if quoted && c == '"' {
                input.next();
                break;
            }
            if !quoted && (c.is_whitespace() || matches!(c, ')' | '#' | ';')) {
                break;
            }
            if c == '\\' || (!quoted && matches!(c, '(' | '[' | '"')) {
                return Err(manual("argumento CMake fora do recorte literal"));
            }
            input.next();
            token.push(c);
        }
        args.push(token);
        if input.peek() == Some(&';') {
            input.next();
        }
    }
}

/// Exige alvo local compilavel e recusa fontes de configuracao nao observadas.
pub(super) fn validate(commands: &[Command], target: &str) -> Result<(), ConfigActionError> {
    const ALLOWED: &[&str] = &[
        "cmake_minimum_required",
        "project",
        "set",
        "option",
        "math",
        "configure_file",
        "add_executable",
        "add_library",
        "target_include_directories",
        "target_compile_definitions",
        "target_compile_options",
        "target_compile_features",
        "target_link_libraries",
        "target_sources",
        "target_link_options",
        "set_target_properties",
        "set_property",
        "set_source_files_properties",
        "add_compile_options",
        "enable_testing",
        "add_test",
    ];
    let mut declarations = 0;
    for command in commands {
        if !ALLOWED.contains(&command.name.as_str()) {
            return Err(manual(format!(
                "{} pode alterar o contexto fora do recorte literal",
                command.name
            )));
        }
        if matches!(command.name.as_str(), "add_executable" | "add_library") {
            if command.args.first().is_some_and(|name| name.contains('$')) {
                return Err(manual("declaração de alvo dinâmica"));
            }
            if command.args.first().is_some_and(|name| name == target) {
                declarations += 1;
                if command
                    .args
                    .iter()
                    .any(|arg| matches!(arg.as_str(), "ALIAS" | "IMPORTED" | "INTERFACE"))
                    || !command.args.iter().skip(1).any(|arg| {
                        !arg.contains('$')
                            && std::path::Path::new(arg).extension()
                                == Some(std::ffi::OsStr::new("c"))
                    })
                {
                    return Err(manual(
                        "o alvo precisa compilar uma fonte .c literal na raiz",
                    ));
                }
            }
        }
        if conflicts(command, target) {
            return Err(manual(format!(
                "o alvo {target} já tem configuração manual de padrão/extensões ou flags de dialeto"
            )));
        }
    }
    if declarations != 1 {
        return Err(manual(format!(
            "o alvo {target} precisa de uma única declaração literal na raiz"
        )));
    }
    Ok(())
}

fn conflicts(command: &Command, target: &str) -> bool {
    let args = &command.args;
    let touches = |names: &[String]| {
        names
            .iter()
            .any(|name| name.contains('$') || name.split(';').any(|name| name == target))
    };
    let property = |arg: &String| {
        matches!(
            arg.as_str(),
            "C_STANDARD" | "C_STANDARD_REQUIRED" | "C_EXTENSIONS"
        )
    };
    let dialect = |arg: &String| {
        arg.contains("-std=")
            || arg.contains("/std:")
            || arg.contains("c_std_")
            || arg.contains("-ansi")
    };
    let dynamic = |arg: &String| arg.contains('$');
    match command.name.as_str() {
        "set_target_properties" => args
            .iter()
            .position(|a| a == "PROPERTIES")
            .is_some_and(|at| {
                touches(&args[..at])
                    && args[at + 1..]
                        .iter()
                        .any(|a| property(a) || dialect(a) || dynamic(a))
            }),
        "set_property" if args.first().is_some_and(|a| a == "TARGET") => {
            args.iter().position(|a| a == "PROPERTY").is_some_and(|at| {
                touches(&args[1..at])
                    && args[at + 1..]
                        .iter()
                        .any(|a| property(a) || dialect(a) || dynamic(a))
            })
        }
        "target_compile_options" | "target_compile_features" => {
            touches(&args[..args.len().min(1)])
                && args.iter().skip(1).any(|a| dialect(a) || dynamic(a))
        }
        "set" => {
            args.first()
                .is_some_and(|a| a.starts_with("CMAKE_C_FLAGS") || a.contains('$'))
                && args.iter().skip(1).any(|a| dialect(a) || dynamic(a))
        }
        "add_compile_options" | "set_source_files_properties" | "set_property" => args
            .iter()
            .any(|a| property(a) || dialect(a) || a == "LANGUAGE"),
        _ => false,
    }
}
