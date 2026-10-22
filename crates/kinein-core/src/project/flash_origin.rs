//! ESP-IDF build provenance, read from its generated description and existing recipe parser.
use kinein_protocol::{FlashRecipe, ProjectModel};
use serde::Deserialize;
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
    Ok(FlashOrigin {
        build,
        recipe,
        sources: vec![
            (description_path, description_revision),
            (config, config_revision),
            (recipe_path, recipe_revision),
        ],
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
