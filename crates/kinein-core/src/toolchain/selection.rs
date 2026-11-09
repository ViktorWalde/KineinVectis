//! A instalacao do compilador C fixada no kit por CAMINHO (receita
//! `cmake.selectCCompiler`, consolidacao C §15).
//!
//! O catalogo nomeia o primeiro `gcc`/`clang` do `PATH`; instalacoes
//! paralelas e caminhos alternativos, que a criacao ja' aceita, so' cabem no
//! kit como caminho absoluto. Este modulo diz quando um valor gravado e' uma
//! instalacao, se ela ainda existe e qual texto o kit teria depois da escolha
//! — com o que a tela precisa para decidir antes de gravar. Quem confere o
//! executavel (versao e sonda C) e' o dono da criacao, fora do laco.

use std::path::Path;

use kinein_protocol::{ProjectEnvironment, ToolInfo, ToolchainRole};

use super::{Toolchain, project::ProjectCache, store};

/// Quantas instalacoes detectadas a previa lista pelo nome.
const DETECTED_SHOWN: usize = 6;

/// O valor gravado no papel e' uma instalacao por caminho absoluto?
///
/// So' o compilador C: e' o papel que tem receita para gravar caminho.
pub(super) fn is_installation(role: ToolchainRole, value: &str) -> bool {
    role == ToolchainRole::CCompiler && Path::new(value).is_absolute()
}

/// `(resolvedPath, effectiveId)` de uma instalacao: ambos ausentes quando o
/// arquivo sumiu, para nenhum consumidor executar outro no lugar.
pub(super) fn installed(path: &str) -> (Option<String>, Option<String>) {
    let file = Path::new(path);
    if !file.is_file() {
        return (None, None);
    }
    let id = file
        .file_name()
        .map(|name| name.to_string_lossy().into_owned());
    (Some(path.to_owned()), id)
}

/// O plano do kit para uma escolha de compilador C, sem gravar nada.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerChoice {
    /// Kit (nome do preset) que recebe a escolha; vazio = o padrao.
    pub kit: String,
    /// `.kinein/toolchain.json`, relativo a raiz.
    pub path: String,
    /// Texto atual do arquivo; `None` quando ele ainda nao existe.
    pub before: Option<String>,
    /// Texto que o store gravaria.
    pub after: String,
    /// Uma linha dizendo o que muda.
    pub summary: String,
    /// Detectado, selecionado, efetivo, origem e divergencias.
    pub notes: Vec<String>,
}

/// O que a previa sabe da maquina: conferido fora do laco, sem processo aqui.
#[derive(Debug, Clone, Copy)]
pub struct Measured<'a> {
    /// Versao do executavel escolhido, quando ha um.
    pub version: Option<&'a str>,
    /// Instalacoes detectadas, com versao.
    pub detected: &'a [ProjectEnvironment],
    /// Preset que o proximo configure deste kit usa (o do kit, ou o padrao
    /// do projeto para o kit padrao): de onde vem a origem sem fixacao.
    pub preset: Option<&'a str>,
}

/// Planeja fixar `compiler` (ou voltar ao automatico) no kit `kit`.
pub fn plan_c_compiler(
    root: &Path,
    tools: &[ToolInfo],
    kit: &str,
    compiler: Option<&str>,
    measured: Measured<'_>,
) -> Result<CompilerChoice, String> {
    let Measured {
        version,
        detected,
        preset,
    } = measured;
    let label = if kit.is_empty() { "padrão" } else { kit };
    let current = Toolchain::resolve_kit(root, tools, kit);
    let previous = current.chosen(ToolchainRole::CCompiler);
    if previous == compiler {
        return Err(compiler.map_or_else(
            || format!("o compilador C do kit {label} já está no automático"),
            |path| format!("o kit {label} já usa {path}"),
        ));
    }
    let mut kits = store::load(root);
    let entry = kits.entry(kit.to_owned()).or_default();
    let role = ToolchainRole::CCompiler.as_str().to_owned();
    match compiler {
        Some(path) => drop(entry.selections.insert(role, path.to_owned())),
        None => drop(entry.selections.remove(&role)),
    }
    let after = store::render(&kits)?;
    let before = std::fs::read_to_string(store::path_for(root)).ok();

    let cache_compiler = ProjectCache::read(root).c_compiler().map(str::to_owned);
    let mut notes = vec![
        if kit.is_empty() {
            "Kit padrão: vale para configure sem preset e para preset dos arquivos do \
             projeto sem kit próprio."
                .to_owned()
        } else {
            format!(
                "Kit {kit}: vale para o configure com o preset {kit} e para o configure \
                 automático depois dele."
            )
        },
        detected_note(detected),
        format!(
            "Selecionado: {} → {}",
            previous.map_or_else(
                || "automático".to_owned(),
                |id| current.resolved_path(ToolchainRole::CCompiler).map_or_else(
                    || format!("{id} (indisponível)"),
                    |path| if path == id {
                        id.to_owned()
                    } else {
                        format!("{id} em {path}")
                    },
                ),
            ),
            compiler.map_or_else(
                || "automático".to_owned(),
                |path| format!(
                    "{path}{}",
                    version.map(|v| format!(" ({v})")).unwrap_or_default()
                ),
            ),
        ),
        cache_compiler.as_ref().map_or_else(
            || "Efetivo: o build dir da IDE ainda não foi configurado.".to_owned(),
            |compiler| format!("Efetivo no último configure: {compiler} (CMakeCache.txt)."),
        ),
    ];
    notes.extend(origin_notes(
        root,
        preset.unwrap_or_default(),
        current.kit_toolchain_file.as_deref(),
        compiler,
        cache_compiler.as_deref(),
    ));
    notes.push(
        "Nada é instalado; padrão C, presets, toolchain files, CMakeLists.txt, flags e \
         .clangd não mudam."
            .to_owned(),
    );
    Ok(CompilerChoice {
        kit: kit.to_owned(),
        path: ".kinein/toolchain.json".to_owned(),
        before,
        after,
        summary: compiler.map_or_else(
            || format!("Volta o compilador C do kit {label} ao automático"),
            |path| {
                format!(
                    "Fixa {path}{} como compilador C do kit {label}",
                    version.map(|v| format!(" ({v})")).unwrap_or_default()
                )
            },
        ),
        notes,
    })
}

/// De onde o compilador viria sem fixacao e o que pode divergir da escolha.
fn origin_notes(
    root: &Path,
    preset: &str,
    kit_toolchain_file: Option<&str>,
    compiler: Option<&str>,
    cache_compiler: Option<&str>,
) -> Vec<String> {
    let preset_compiler =
        crate::cmake_presets::preset_value(root, preset, &["cacheVariables", "CMAKE_C_COMPILER"]);
    let toolchain_file = crate::cmake_presets::preset_value(root, preset, &["toolchainFile"])
        .or_else(|| kit_toolchain_file.map(str::to_owned));
    let mut notes = vec![match (&preset_compiler, &toolchain_file) {
        (Some(value), _) => {
            format!("Sem fixação, o preset {preset} define CMAKE_C_COMPILER={value}.")
        }
        (None, Some(file)) => format!("Sem fixação, o toolchain file {file} decide o compilador."),
        (None, None) => "Sem fixação, o CMake escolhe pelo CC ou pelo cc do PATH no primeiro \
                         configure e conserva a escolha no cache."
            .to_owned(),
    }];
    let Some(path) = compiler else {
        if let Some(cache_compiler) = cache_compiler {
            notes.push(format!(
                "Sem fixação, o CMake conserva {cache_compiler} neste build dir até um configure limpo."
            ));
        }
        return notes;
    };
    if let Some(file) = &toolchain_file {
        notes.push(format!(
            "Divergência: o toolchain file {file} pode redefinir CMAKE_C_COMPILER e \
             prevalecer sobre esta escolha; confira o compilador na CDB após configurar."
        ));
    }
    if let Some(value) = preset_compiler.as_deref().filter(|value| *value != path) {
        notes.push(format!(
            "Divergência: o preset define CMAKE_C_COMPILER={value}; a escolha do kit vai \
             depois na linha de comando e prevalece."
        ));
    }
    if let Some(cache_compiler) = cache_compiler.filter(|cache_compiler| *cache_compiler != path) {
        notes.push(format!(
            "O próximo configure troca {cache_compiler} por {path}: o CMake recria o cache e a IDE \
             confere suas definições, repetindo o configure uma vez se preciso. Até lá, \
             CDB e index.context continuam descrevendo {cache_compiler}."
        ));
    }
    notes
}

fn detected_note(detected: &[ProjectEnvironment]) -> String {
    let c: Vec<_> = detected.iter().filter(|env| env.language == "c").collect();
    if c.is_empty() {
        return "Detectado: nenhum driver C nas pastas do detector; informe o caminho da \
                instalação."
            .to_owned();
    }
    let shown: Vec<String> = c
        .iter()
        .take(DETECTED_SHOWN)
        .map(|env| format!("{} — {}", env.version, env.executable))
        .collect();
    let rest = c.len().saturating_sub(DETECTED_SHOWN);
    format!(
        "Detectado: {}{}",
        shown.join("; "),
        if rest > 0 {
            format!(" (+{rest})")
        } else {
            String::new()
        }
    )
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use kinein_protocol::{ProjectEnvironment, ToolInfo, ToolStatus, ToolchainRole};

    use super::{Measured, plan_c_compiler};
    use crate::toolchain::Toolchain;

    fn temp_root(name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("kinein-toolchain-selection")
            .join(format!("{}-{name}", std::process::id()));
        drop(std::fs::remove_dir_all(&dir));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn gcc(path: &str) -> Vec<ToolInfo> {
        vec![ToolInfo {
            id: "gcc".to_owned(),
            display_name: "GCC".to_owned(),
            status: ToolStatus::Detected,
            path: Some(path.to_owned()),
            version: None,
            suggested_install: None,
            message: None,
        }]
    }

    #[test]
    fn an_installation_path_is_planned_kept_and_reported_missing() {
        let root = temp_root("plan");
        let driver = root.join("gcc local ação");
        std::fs::write(&driver, "").unwrap();
        let driver = driver.to_str().unwrap();
        let detected = [ProjectEnvironment {
            language: "c".to_owned(),
            executable: driver.to_owned(),
            version: "gcc 16".to_owned(),
        }];
        let tools = gcc("/usr/bin/gcc");
        let measured = Measured {
            version: Some("gcc 16"),
            detected: &detected,
            preset: Some("dev"),
        };
        let plan = plan_c_compiler(&root, &tools, "dev", Some(driver), measured).unwrap();
        assert!(
            plan.before.is_none() && plan.after.contains(driver),
            "{plan:?}"
        );
        assert!(
            plan.notes
                .iter()
                .any(|note| note.contains("Detectado: gcc 16"))
        );
        assert!(
            !root.join(".kinein/toolchain.json").exists(),
            "a prévia não grava"
        );

        std::fs::create_dir_all(root.join(".kinein")).unwrap();
        std::fs::write(root.join(".kinein/toolchain.json"), &plan.after).unwrap();
        let kit = Toolchain::resolve_kit(&root, &tools, "dev");
        assert_eq!(kit.chosen(ToolchainRole::CCompiler), Some(driver));
        assert_eq!(
            kit.program_for(ToolchainRole::CCompiler),
            Some(PathBuf::from(driver))
        );
        assert!(kit.unavailable_compiler().is_none());
        assert!(plan_c_compiler(&root, &tools, "dev", Some(driver), measured).is_err());

        std::fs::remove_file(driver).unwrap();
        let kit = Toolchain::resolve_kit(&root, &tools, "dev");
        assert_eq!(kit.program_for(ToolchainRole::CCompiler), None);
        assert!(kit.cmake_arguments().is_empty());
        let reason = kit.unavailable_compiler().unwrap();
        assert!(
            reason.contains(driver) && reason.contains("kit dev"),
            "{reason}"
        );
        let back = plan_c_compiler(&root, &tools, "dev", None, measured).unwrap();
        assert!(!back.after.contains(driver), "{back:?}");
        assert!(back.summary.contains("automático"));
    }
}
