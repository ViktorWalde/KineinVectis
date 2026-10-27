//! UF2 selected within the configured `CMake` build and tied to its single ELF.
use super::uf2;
use kinein_protocol::PicoFlashSelection;
use std::{
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
};

pub(super) struct Artifact {
    pub path: PathBuf,
    pub revision: String,
    pub image: uf2::Image,
    pub sources: Vec<(PathBuf, String)>,
    pub evidence: Vec<String>,
}

pub(super) fn read(
    root: &Path,
    target: &str,
    selection: &PicoFlashSelection,
    cancel: &AtomicBool,
) -> Result<Artifact, String> {
    let path =
        crate::platform::canonicalize(&root.join(&selection.image)).map_err(|e| e.to_string())?;
    if !path.starts_with(root) {
        return Err(
            "o UF2 precisa estar dentro do workspace; aliases externos são recusados".into(),
        );
    }
    let origin = crate::project::flash_origin::cmake_target(root, target)?;
    if !path.starts_with(&origin.build) {
        return Err("o UF2 precisa estar no build canônico do alvo CMake escolhido".into());
    }
    let (bytes, revision) = super::super::artifact::read_bounded(&path, uf2::MAX_BYTES)?;
    let (elf, elf_revision) =
        super::super::artifact::read_bounded(&origin.artifact, 64 * 1024 * 1024)?;
    let image = uf2::validate(&elf, &bytes, cancel)?;
    let evidence = vec![
        format!(
            "alvo UF2: {target} ({}, codemodel do .kinein/build)",
            origin.id
        ),
        format!(
            "ELF conferido: {}; SHA256 {}",
            origin.artifact.display(),
            super::super::snapshot::digest(&elf)
        ),
        format!(
            "páginas UF2: {} × 256 bytes, 0x{:08x}–0x{:08x}; bytes ELF e padding zero conferidos",
            image.pages, image.start, image.end
        ),
    ];
    let mut sources = origin.sources;
    sources.push((origin.artifact, elf_revision));
    Ok(Artifact {
        path,
        revision,
        image,
        sources,
        evidence,
    })
}
