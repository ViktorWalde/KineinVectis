//! Build provenance for native flashing: the ESP-IDF generated description with
//! its recipe parser, and a `CMake` executable target from the file-api reply.
use kinein_protocol::{FlashRecipe, ProjectModel};
use serde::Deserialize;
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Proven recipe and the revision of every source read to establish its origin.
#[derive(Debug)]
pub struct FlashOrigin {
    /// Canonical build directory selected by the framework/user.
    pub build: PathBuf,
    /// Images and offsets from the existing ESP-IDF recipe parser.
    pub recipe: FlashRecipe,
    /// Metadata/configuration files with their observed change token.
    pub sources: Vec<(PathBuf, String)>,
}

#[derive(Deserialize)]
struct Description {
    version: String,
    project_path: PathBuf,
    build_dir: PathBuf,
    config_file: PathBuf,
    target: String,
    app_bin: PathBuf,
}

/// Validate the selected ESP-IDF build against its project and configured target.
///
/// # Errors
/// Missing, changed, unsupported or mismatched metadata never proves a flashable build.
pub fn validate(
    root: &Path,
    selected: Option<&str>,
    model: &ProjectModel,
) -> Result<FlashOrigin, String> {
    let build = selected.map_or_else(|| root.join("build"), |path| root.join(path));
    let build = if selected.is_some() {
        canonical(&build)?
    } else {
        within(root, &build)?
    };
    let description_path = within(&build, &build.join("project_description.json"))?;
    let (body, description_revision) = read(&description_path)?;
    let description: Description = serde_json::from_str(&body)
        .map_err(|error| format!("{}: {error}", description_path.display()))?;
    if !matches!(description.version.as_str(), "1.2" | "1.3") {
        return Err(
            "formato de project_description.json ainda não validado; confira o SDK e recompile"
                .to_owned(),
        );
    }
    if crate::platform::canonicalize(&description.project_path).map_err(|e| e.to_string())? != root
        || crate::platform::canonicalize(&description.build_dir).map_err(|e| e.to_string())?
            != build
    {
        return Err("os metadados do build pertencem a outro projeto ou diretório; selecione o build correto".to_owned());
    }
    let config = canonical(&description.config_file)?;
    let (config_body, config_revision) = read(&config)?;
    let target = super::detect::idf_target_from_config(&config_body)
        .ok_or("o sdkconfig do build não comprova CONFIG_IDF_TARGET; reconfigure e compile")?;
    if target != description.target
        || model
            .target
            .chip
            .as_ref()
            .is_some_and(|chip| *chip != target)
    {
        return Err("chip do build, configuração e alvo atual divergem; reconfigure/compile e gere outra prévia".to_owned());
    }
    let recipe_path = within(&build, &build.join("flasher_args.json"))?;
    let (recipe_body, recipe_revision) = read(&recipe_path)?;
    let mut recipe =
        super::esp::flash_recipe(&recipe_body, &build).ok_or("receita de gravação inválida")?;
    if recipe.chip.as_deref() != Some(target.as_str()) || recipe.files.is_empty() {
        return Err("a receita não comprova o chip e as imagens do build selecionado".to_owned());
    }
    let app = within(&build, &build.join(&description.app_bin))?;
    let mut contains_app = false;
    for image in &mut recipe.files {
        let resolved = within(&build, Path::new(&image.file))?;
        contains_app |= resolved == app;
        if !fs::metadata(&resolved).is_ok_and(|metadata| metadata.is_file() && metadata.len() > 0) {
            return Err(
                "o build ainda não gerou todas as imagens; compile antes de gravar".to_owned(),
            );
        }
        image.file = resolved.display().to_string();
    }
    if !contains_app {
        return Err("a receita não contém a aplicação declarada pelo build selecionado".to_owned());
    }
    let effective = within(&build, &build.join("config").join("sdkconfig.json"))?;
    let (effective_body, effective_revision) = read(&effective)?;
    refuse_efuse_burning(&effective_body, &recipe)?;
    Ok(FlashOrigin {
        build,
        recipe,
        sources: vec![
            (description_path, description_revision),
            (config, config_revision),
            (recipe_path, recipe_revision),
            (effective, effective_revision),
        ],
    })
}

/// Every ESP-IDF v6.1 option whose bootloader or app burns eFuses on its own
/// at boot depends on one of these (`components/bootloader/Kconfig.projbuild`).
const EFUSE_BURNING: [&str; 2] = ["SECURE_BOOT", "SECURE_FLASH_ENC_ENABLED"];

/// The build's effective configuration (`config/sdkconfig.json`) must not turn
/// on secure boot or flash encryption: their first boot burns eFuses, an
/// irreversible effect a native preview does not cover (52 §14.39). A missing
/// key means the target lacks the feature.
fn refuse_efuse_burning(body: &str, recipe: &FlashRecipe) -> Result<(), String> {
    let config: serde_json::Map<String, Value> = serde_json::from_str(body).map_err(|_| {
        "a configuração efetiva do build (config/sdkconfig.json) não é um objeto JSON; recompile"
            .to_owned()
    })?;
    let mut enabled = Vec::new();
    for key in EFUSE_BURNING {
        match config.get(key) {
            None | Some(Value::Bool(false)) => {}
            Some(Value::Bool(true)) => enabled.push(format!("CONFIG_{key}")),
            Some(_) => {
                return Err(format!(
                    "config/sdkconfig.json traz {key} sem valor booleano; recompile"
                ));
            }
        }
    }
    if recipe.files.iter().any(|image| image.encrypted) {
        enabled.push("imagens cifradas na receita".to_owned());
    }
    if enabled.is_empty() {
        return Ok(());
    }
    Err(format!(
        "o build habilita {}: no primeiro boot o bootloader queima eFuses de forma irreversível (secure boot, criptografia de flash, modo de download da ROM), um efeito que a gravação nativa não cobre; grave pelo modo manual seguindo a documentação de segurança do ESP-IDF",
        enabled.join(", ")
    ))
}

/// A `CMake` executable target proven by the canonical build's file-api reply.
#[derive(Debug)]
pub struct CmakeOrigin {
    /// Canonical build directory (`.kinein/build`).
    pub build: PathBuf,
    /// Codemodel identity (`name::@hash`).
    pub id: String,
    /// The target's single artifact, canonical, inside the build directory.
    pub artifact: PathBuf,
    /// The artifact's ELF `e_machine`.
    pub machine: crate::dap::gdb_pick::ElfMachine,
    /// Reply files read by `CmakeModel`, with the revision seen before reading.
    pub sources: Vec<(PathBuf, String)>,
}

/// Bind `name` to its single ELF artifact through the codemodel of this root.
///
/// # Errors
/// Missing reply, another project/build, several configurations, an absent,
/// duplicated or non-executable target, several artifacts, or a non-ELF file.
/// Never falls back to the newest ELF of the build.
pub fn cmake_target(root: &Path, name: &str) -> Result<CmakeOrigin, String> {
    let build = canonical(&crate::cmake::build_dir(root))?;
    let model = crate::cmake::model::CmakeModel::load(&build).ok_or(
        "o build CMake não tem o modelo do file-api; use Configurar (CMake) no Kinein e compile antes de gravar",
    )?;
    if canonical(&model.source_dir)? != root || canonical(&model.build_dir)? != build {
        return Err(
            "o codemodel do build pertence a outro projeto ou diretório; reconfigure o projeto"
                .to_owned(),
        );
    }
    if model.configurations != 1 {
        return Err(format!(
            "o codemodel tem {} configurações; a gravação nativa exige gerador de configuração única",
            model.configurations
        ));
    }
    let mut named = model.targets.iter().filter(|target| target.name == name);
    let target = named.next().ok_or_else(|| {
        format!(
            "o alvo CMake `{name}` não existe no build configurado; reconfigure ou escolha outro"
        )
    })?;
    if named.next().is_some() {
        return Err(format!("mais de um alvo CMake se chama `{name}`"));
    }
    if target.kind != "EXECUTABLE" {
        return Err(format!(
            "o alvo `{name}` é {}, não um executável gravável",
            target.kind
        ));
    }
    let [artifact] = target.artifacts.as_slice() else {
        return Err(format!(
            "o alvo `{name}` declara {} artefatos; a gravação precisa de exatamente um",
            target.artifacts.len()
        ));
    };
    let artifact = within(&build, artifact)?;
    let mut header = Vec::with_capacity(64);
    let file = crate::fsops::open_regular_file(&artifact)
        .map_err(|error| format!("{}: {error}", artifact.display()))?;
    std::io::Read::read_to_end(&mut std::io::Read::take(file, 64), &mut header)
        .map_err(|error| format!("{}: {error}", artifact.display()))?;
    let machine = crate::dap::gdb_pick::parse_elf_machine(&header).ok_or_else(|| {
        format!(
            "{} não é ELF; a gravação nativa por sonda grava o ELF do alvo",
            artifact.display()
        )
    })?;
    Ok(CmakeOrigin {
        build,
        id: target.id.clone(),
        artifact,
        machine,
        sources: model.observed,
    })
}

fn within(root: &Path, path: &Path) -> Result<PathBuf, String> {
    let path = canonical(path)?;
    if !path.starts_with(root) {
        return Err(format!(
            "{} está fora do contexto selecionado",
            path.display()
        ));
    }
    Ok(path)
}

fn canonical(path: &Path) -> Result<PathBuf, String> {
    crate::platform::canonicalize(path).map_err(|error| {
        format!(
            "{}: {error}; compile e selecione um build válido",
            path.display()
        )
    })
}

fn read(path: &Path) -> Result<(String, String), String> {
    let revision = crate::platform::file_revision(path).map_err(|error| error.to_string())?;
    let (_, body) = crate::fsops::read_external_file(path).map_err(|error| error.to_string())?;
    if crate::platform::file_revision(path).map_err(|error| error.to_string())? != revision {
        return Err("metadados mudaram durante a leitura; gere outra prévia".to_owned());
    }
    Ok((body, revision))
}
