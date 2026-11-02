//! Conservative command grammar for a read-only Clang VFS comparison.

use std::path::Path;

use kinein_protocol::CompileUnit;

use crate::build::syntax_check::{SyntaxCheck, syntax_only_command};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Backend {
    Clang,
    Gcc,
}

pub(super) struct Check {
    pub command: SyntaxCheck,
    pub backend: Backend,
    pub source: std::path::PathBuf,
}

const VALUE_FLAGS: &[&str] = &[
    "-I",
    "-isystem",
    "-iquote",
    "-idirafter",
    "-include",
    "-imacros",
    "-D",
    "-U",
    "--sysroot",
    "-isysroot",
    "-target",
    "-x",
];

const JOINED_FLAGS: &[&str] = &[
    "-I",
    "-D",
    "-U",
    "-std=",
    "--sysroot=",
    "--target=",
    "-march=",
    "-mcpu=",
    "-mtune=",
    "-stdlib=",
    "-fstrict-flex-arrays=",
    "-fcf-protection=",
    "-isystem",
    "-iquote",
    "-idirafter",
];

const STANDALONE_FLAGS: &[&str] = &[
    "-fsyntax-only",
    "-fdiagnostics-color=never",
    "-fPIC",
    "-fpic",
    "-fPIE",
    "-fpie",
    "-fno-exceptions",
    "-fexceptions",
    "-fno-rtti",
    "-frtti",
    "-fno-builtin",
    "-ffreestanding",
    "-fno-common",
    "-fstack-protector-strong",
    "-fno-omit-frame-pointer",
    "-fno-direct-access-external-data",
    "-fcommon",
    "-funsigned-char",
    "-fsigned-char",
    "-pthread",
    "-nostdinc",
    "-nostdinc++",
    "-m32",
    "-m64",
    "-g",
    "-g0",
    "-g1",
    "-g2",
    "-g3",
    "-w",
];

pub(super) fn command(unit: &CompileUnit, path: &Path) -> Result<Check, String> {
    let program = Path::new(&unit.compiler)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    let stem = program.strip_suffix(".exe").unwrap_or(program);
    let compatible = ["clang", "clang++"].iter().any(|name| {
        stem == *name
            || stem
                .strip_prefix(&format!("{name}-"))
                .is_some_and(|v| !v.is_empty() && v.chars().all(|c| c.is_ascii_digit()))
    });
    let gcc = ["gcc", "g++", "cc", "c++"].iter().any(|n| {
        stem == *n
            || stem
                .strip_prefix(&format!("{n}-"))
                .is_some_and(|v| v.chars().all(|c| c.is_ascii_digit()))
    });
    if (!compatible && !gcc) || unit.arguments.first() != Some(&unit.compiler) {
        return Err(
            "Compilador incompatível: requer driver GCC/G++ ou Clang/Clang++ da CDB.".into(),
        );
    }
    let mut check = syntax_only_command(unit).ok_or("Comando da CDB não interpretável.")?;
    let mut sources = 0;
    let mut source = path.to_path_buf();
    let mut iter = check.args.iter();
    while let Some(arg) = iter.next() {
        if VALUE_FLAGS.contains(&arg.as_str()) {
            let value = iter
                .next()
                .filter(|v| !v.is_empty() && !v.starts_with('@'))
                .ok_or("Flag sem valor interpretável.")?;
            if arg == "-x" && !["c", "c++"].contains(&value.as_str()) {
                return Err("Linguagem fora do recorte C/C++.".into());
            }
            continue;
        }
        let joined = JOINED_FLAGS
            .iter()
            .any(|prefix| arg.starts_with(prefix) && arg.len() > prefix.len());
        let standalone = STANDALONE_FLAGS.contains(&arg.as_str());
        let warning = arg.starts_with("-W")
            && !arg.starts_with("-Wl,")
            && !arg.starts_with("-Wa,")
            && !arg.starts_with("-Wp,");
        let optimization =
            ["-O0", "-O1", "-O2", "-O3", "-Os", "-Oz", "-Og", "-Ofast"].contains(&arg.as_str());
        if joined || standalone || warning || optimization {
            continue;
        }
        if !arg.starts_with('-')
            && crate::platform::canonicalize(&check.directory.join(arg))
                .ok()
                .as_deref()
                == Some(path)
        {
            source = check.directory.join(arg);
            sources += 1;
            continue;
        }
        return Err(format!("Opção ou entrada ainda não suportada: {arg}"));
    }
    if sources != 1 {
        return Err("A CDB não identifica uma única unidade de entrada.".into());
    }
    // Keep the virtual filename, cwd and include search order unchanged.
    check
        .args
        .extend(["-H", "-v", "-fmessage-length=0"].map(str::to_owned));
    let backend = if compatible {
        Backend::Clang
    } else {
        Backend::Gcc
    };
    let flags = if compatible {
        &[
            "-ferror-limit=0",
            "-fshow-skipped-includes",
            "-fno-caret-diagnostics",
            "-fno-diagnostics-fixit-info",
        ][..]
    } else {
        &[
            "-fmax-errors=0",
            "-fdiagnostics-column-unit=byte",
            "-fno-diagnostics-show-caret",
            "-ftrack-macro-expansion=0",
        ][..]
    };
    check.args.extend(flags.iter().map(|s| (*s).to_owned()));
    Ok(Check {
        command: check,
        backend,
        source,
    })
}
