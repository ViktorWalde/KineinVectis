//! O que cada parametro SIGNIFICA, e quais valores o projeto ja' oferece.
//!
//! POR QUE ESTE MODULO EXISTE (2026-09-04). Pedido do autor: *"mostrar o risco
//! de forma explicita, o que aquilo faz ou nao no projeto, e ter uma descricao
//! explicando... e quando for implementar, o usuario clicar no campo e, em vez
//! de digitar manualmente, ter a opcao de selecionar visualmente a leitura que
//! a IDE faz"*.
//!
//! Sao duas faltas diferentes e ele juntou as duas com razao:
//!
//! ```text
//! o campo nao EXPLICA    `visibility` com placeholder `PRIVATE` nao diz a
//!                        ninguem o que muda se virar `PUBLIC`
//! o campo nao SUGERE     a IDE SABE quais targets existem e quais arquivos
//!                        existem, e mesmo assim pedia que o autor digitasse
//! ```
//!
//! POR QUE POR NOME DE PARAMETRO, e nao por acao. `target` quer dizer a mesma
//! coisa em `addSourceToTarget`, `strictWarnings` e `enableOpenMP`; escrever a
//! explicacao em cada uma das acoes criaria copias que envelhecem separadas —
//! e' a mesma razao pela qual `isWordChar` virou um dono so'
//! (`docs/roadmaps/39` §5). O nome do parametro E' a convencao, e ela ja'
//! existia neste catalogo antes deste modulo.

use std::path::Path;

/// Quantos valores sugerir por campo.
///
/// Vinte cabe numa tela e ainda deixa o campo livre para o resto. Sugestao que
/// vira lista de mil itens deixa de ser sugestao e vira outro problema.
const MAX_SUGESTOES: usize = 20;

/// Extensoes que contam como fonte compilavel.
const FONTES: [&str; 8] = ["c", "cc", "cpp", "cxx", "h", "hh", "hpp", "hxx"];

/// O que o parametro faz, em uma frase.
///
/// Nome desconhecido devolve vazio: melhor nao dizer nada que inventar uma
/// explicacao para um campo que alguem acabou de criar.
#[must_use]
pub(super) fn describe(name: &str) -> &'static str {
    match name {
        "target" => {
            "O alvo do CMake que recebe a mudanca — um executavel ou uma biblioteca \
                     declarada no seu CMakeLists."
        }
        "visibility" => {
            "PRIVATE: so' este alvo usa. PUBLIC: quem linkar neste alvo tambem \
                         herda. INTERFACE: so' quem linkar, nao ele mesmo."
        }
        "sources" | "source" => "Arquivos de codigo que passam a ser compilados dentro do alvo.",
        "directory" | "directories" => {
            "Diretorio de headers que o compilador passa a \
                     procurar (-I)."
        }
        "package" => {
            "Nome do pacote como o CMake o conhece — o mesmo que vai no \
                      find_package()."
        }
        "name" => "Nome que identifica o que esta sendo criado.",
        "repository" => "URL do repositorio git de onde a dependencia e' baixada.",
        "tag" => {
            "Tag ou commit FIXO. Nunca um branch: `main` muda sob os seus pes e \
                  transforma um build que passava em um que falha."
        }
        "libraries" => {
            "Alvos de link, como `fmt::fmt` — o nome que a biblioteca exporta, \
                        nao o nome do arquivo."
        }
        "standard" => "Versao do padrao C++ que o projeto exige (11, 14, 17, 20, 23, 26).",
        "werror" => {
            "ON transforma todo aviso em erro. Num projeto que ja' tem avisos, isso \
                     para o build inteiro — ligue depois de limpar."
        }
        "sanitizers" => {
            "Verificacoes em tempo de execucao. `address` pega acesso invalido de \
                         memoria; `undefined` pega comportamento indefinido. Custam \
                         desempenho: sao para teste, nao para release."
        }
        "chip" => "O chip da placa, como o probe-rs o chama (ex.: STM32F401RETx).",
        "edition" => "Edition do Rust — muda regras da linguagem, nao a versao do compilador.",
        "features" | "enables" => "Features que este item liga por padrao.",
        "version" => "Versao pedida. Vazio aceita qualquer uma que o sistema tenha.",
        _ => "",
    }
}

/// Valores reais do projeto para este parametro.
///
/// Le' o disco a cada chamada de proposito: o autor pode ter criado um alvo ou
/// um arquivo desde que abriu o painel, e uma lista em cache mostraria um
/// projeto que nao existe mais.
#[must_use]
pub(super) fn suggestions(root: &Path, name: &str) -> Vec<String> {
    match name {
        "target" => crate::cmake::list_targets(root)
            .into_iter()
            .map(|alvo| alvo.name)
            .chain(
                crate::cmake::targets_from_source(root)
                    .into_iter()
                    .map(|alvo| alvo.name),
            )
            .take(MAX_SUGESTOES)
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect(),
        "visibility" => vec![
            "PRIVATE".to_owned(),
            "PUBLIC".to_owned(),
            "INTERFACE".to_owned(),
        ],
        "werror" => vec!["OFF".to_owned(), "ON".to_owned()],
        "standard" => ["11", "14", "17", "20", "23", "26"]
            .iter()
            .map(|v| (*v).to_owned())
            .collect(),
        "sources" | "source" => arquivos_de_fonte(root),
        "directory" | "directories" => diretorios(root),
        _ => Vec::new(),
    }
}

/// Fontes compilaveis do projeto, em caminho relativo.
fn arquivos_de_fonte(root: &Path) -> Vec<String> {
    let mut achados = Vec::new();
    varrer(root, 0, &mut |caminho| {
        if caminho.is_file()
            && caminho
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| FONTES.contains(&e.to_ascii_lowercase().as_str()))
            && let Ok(relativo) = caminho.strip_prefix(root)
        {
            achados.push(relativo.display().to_string());
        }
    });
    achados.sort();
    achados.truncate(MAX_SUGESTOES);
    achados
}

/// Diretorios do projeto, em caminho relativo.
fn diretorios(root: &Path) -> Vec<String> {
    let mut achados = Vec::new();
    varrer(root, 0, &mut |caminho| {
        if caminho.is_dir()
            && let Ok(relativo) = caminho.strip_prefix(root)
            && !relativo.as_os_str().is_empty()
        {
            achados.push(relativo.display().to_string());
        }
    });
    achados.sort();
    achados.truncate(MAX_SUGESTOES);
    achados
}

/// Varredura rasa do projeto, pulando o que nao e' do autor.
///
/// Profundidade limitada e diretorios de build fora: sugerir
/// `.kinein/build/CMakeFiles/...` seria oferecer o que a propria IDE gerou.
fn varrer(atual: &Path, nivel: usize, visitar: &mut impl FnMut(&Path)) {
    const PROFUNDIDADE_MAXIMA: usize = 3;
    const IGNORADOS: [&str; 7] = [
        ".kinein",
        ".git",
        "build",
        "target",
        "node_modules",
        "out",
        "dist",
    ];

    let Ok(entradas) = std::fs::read_dir(atual) else {
        return;
    };
    for entrada in entradas.flatten() {
        let caminho = entrada.path();
        let nome = entrada.file_name().to_string_lossy().into_owned();
        if nome.starts_with('.') || IGNORADOS.contains(&nome.as_str()) {
            continue;
        }
        visitar(&caminho);
        if caminho.is_dir() && nivel < PROFUNDIDADE_MAXIMA {
            varrer(&caminho, nivel + 1, visitar);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(nome: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir()
            .join("kinein-configaction-parametros")
            .join(format!("{}-{nome}", std::process::id()));
        if dir.exists() {
            std::fs::remove_dir_all(&dir).unwrap();
        }
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::create_dir_all(dir.join(".kinein/build/CMakeFiles")).unwrap();
        std::fs::write(dir.join("src/main.cpp"), "int main(){}").unwrap();
        std::fs::write(dir.join("src/util.hpp"), "#pragma once").unwrap();
        std::fs::write(dir.join("LEIAME.md"), "texto").unwrap();
        std::fs::write(dir.join(".kinein/build/CMakeFiles/gerado.cpp"), "").unwrap();
        dir
    }

    #[test]
    fn sugere_as_fontes_do_autor_e_nao_o_que_a_ide_gerou() {
        let root = temp_root("fontes");
        let fontes = suggestions(&root, "sources");
        assert!(fontes.contains(&"src/main.cpp".to_owned()), "{fontes:?}");
        assert!(fontes.contains(&"src/util.hpp".to_owned()), "{fontes:?}");
        assert!(
            !fontes.iter().any(|f| f.contains(".kinein")),
            "sugeriu o que a propria IDE gerou: {fontes:?}"
        );
        assert!(
            !fontes
                .iter()
                .any(|f| f.to_ascii_lowercase().ends_with(".md")),
            "markdown nao e' fonte compilavel: {fontes:?}"
        );
    }

    #[test]
    fn sugere_diretorios_sem_o_build() {
        let root = temp_root("dirs");
        // As DUAS grafias: o catalogo usa `directories` e a tabela so' tinha
        // `directory`, buraco que a exercitacao contra um projeto real achou
        // em 2026-09-04 — o campo aparecia sem sugestao e sem explicacao.
        for chave in ["directory", "directories"] {
            let dirs = suggestions(&root, chave);
            assert!(dirs.contains(&"src".to_owned()), "{chave}: {dirs:?}");
            assert!(
                !dirs.iter().any(|d| d.contains(".kinein")),
                "{chave}: {dirs:?}"
            );
            assert!(!describe(chave).is_empty(), "{chave} sem explicacao");
        }
    }

    /// Campo com valores fechados nao deveria ser texto livre.
    #[test]
    fn campos_de_escolha_fechada_sugerem_todas_as_opcoes() {
        let root = temp_root("fechados");
        assert_eq!(suggestions(&root, "visibility").len(), 3);
        assert_eq!(suggestions(&root, "werror"), vec!["OFF", "ON"]);
        assert!(suggestions(&root, "standard").contains(&"20".to_owned()));
    }

    /// Melhor nao dizer nada que inventar explicacao para um campo novo.
    #[test]
    fn parametro_desconhecido_nao_ganha_explicacao_inventada() {
        assert_eq!(describe("campo_que_nao_existe"), "");
        assert!(describe("target").contains("CMakeLists"));
        assert!(describe("tag").contains("branch"));
    }
}
