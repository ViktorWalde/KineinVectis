//! O PLANO de uma acao: o que sera escrito, calculado antes de escrever.
//!
//! O plano existe porque o contrato das Configuration Actions (spec 9.1 §10)
//! e "explicar, mostrar preview, aplicar com consentimento e validar". Preview
//! e apply calculam **o mesmo** plano; o preview so nao o grava. Isso e o que
//! garante que o diff mostrado e o diff aplicado — nada de um caminho de
//! codigo para exibir e outro para escrever.

use std::path::Path;

use kinein_protocol::ConfigActionFileChange;

use super::error::ConfigActionError;
use crate::fsops;

/// Uma mudanca planejada em um arquivo, relativa a raiz do workspace.
#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) struct PlannedFile {
    /// Caminho relativo a raiz (`CMakeLists.txt`, `.kinein/runconfigs.json`).
    pub(super) path: String,
    /// Conteudo lido no disco; `None` quando o arquivo sera criado.
    pub(super) before: Option<String>,
    /// Conteudo a gravar.
    pub(super) after: String,
}

/// O que uma acao fara, pronto para virar preview ou escrita.
#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) struct ActionPlan {
    /// Uma linha dizendo o que sera feito.
    pub(super) summary: String,
    /// Arquivos afetados; vazio em acoes `inspect` e `job`.
    pub(super) files: Vec<PlannedFile>,
    /// Linhas de relatorio (acoes `inspect`).
    pub(super) report: Vec<String>,
    /// Avisos que o usuario precisa ler antes de aplicar.
    pub(super) notes: Vec<String>,
}

impl ActionPlan {
    /// Plano que so escreve um arquivo.
    pub(super) fn edit(summary: impl Into<String>, file: PlannedFile) -> Self {
        Self {
            summary: summary.into(),
            files: vec![file],
            report: Vec::new(),
            notes: Vec::new(),
        }
    }

    /// Plano que so relata (nao escreve nada).
    pub(super) fn report(summary: impl Into<String>, report: Vec<String>) -> Self {
        Self {
            summary: summary.into(),
            files: Vec::new(),
            report,
            notes: Vec::new(),
        }
    }

    /// Plano sem arquivo textual: o efeito acontece em outro dominio do core.
    pub(super) fn effect(summary: impl Into<String>) -> Self {
        Self {
            summary: summary.into(),
            files: Vec::new(),
            report: Vec::new(),
            notes: Vec::new(),
        }
    }

    /// Acrescenta um aviso ao plano.
    #[must_use]
    pub(super) fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Converte os arquivos do plano no payload de preview.
    #[must_use]
    pub(super) fn file_changes(&self) -> Vec<ConfigActionFileChange> {
        self.files
            .iter()
            .map(|file| ConfigActionFileChange {
                path: file.path.clone(),
                before: file.before.clone(),
                after: file.after.clone(),
            })
            .collect()
    }
}

/// Le um arquivo do workspace, devolvendo `None` quando ele nao existe.
///
/// Usa o `fsops` (e nao `std::fs`) porque o confinamento a raiz e o limite de
/// tamanho valem aqui exatamente como valem no `fs.read` — ninguem edita um
/// `Cargo.toml` de fora do workspace por causa de um `..` no catalogo. Como no
/// resto do core, o `fsops` recebe caminho ABSOLUTO; o plano fala em caminho
/// relativo porque e assim que a UI mostra e compara.
pub(super) fn read_optional(
    root: &Path,
    relative: &str,
) -> Result<Option<String>, ConfigActionError> {
    let absolute = root.join(relative);
    if !absolute.is_file() {
        return Ok(None);
    }
    let (_, content) = fsops::read_file(root, &absolute)?;
    Ok(Some(content))
}

/// Le um arquivo obrigatorio, com mensagem de dominio quando ele falta.
pub(super) fn read_required(root: &Path, relative: &str) -> Result<String, ConfigActionError> {
    read_optional(root, relative)?.ok_or_else(|| ConfigActionError::NotApplicable {
        reason: format!("{relative} nao existe neste workspace"),
    })
}

/// Grava os arquivos do plano, honrando os snapshots do preview.
///
/// Cada acao do catalogo toca UM arquivo, entao aqui nao ha transacao
/// multi-arquivo: a barreira e a mesma do `fs.write` — escrita atomica com
/// `compare-before-save` (`ARCHITECTURE.md` §7.1). `expected` vem do preview;
/// vazio significa "sem snapshot", e ai o disco de agora e a referencia.
pub(super) fn apply_files(
    root: &Path,
    files: &[PlannedFile],
    expected: &dyn Fn(&str) -> Option<Option<String>>,
) -> Result<Vec<String>, ConfigActionError> {
    let mut written = Vec::with_capacity(files.len());
    for file in files {
        let snapshot = expected(&file.path).unwrap_or_else(|| file.before.clone());
        if let Some(previous) = snapshot {
            fsops::write_file_if_unchanged(root, &root.join(&file.path), &file.after, &previous)?;
        } else {
            if root.join(&file.path).exists() {
                return Err(ConfigActionError::Fs(fsops::FsError::ChangedOnDisk {
                    path: file.path.clone(),
                }));
            }
            create_with_parents(root, file)?;
        }
        written.push(file.path.clone());
    }
    Ok(written)
}

/// Cria um arquivo novo do plano, abrindo os diretorios que faltarem.
fn create_with_parents(root: &Path, file: &PlannedFile) -> Result<(), ConfigActionError> {
    let parent = Path::new(&file.path)
        .parent()
        .unwrap_or_else(|| Path::new(""));
    let absolute_parent = root.join(parent);
    if !parent.as_os_str().is_empty() && !absolute_parent.is_dir() {
        fsops::create_directory(root, &absolute_parent)?;
    }
    fsops::create_file(root, &root.join(&file.path), &file.after)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{ActionPlan, PlannedFile, apply_files, read_optional, read_required};

    fn temp_root(name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("kinein-configaction-plan")
            .join(format!("{}-{name}", std::process::id()));
        if dir.exists() {
            std::fs::remove_dir_all(&dir).unwrap();
        }
        std::fs::create_dir_all(&dir).unwrap();
        dir.canonicalize().unwrap()
    }

    #[test]
    fn missing_file_reads_as_none_and_required_explains() {
        let root = temp_root("read");
        assert_eq!(read_optional(&root, "Cargo.toml").unwrap(), None);
        let error = read_required(&root, "Cargo.toml").unwrap_err();
        assert!(error.to_string().contains("Cargo.toml nao existe"));
    }

    #[test]
    fn apply_writes_when_the_snapshot_still_matches() {
        let root = temp_root("write");
        std::fs::write(root.join("CMakeLists.txt"), "antes\n").unwrap();
        let files = vec![PlannedFile {
            path: "CMakeLists.txt".to_owned(),
            before: Some("antes\n".to_owned()),
            after: "depois\n".to_owned(),
        }];

        let written = apply_files(&root, &files, &|_| None).unwrap();

        assert_eq!(written, vec!["CMakeLists.txt".to_owned()]);
        assert_eq!(
            std::fs::read_to_string(root.join("CMakeLists.txt")).unwrap(),
            "depois\n"
        );
    }

    #[test]
    fn apply_refuses_when_the_disk_moved_after_the_preview() {
        let root = temp_root("stale");
        std::fs::write(root.join("CMakeLists.txt"), "outra coisa\n").unwrap();
        let files = vec![PlannedFile {
            path: "CMakeLists.txt".to_owned(),
            before: Some("outra coisa\n".to_owned()),
            after: "depois\n".to_owned(),
        }];

        // O snapshot do preview e' o que vale, nao o `before` recalculado.
        let error =
            apply_files(&root, &files, &|_| Some(Some("o que eu vi\n".to_owned()))).unwrap_err();

        assert!(error.changed_path().is_some());
        assert_eq!(
            std::fs::read_to_string(root.join("CMakeLists.txt")).unwrap(),
            "outra coisa\n"
        );
    }

    #[test]
    fn apply_creates_a_new_file_with_its_parent_directory() {
        let root = temp_root("create");
        let files = vec![PlannedFile {
            path: ".kinein/exemplo.json".to_owned(),
            before: None,
            after: "{}\n".to_owned(),
        }];

        apply_files(&root, &files, &|_| None).unwrap();

        assert_eq!(
            std::fs::read_to_string(root.join(".kinein/exemplo.json")).unwrap(),
            "{}\n"
        );
    }

    #[test]
    fn apply_refuses_to_create_a_file_that_appeared_meanwhile() {
        let root = temp_root("raced");
        std::fs::write(root.join("CMakePresets.json"), "alguem criou\n").unwrap();
        let files = vec![PlannedFile {
            path: "CMakePresets.json".to_owned(),
            before: None,
            after: "{}\n".to_owned(),
        }];

        let error = apply_files(&root, &files, &|_| None).unwrap_err();

        assert!(error.changed_path().is_some());
    }

    #[test]
    fn file_changes_mirror_the_plan() {
        let plan = ActionPlan::edit(
            "resumo",
            PlannedFile {
                path: "Cargo.toml".to_owned(),
                before: Some("a\n".to_owned()),
                after: "b\n".to_owned(),
            },
        )
        .with_note("cuidado");

        let changes = plan.file_changes();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].before.as_deref(), Some("a\n"));
        assert_eq!(plan.notes, vec!["cuidado".to_owned()]);
    }
}
