//! Registro dos servidores principais e companheiros de cada linguagem.

use serde_json::Value;
use std::path::Path;

/// Descricao de um language server suportado.
///
/// `key`, `language` e `language_id` sao vocabulario FECHADO do projeto e
/// continuam `&'static`: sao chave de mapa e valor de protocolo. O que muda
/// em tempo de execucao e o EXECUTAVEL — por isso `command` e `args` sao
/// proprios. Ver [`ServerRegistry::set_command`].
///
/// # Um servidor PRINCIPAL e os COMPANHEIROS de uma linguagem (2026-09-13)
///
/// `key == language` e' o servidor principal: quem responde definition, hover,
/// completion, rename, simbolos. Um companheiro (`key` proprio, mesma
/// `language`) recebe o MESMO texto, publica diagnosticos que o core FUNDE
/// com os do principal num unico `event.lsp.diagnostics`, e entra na consulta
/// de code actions — e so'. E' a forma que o `ruff server` tem de existir ao
/// lado do basedpyright sem a UI saber que sao dois processos.
#[derive(Debug, Clone)]
pub(super) struct ServerSpec {
    /// Chave unica do servidor (`cpp`, `rust`, `python`, `python-ruff`).
    pub(super) key: &'static str,
    pub(super) language: &'static str,
    pub(super) command: String,
    pub(super) args: Vec<String>,
    pub(super) language_id: &'static str,
    /// Configuracao que o servidor le como `settings` (secoes por nome):
    /// vai num `workspace/didChangeConfiguration` logo apos o `initialized`
    /// e responde os `workspace/configuration` que ele pedir. `Null` = nada.
    /// E' assim que o basedpyright recebe `python.pythonPath` — o
    /// interpretador do projeto (2026-09-12).
    pub(super) settings: Value,
}

/// Tabela de servidores por linguagem.
///
/// Existe como ESTADO do manager, e nao como `const`, por um motivo medido: ate
/// 2026-09-02 nenhum teste deste repositorio conseguia observar o que o core
/// FALA com um language server, porque o executavel estava fixado no binario.
/// Com a tabela injetavel, um servidor FALSO entra no lugar e o `didOpen`/
/// `didChange`/`didClose` passam a ser verificaveis (roadmap 30, etapa 3).
///
/// A mesma costura serve ao produto na etapa 5 (toolchain como entidade): um
/// clangd fora do `PATH` e a mesma pergunta — "qual executavel roda esta
/// linguagem?".
#[derive(Debug, Clone)]
pub(super) struct ServerRegistry {
    specs: Vec<ServerSpec>,
}

impl Default for ServerRegistry {
    fn default() -> Self {
        Self {
            specs: vec![
                ServerSpec {
                    key: "cpp",
                    language: "cpp",
                    command: "clangd".to_owned(),
                    // `--clang-tidy` (P5): os avisos do tidy no canal dos
                    // diagnosticos; o kit reescreve isto pelo `clangd_args`.
                    args: vec!["--background-index".to_owned(), "--clang-tidy".to_owned()],
                    language_id: "cpp",
                    settings: Value::Null,
                },
                ServerSpec {
                    key: "rust",
                    language: "rust",
                    command: "rust-analyzer".to_owned(),
                    args: Vec::new(),
                    language_id: "rust",
                    settings: Value::Null,
                },
                // Python (cadeia do roadmaps/41 bloco B, fatia 2, 2026-09-12):
                // basedpyright pelo PyPI, `--stdio`; o interpretador do projeto
                // chega em `settings` pelo Core (`configure_python_lsp`). O
                // `ruff server` entra como companheiro pelo Core, so' quando o
                // binario existe (`use_companion`).
                ServerSpec {
                    key: "python",
                    language: "python",
                    command: "basedpyright-langserver".to_owned(),
                    args: vec!["--stdio".to_owned()],
                    language_id: "python",
                    settings: Value::Null,
                },
            ],
        }
    }
}

impl ServerRegistry {
    /// Seleciona o servidor PRINCIPAL pela extensao do arquivo, quando houver.
    ///
    /// Devolve uma COPIA de proposito: o chamador precisa do spec e do `&mut
    /// self` ao mesmo tempo (subir o servidor), e o spec tem duas strings.
    pub(super) fn spec_for_path(&self, path: &Path) -> Option<ServerSpec> {
        let language = language_for_path(path)?;
        self.specs.iter().find(|spec| spec.key == language).cloned()
    }

    /// Os companheiros da linguagem do arquivo (sem o principal), na ordem em
    /// que foram registrados.
    pub(super) fn companions_for_path(&self, path: &Path) -> Vec<ServerSpec> {
        let Some(language) = language_for_path(path) else {
            return Vec::new();
        };
        self.companions_of(language)
    }

    /// Os companheiros de `language`, sem o principal.
    pub(super) fn companions_of(&self, language: &str) -> Vec<ServerSpec> {
        self.specs
            .iter()
            .filter(|spec| spec.language == language && spec.key != spec.language)
            .cloned()
            .collect()
    }

    /// O `languageId` do protocolo de uma linguagem (o do principal).
    pub(super) fn language_id_of(&self, language: &str) -> &'static str {
        self.specs
            .iter()
            .find(|spec| spec.key == language)
            .map_or("plaintext", |spec| spec.language_id)
    }

    /// A linguagem de uma chave de servidor, se ela existe na tabela.
    pub(super) fn language_of(&self, key: &str) -> Option<&'static str> {
        self.specs
            .iter()
            .find(|spec| spec.key == key)
            .map(|spec| spec.language)
    }

    /// Todas as chaves de servidor da linguagem: o principal primeiro.
    pub(super) fn keys_of(&self, language: &str) -> Vec<&'static str> {
        let mut keys: Vec<&'static str> = self
            .specs
            .iter()
            .filter(|spec| spec.language == language)
            .map(|spec| spec.key)
            .collect();
        keys.sort_by_key(|key| *key != language);
        keys
    }

    /// Troca o executavel de um servidor conhecido; `false` se ele nao existe.
    pub(super) fn set_command(&mut self, key: &str, command: &str, args: &[&str]) -> bool {
        let Some(spec) = self.specs.iter_mut().find(|spec| spec.key == key) else {
            return false;
        };
        command.clone_into(&mut spec.command);
        spec.args = args.iter().map(|argument| (*argument).to_owned()).collect();
        true
    }

    /// Troca a configuracao que um servidor recebe; vale na PROXIMA subida.
    pub(super) fn set_settings(&mut self, key: &str, settings: Value) -> bool {
        let Some(spec) = self.specs.iter_mut().find(|spec| spec.key == key) else {
            return false;
        };
        spec.settings = settings;
        true
    }

    /// Registra (ou substitui) um companheiro de `language`. `false` quando a
    /// linguagem nao tem principal ou `key` colide com o de um principal.
    pub(super) fn add_companion(
        &mut self,
        language: &'static str,
        key: &'static str,
        command: &str,
        args: &[&str],
    ) -> bool {
        let Some(principal) = self.specs.iter().find(|spec| spec.key == language) else {
            return false;
        };
        if key == language
            || self
                .specs
                .iter()
                .any(|s| s.key == key && s.language != language)
        {
            return false;
        }
        let language_id = principal.language_id;
        self.remove(key);
        self.specs.push(ServerSpec {
            key,
            language,
            command: command.to_owned(),
            args: args.iter().map(|argument| (*argument).to_owned()).collect(),
            language_id,
            settings: Value::Null,
        });
        true
    }

    /// Tira um companheiro da tabela; `false` se nao existia (ou e' principal).
    pub(super) fn remove(&mut self, key: &str) -> bool {
        let antes = self.specs.len();
        self.specs
            .retain(|spec| !(spec.key == key && spec.key != spec.language));
        self.specs.len() != antes
    }
}

/// Linguagem do arquivo pela extensao, quando ha uma suportada.
pub(super) fn language_for_path(path: &Path) -> Option<&'static str> {
    let suffix = path.extension()?.to_str()?.to_lowercase();
    match suffix.as_str() {
        "c" | "h" | "cc" | "cpp" | "cxx" | "hpp" | "hh" | "ipp" => Some("cpp"),
        "rs" => Some("rust"),
        "py" | "pyi" | "pyw" => Some("python"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::ServerRegistry;
    use std::path::Path;

    #[test]
    fn unsupported_extensions_have_no_server() {
        let registry = ServerRegistry::default();
        assert!(registry.spec_for_path(Path::new("/tmp/nota.txt")).is_none());
        assert!(registry.spec_for_path(Path::new("/tmp/main.rs")).is_some());
        assert!(registry.spec_for_path(Path::new("/tmp/app.cpp")).is_some());
    }

    /// A troca de executavel e o que torna o LSP testavel (roadmap 30 §3).
    #[test]
    fn the_command_of_a_known_language_can_be_replaced() {
        let mut registry = ServerRegistry::default();
        assert!(registry.set_command("rust", "/tmp/falso", &["--x"]));
        assert!(!registry.set_command("cobol", "/tmp/falso", &[]));

        let spec = registry.spec_for_path(Path::new("/tmp/main.rs")).unwrap();
        assert_eq!(spec.command, "/tmp/falso");
        assert_eq!(spec.args, ["--x"]);
        assert_eq!(spec.language_id, "rust", "o vocabulario nao muda");
    }

    /// Um companheiro anda ao lado do principal: mesma linguagem, chave
    /// propria, nunca no lugar dele — e sai sem levar o principal junto.
    #[test]
    fn a_companion_sits_beside_the_principal_and_never_replaces_it() {
        let mut registry = ServerRegistry::default();
        let py = Path::new("/tmp/app.py");
        assert!(registry.companions_for_path(py).is_empty());
        assert!(registry.add_companion("python", "python-ruff", "/usr/bin/ruff", &["server"]));
        assert!(
            !registry.add_companion("python", "python", "/x", &[]),
            "a chave do principal nao vira companheiro"
        );
        assert!(
            !registry.add_companion("cobol", "cobol-lint", "/x", &[]),
            "linguagem sem principal nao tem companheiro"
        );
        assert!(
            !registry.add_companion("rust", "python-ruff", "/x", &[]),
            "a mesma chave nao serve a duas linguagens"
        );
        let principal = registry.spec_for_path(py).unwrap();
        assert_eq!(principal.key, "python");
        assert!(
            principal.command.contains("basedpyright"),
            "{}",
            principal.command
        );
        let companheiros = registry.companions_for_path(py);
        assert_eq!(companheiros.len(), 1);
        assert_eq!(companheiros[0].key, "python-ruff");
        assert_eq!(companheiros[0].language_id, "python", "herda o languageId");
        assert_eq!(companheiros[0].args, ["server"]);
        assert_eq!(registry.keys_of("python"), ["python", "python-ruff"]);
        // Registrar de novo SUBSTITUI (o binario detectado mudou), nao duplica.
        assert!(registry.add_companion("python", "python-ruff", "/opt/ruff", &["server"]));
        assert_eq!(registry.companions_of("python").len(), 1);
        assert_eq!(registry.companions_of("python")[0].command, "/opt/ruff");
        assert!(registry.remove("python-ruff"));
        assert!(!registry.remove("python-ruff"), "ja' nao existia");
        assert!(!registry.remove("python"), "o principal nao sai por aqui");
        assert!(registry.spec_for_path(py).is_some());
        assert!(registry.companions_for_path(py).is_empty());
    }
}
