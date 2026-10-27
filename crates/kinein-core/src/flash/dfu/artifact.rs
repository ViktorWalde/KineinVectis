//! Chosen image and, for BIN, `CMake`/ELF evidence; no build tool is executed.
use super::{binary, image};
use kinein_protocol::DfuFlashSelection;
use std::{
    io::Read,
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
};

pub(super) struct Artifact {
    pub path: PathBuf,
    pub revision: String,
    pub image: image::Image,
    pub address: Option<u32>,
    pub sources: Vec<(PathBuf, String)>,
    pub evidence: Vec<String>,
}

pub(super) fn read(
    root: &Path,
    selection: &DfuFlashSelection,
    cancel: &AtomicBool,
) -> Result<Artifact, String> {
    let candidate = Path::new(&selection.image);
    let candidate = if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        root.join(candidate)
    };
    let path = crate::platform::canonicalize(&candidate).map_err(|error| error.to_string())?;
    if !path.starts_with(root) {
        return Err(
            "o arquivo escolhido precisa estar dentro do workspace; aliases externos são recusados"
                .into(),
        );
    }
    let (bytes, revision) = read_bounded(&path, image::MAX_BYTES)?;
    let Some(target) = selection.binary_target.as_deref() else {
        return Ok(Artifact {
            path,
            revision,
            image: image::parse(&bytes, cancel)?,
            address: None,
            sources: Vec::new(),
            evidence: Vec::new(),
        });
    };
    let origin = crate::project::flash_origin::cmake_target(root, target)?;
    if !path.starts_with(&origin.build) {
        return Err("o BIN precisa estar dentro do build do alvo CMake escolhido".into());
    }
    let (elf, elf_revision) = read_bounded(&origin.artifact, 64 * 1024 * 1024)?;
    let address = binary::validate(&elf, &bytes, cancel)?;
    let evidence = vec![
        format!("alvo BIN: {target} ({}, codemodel do .kinein/build)", origin.id),
        format!("ELF conferido: {}; SHA256 {}", origin.artifact.display(), crate::flash::snapshot::digest(&elf)),
        "conteúdo: bytes PT_LOAD iguais; gaps somente zero; endereço obtido do ELF, sem executar build ou conversão".into(),
    ];
    let mut sources = origin.sources;
    sources.push((origin.artifact, elf_revision));
    Ok(Artifact {
        path,
        revision,
        image: image::Image {
            alternate: selection.destination.alternate,
            release: 0,
            elements: vec![image::Element {
                address,
                size: u32::try_from(bytes.len()).map_err(|e| e.to_string())?,
            }],
        },
        address: Some(address),
        sources,
        evidence,
    })
}

fn read_bounded(path: &Path, limit: usize) -> Result<(Vec<u8>, String), String> {
    let revision = crate::platform::file_revision(path).map_err(|error| error.to_string())?;
    let mut bytes = Vec::new();
    let file = crate::fsops::open_regular_file(path).map_err(|error| error.to_string())?;
    let length = file.metadata().map_err(|error| error.to_string())?.len();
    if length == 0 || length > limit as u64 {
        return Err(format!(
            "{} precisa ser regular, não vazio e de até {limit} bytes",
            path.display()
        ));
    }
    file.take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.is_empty() || bytes.len() > limit {
        return Err(format!(
            "{} precisa ser regular, não vazio e de até {limit} bytes",
            path.display()
        ));
    }
    if crate::platform::file_revision(path).map_err(|error| error.to_string())? != revision {
        return Err("arquivo mudou durante a leitura; gere outra prévia".into());
    }
    Ok((bytes, revision))
}
