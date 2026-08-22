//! Contexto de compilacao efetivo de um arquivo (L3, fatia 1).
//!
//! Le o `compile_commands.json` que o proprio build ja gera e responde: qual
//! comando se aplica a ESTE arquivo, e de onde ele veio. Nada aqui invoca
//! compilador nem reimplementa parsing de build system — o banco de comandos e
//! a fonte, e o `CMake` ja o produz.
//!
//! A parte que da valor nao e achar o comando: e dizer com que CONFIANCA ele
//! foi achado. Header nao tem entrada propria, o `clangd` empresta a de outra
//! unidade por heuristica, e essa heuristica erra. Mostrar contexto emprestado
//! como se fosse proprio e a IDE mentindo sobre a propria certeza.

use std::path::{Path, PathBuf};

use kinein_protocol::{CommandOrigin, FileContextResult};

/// Uma entrada do banco de comandos, ja normalizada.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntradaDeCompilacao {
    /// Arquivo compilado, absoluto.
    pub file: PathBuf,
    /// Comando completo.
    pub command: String,
}

/// Resolve o contexto de compilacao de um arquivo.
///
/// `root` e a raiz do workspace; `alvo` o arquivo consultado (absoluto).
#[must_use]
pub fn file_context(root: &Path, alvo: &Path) -> FileContextResult {
    let build_dir = root.join(".kinein").join("build");
    let banco = build_dir.join("compile_commands.json");
    let relativo = relativo(root, alvo);

    let Some(entradas) = ler_banco(&banco) else {
        return FileContextResult {
            path: relativo,
            origin: CommandOrigin::None,
            borrowed_from: None,
            target: None,
            compiler: None,
            standard: None,
            defines: Vec::new(),
            includes: Vec::new(),
            flags: Vec::new(),
            database: None,
            command: None,
            // O gesto que falta, nao so o sintoma.
            note: Some(
                "sem banco de comandos: configure o projeto uma vez para gerar \
                 .kinein/build/compile_commands.json"
                    .to_owned(),
            ),
        };
    };

    let (entrada, origem, emprestado) = escolher(&entradas, alvo);
    let Some(entrada) = entrada else {
        return FileContextResult {
            path: relativo,
            origin: CommandOrigin::None,
            borrowed_from: None,
            target: None,
            compiler: None,
            standard: None,
            defines: Vec::new(),
            includes: Vec::new(),
            flags: Vec::new(),
            database: Some(relativo_str(root, &banco)),
            command: None,
            note: Some(
                "o arquivo nao esta no banco de comandos e nenhuma unidade \
                 vizinha serve de referencia — ele pode estar fora do build"
                    .to_owned(),
            ),
        };
    };

    let partes = separar(&entrada.command);
    FileContextResult {
        path: relativo,
        origin: origem,
        borrowed_from: emprestado.map(|caminho| relativo_str(root, &caminho)),
        target: None,
        compiler: partes.compiler,
        standard: partes.standard,
        defines: partes.defines,
        includes: partes.includes,
        flags: partes.flags,
        database: Some(relativo_str(root, &banco)),
        command: Some(entrada.command.clone()),
        note: None,
    }
}

/// Escolhe a entrada que se aplica ao arquivo.
///
/// Ordem deliberada, da maior para a menor confianca:
///
/// 1. entrada PROPRIA — a unica resposta exata;
/// 2. unidade com o mesmo nome-base no mesmo diretorio (`a.hpp` -> `a.cpp`),
///    que e a relacao mais forte entre header e implementacao;
/// 3. qualquer unidade do mesmo diretorio, que compartilha include path e
///    defines na esmagadora maioria dos projetos.
///
/// Nao ha passo 4. "Qualquer unidade do projeto" seria chute com cara de
/// resposta: em projeto com mais de um target, o comando de outro diretorio
/// pode ter defines incompativeis, e o usuario nao teria como saber.
fn escolher<'a>(
    entradas: &'a [EntradaDeCompilacao],
    alvo: &Path,
) -> (
    Option<&'a EntradaDeCompilacao>,
    CommandOrigin,
    Option<PathBuf>,
) {
    if let Some(exata) = entradas.iter().find(|entrada| entrada.file == alvo) {
        return (Some(exata), CommandOrigin::Exact, None);
    }

    let diretorio = alvo.parent();
    let base = alvo.file_stem();

    if let (Some(diretorio), Some(base)) = (diretorio, base) {
        if let Some(irma) = entradas.iter().find(|entrada| {
            entrada.file.parent() == Some(diretorio) && entrada.file.file_stem() == Some(base)
        }) {
            return (Some(irma), CommandOrigin::Borrowed, Some(irma.file.clone()));
        }
        if let Some(vizinha) = entradas
            .iter()
            .find(|entrada| entrada.file.parent() == Some(diretorio))
        {
            return (
                Some(vizinha),
                CommandOrigin::Borrowed,
                Some(vizinha.file.clone()),
            );
        }
    }

    (None, CommandOrigin::None, None)
}

/// Comando quebrado nas partes que o usuario quer ver separadas.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PartesDoComando {
    /// Compilador (primeiro token).
    pub compiler: Option<String>,
    /// Padrao da linguagem, como escrito (`gnu++23`).
    pub standard: Option<String>,
    /// Macros, sem o `-D`.
    pub defines: Vec<String>,
    /// Diretorios de include, sem o `-I`/`-isystem`.
    pub includes: Vec<String>,
    /// O resto, na ordem original.
    pub flags: Vec<String>,
}

/// Separa o comando de compilacao em compilador, padrao, defines e includes.
///
/// Formato medido no `compile_commands.json` que este projeto gera (`CMake` +
/// Ninja + clang): campo `command` com a linha inteira. `-isystem` vem em
/// DOIS tokens (`-isystem` e o caminho), enquanto `-I` e `-D` vem colados —
/// tratar os dois iguais perderia metade dos includes.
#[must_use]
pub fn separar(command: &str) -> PartesDoComando {
    let tokens: Vec<&str> = command.split_whitespace().collect();
    let mut partes = PartesDoComando {
        compiler: tokens.first().map(|primeiro| (*primeiro).to_owned()),
        ..PartesDoComando::default()
    };
    // O primeiro token e o compilador; a varredura comeca depois dele.
    let mut indice = usize::from(!tokens.is_empty());

    while indice < tokens.len() {
        let token = tokens[indice];
        if let Some(valor) = token.strip_prefix("-D") {
            if valor.is_empty() {
                // `-D FOO`: valor no proximo token.
                if let Some(proximo) = tokens.get(indice + 1) {
                    partes.defines.push((*proximo).to_owned());
                    indice += 2;
                    continue;
                }
            } else {
                partes.defines.push(valor.to_owned());
                indice += 1;
                continue;
            }
        }
        if let Some(valor) = token.strip_prefix("-I") {
            if valor.is_empty() {
                if let Some(proximo) = tokens.get(indice + 1) {
                    partes.includes.push((*proximo).to_owned());
                    indice += 2;
                    continue;
                }
            } else {
                partes.includes.push(valor.to_owned());
                indice += 1;
                continue;
            }
        }
        if token == "-isystem" {
            if let Some(proximo) = tokens.get(indice + 1) {
                partes.includes.push((*proximo).to_owned());
                indice += 2;
                continue;
            }
        }
        if let Some(valor) = token.strip_prefix("-std=") {
            partes.standard = Some(valor.to_owned());
            indice += 1;
            continue;
        }
        partes.flags.push(token.to_owned());
        indice += 1;
    }

    partes
}

/// Le o banco de comandos, aceitando `command` ou `arguments`.
fn ler_banco(banco: &Path) -> Option<Vec<EntradaDeCompilacao>> {
    let raw = std::fs::read_to_string(banco).ok()?;
    let valor: serde_json::Value = serde_json::from_str(&raw).ok()?;
    let lista = valor.as_array()?;
    Some(
        lista
            .iter()
            .filter_map(|entrada| {
                let file = entrada.get("file")?.as_str()?;
                // `arguments` e a forma alternativa do padrao; o CMake usa
                // `command`, mas Bear e outros geradores usam a lista.
                let command = entrada
                    .get("command")
                    .and_then(serde_json::Value::as_str)
                    .map(ToOwned::to_owned)
                    .or_else(|| {
                        let args = entrada.get("arguments")?.as_array()?;
                        Some(
                            args.iter()
                                .filter_map(serde_json::Value::as_str)
                                .collect::<Vec<_>>()
                                .join(" "),
                        )
                    })?;
                Some(EntradaDeCompilacao {
                    file: PathBuf::from(file),
                    command,
                })
            })
            .collect(),
    )
}

fn relativo(root: &Path, alvo: &Path) -> String {
    relativo_str(root, alvo)
}

fn relativo_str(root: &Path, alvo: &Path) -> String {
    alvo.strip_prefix(root)
        .unwrap_or(alvo)
        .to_string_lossy()
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::{EntradaDeCompilacao, escolher, separar};
    use kinein_protocol::CommandOrigin;
    use std::path::{Path, PathBuf};

    fn entrada(caminho: &str, comando: &str) -> EntradaDeCompilacao {
        EntradaDeCompilacao {
            file: PathBuf::from(caminho),
            command: comando.to_owned(),
        }
    }

    #[test]
    fn separar_le_o_comando_real_do_projeto() {
        // Recorte FIEL do compile_commands.json que este projeto gera
        // (CMake + Ninja + clang), medido em 2026-08-21.
        let comando = "/usr/bin/clang++ -DQT_CORE_LIB -DQT_QML_LIB \
                       -I/home/ws/ui -isystem /usr/include/qt6 \
                       -isystem /usr/include/qt6/QtCore -std=gnu++23 -g \
                       -Wall -o ui/a.o -c /home/ws/ui/src/a.cpp";
        let partes = separar(comando);

        assert_eq!(partes.compiler.as_deref(), Some("/usr/bin/clang++"));
        assert_eq!(partes.standard.as_deref(), Some("gnu++23"));
        assert_eq!(partes.defines, vec!["QT_CORE_LIB", "QT_QML_LIB"]);
        assert_eq!(
            partes.includes,
            vec!["/home/ws/ui", "/usr/include/qt6", "/usr/include/qt6/QtCore"],
            "`-I` vem colado e `-isystem` em dois tokens: tratar iguais \
             perderia metade dos includes"
        );
        assert!(partes.flags.contains(&"-Wall".to_owned()));
        assert!(!partes.flags.iter().any(|f| f.starts_with("-D")));
        assert!(!partes.flags.iter().any(|f| f.starts_with("-std=")));
    }

    #[test]
    fn separar_aceita_a_forma_separada_de_d_e_i() {
        let partes = separar("cc -D FOO=1 -I /inc -c a.c");
        assert_eq!(partes.defines, vec!["FOO=1"]);
        assert_eq!(partes.includes, vec!["/inc"]);
    }

    #[test]
    fn entrada_propria_e_exata_e_nao_empresta() {
        let entradas = vec![
            entrada("/ws/src/a.cpp", "cc -c /ws/src/a.cpp"),
            entrada("/ws/src/b.cpp", "cc -c /ws/src/b.cpp"),
        ];
        let (achada, origem, de) = escolher(&entradas, Path::new("/ws/src/b.cpp"));
        assert_eq!(origem, CommandOrigin::Exact);
        assert!(de.is_none(), "entrada propria nao empresta de ninguem");
        assert_eq!(achada.unwrap().file, PathBuf::from("/ws/src/b.cpp"));
    }

    #[test]
    fn header_empresta_primeiro_da_implementacao_de_mesmo_nome() {
        // A relacao mais forte entre header e unidade e o nome-base. Se o
        // a.hpp emprestasse de outro arquivo qualquer do diretorio, o
        // contexto poderia vir de um target diferente.
        let entradas = vec![
            entrada("/ws/src/zzz.cpp", "cc -DOUTRO -c /ws/src/zzz.cpp"),
            entrada("/ws/src/a.cpp", "cc -DCERTO -c /ws/src/a.cpp"),
        ];
        let (achada, origem, de) = escolher(&entradas, Path::new("/ws/src/a.hpp"));
        assert_eq!(origem, CommandOrigin::Borrowed);
        assert_eq!(de, Some(PathBuf::from("/ws/src/a.cpp")));
        assert!(achada.unwrap().command.contains("CERTO"));
    }

    #[test]
    fn sem_irma_de_mesmo_nome_empresta_do_diretorio_mas_admite() {
        let entradas = vec![entrada("/ws/src/outro.cpp", "cc -c /ws/src/outro.cpp")];
        let (achada, origem, de) = escolher(&entradas, Path::new("/ws/src/solto.hpp"));
        assert_eq!(
            origem,
            CommandOrigin::Borrowed,
            "emprestado tem que ser dito, nunca apresentado como exato"
        );
        assert_eq!(de, Some(PathBuf::from("/ws/src/outro.cpp")));
        assert!(achada.is_some());
    }

    #[test]
    fn arquivo_de_outro_diretorio_nao_vira_chute() {
        // NAO ha passo "qualquer unidade do projeto": em projeto com mais de
        // um target, o comando de outro diretorio pode ter defines
        // incompativeis, e o usuario nao teria como saber.
        let entradas = vec![entrada(
            "/ws/outro/x.cpp",
            "cc -DINCOMPATIVEL -c /ws/outro/x.cpp",
        )];
        let (achada, origem, de) = escolher(&entradas, Path::new("/ws/src/solto.hpp"));
        assert_eq!(origem, CommandOrigin::None);
        assert!(achada.is_none());
        assert!(de.is_none());
    }
}
