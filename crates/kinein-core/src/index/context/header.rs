//! Referência de build comprovada para o clangd transferir comandos de headers.

use std::path::Path;

use super::{CompileContext, HeaderCommand};

impl CompileContext {
    /// Não cria uma unidade CDB; a ferramenta transfere o argv original.
    pub(crate) fn header_command(&self, root: &Path, path: &Path) -> Option<HeaderCommand> {
        self.header_reference(root, path).ok().flatten()
    }

    pub(super) fn header_hint(&self, root: &Path, path: &Path) -> Option<String> {
        match self.header_reference(root, path) {
            Ok(Some(command)) => Some(format!(
                "referência semântica do alvo {}: {}; comando original transferido pelo clangd",
                command.target,
                command.source.display()
            )),
            Err(reason) => Some(format!(
                "referência semântica indisponível: {reason}; inferência pelo clangd"
            )),
            Ok(None) => None,
        }
    }

    fn header_reference(
        &self,
        root: &Path,
        path: &Path,
    ) -> Result<Option<HeaderCommand>, &'static str> {
        if !is_header(path) {
            return Ok(None);
        }
        let path = crate::platform::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        if self.units.contains_key(&path) {
            return Ok(None);
        }
        if self.cdb_stale_because.is_some() || !self.has_current_cdb(root) {
            return Err("CDB ausente ou alterada: reconfigure/recarregue o contexto");
        }
        // A configuração continua interpretada pelo clangd. Uma escolha manual
        // da base (inclusive em fragmentos condicionais) impede sobreposição.
        if path.ancestors().skip(1).any(|directory| {
            directory.join("compile_flags.txt").is_file()
                || std::fs::read_to_string(directory.join(".clangd"))
                    .is_ok_and(|text| text.contains("CompilationDatabase"))
        }) {
            return Err("base de compilação configurada manualmente");
        }
        let model = self.cmake.as_ref().ok_or("codemodel CMake ausente")?;
        let owners = model.targets_for(&path);
        if owners.len() != 1 {
            return Err("alvo proprietário ausente ou ambíguo");
        }
        let target = owners[0];
        let sources: Vec<_> = target
            .sources
            .iter()
            .filter(|source| source.compile_group.is_some())
            .collect();
        if sources.len() != 1 {
            return Err("unidade de referência ausente ou ambígua no alvo");
        }
        let source = crate::platform::canonicalize(&sources[0].path)
            .unwrap_or_else(|_| sources[0].path.clone());
        let unit = self
            .units
            .get(&source)
            .filter(|unit| !unit.ambiguous)
            .ok_or("comando da fonte ausente ou duplicado na CDB")?;
        // A transferência do clangd exige a entrada original no argv. Não
        // completar comandos incompletos nem interpretar wrappers aqui.
        if !unit.arguments.iter().skip(1).any(|argument| {
            crate::platform::canonicalize(&Path::new(&unit.directory).join(argument))
                .is_ok_and(|input| input == source)
        }) {
            return Err("entrada original ausente no comando");
        }
        Ok(Some(HeaderCommand {
            source,
            target: target.name.clone(),
            directory: unit.directory.clone(),
            arguments: unit.arguments.clone(),
        }))
    }
}

fn is_header(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| {
            matches!(
                value.to_ascii_lowercase().as_str(),
                "h" | "hh" | "hpp" | "hxx" | "ipp"
            )
        })
}
