//! O que JA' esta ligado no projeto — lido do `CMakeLists.txt`.
//!
//! POR QUE ESTE ARQUIVO EXISTE (2026-09-04). Relato de uso do autor: *"ainda
//! fica confuso sobre o que estou ativado no meu projeto, com o que tenho de
//! opcao de ativar"*. Ele estava certo, e a IDE nao tinha como responder: o
//! catalogo dizia apenas se a biblioteca existe NESTA MAQUINA
//! (`LibraryStatus`), que e' outra pergunta. "Instalado no sistema" e "ligado
//! no meu projeto" sao coisas diferentes, e a tela mostrava so' a primeira.
//!
//! Sem esta resposta, a bolinha verde do painel seria decoracao: verde por
//! "existe no sistema", nao por "esta no meu build". O autor pediu
//! explicitamente a distincao, e ela e' a diferenca entre uma tela que informa
//! e uma que enfeita.
//!
//! COMO SE DESCOBRE. Procura os alvos de link do catalogo (`fmt::fmt`,
//! `SQLite::SQLite3`, ...) dentro das chamadas `target_link_libraries` dos
//! `CMakeLists.txt` do projeto. E' textual de proposito: avaliar `CMake` exigiria
//! rodar o `CMake`, e a resposta e' precisa para o que a IDE escreve — ela mesma
//! escreve essas linhas.
//!
//! LIMITES, e sao reais:
//!
//! ```text
//! alvo montado por variavel   NAO detectado: `${LIBS}` nao diz o que contem
//! linkado num sub-projeto     detectado, e isso e' desejado: esta' no build
//! comentado                   NAO detectado: a linha e' descartada antes
//! ```
//!
//! O pior caso e' dizer "nao ligado" para algo que esta' ligado por uma
//! variavel — e ai' o autor aplica de novo, o que a previa do `configaction`
//! mostra antes de escrever. Nunca o contrario.

use std::collections::BTreeSet;
use std::path::Path;

use super::catalog::DEFINITIONS;

/// Ids do catalogo cujos alvos ja' aparecem num `target_link_libraries`.
#[must_use]
pub fn applied_ids(root: &Path) -> BTreeSet<String> {
    let ligados = linked_targets(root);
    DEFINITIONS
        .iter()
        .filter(|definicao| definicao.targets.iter().any(|alvo| ligados.contains(*alvo)))
        .map(|definicao| definicao.id.to_owned())
        .collect()
}

/// Nomes que aparecem dentro de alguma chamada `target_link_libraries`.
fn linked_targets(root: &Path) -> BTreeSet<String> {
    let mut encontrados = BTreeSet::new();
    for arquivo in crate::cmake::cmake_lists_files(root) {
        let Ok(texto) = std::fs::read_to_string(&arquivo) else {
            continue;
        };
        // Comentario fora ANTES de qualquer coisa: uma linha comentada nao
        // liga nada, e contar com ela diria "ja' esta ativo" para algo que o
        // autor desativou comentando.
        let sem_comentario: String = texto
            .lines()
            .map(|linha| linha.split('#').next().unwrap_or(""))
            .collect::<Vec<_>>()
            .join("\n");
        let minusculo = sem_comentario.to_ascii_lowercase();

        let mut inicio = 0;
        while let Some(posicao) = minusculo[inicio..].find("target_link_libraries") {
            let abertura = inicio + posicao;
            let Some(parenteses) = sem_comentario[abertura..].find('(') else {
                break;
            };
            let corpo_inicio = abertura + parenteses + 1;
            let Some(fechamento) = sem_comentario[corpo_inicio..].find(')') else {
                break;
            };
            let corpo = &sem_comentario[corpo_inicio..corpo_inicio + fechamento];
            for palavra in corpo.split_whitespace() {
                encontrados.insert(palavra.to_owned());
            }
            inicio = corpo_inicio + fechamento;
        }
    }
    encontrados
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn temp_root(nome: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("kinein-library-applied")
            .join(format!("{}-{nome}", std::process::id()));
        if dir.exists() {
            std::fs::remove_dir_all(&dir).unwrap();
        }
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn reconhece_o_que_esta_ligado_e_ignora_o_comentado() {
        let root = temp_root("ligado");
        std::fs::write(
            root.join("CMakeLists.txt"),
            "add_executable(app main.cpp)\n\
             target_link_libraries(app PRIVATE fmt::fmt)\n\
             # target_link_libraries(app PRIVATE spdlog::spdlog)\n",
        )
        .unwrap();

        let ligados = applied_ids(&root);
        assert!(ligados.contains("fmt"), "nao viu o fmt ligado");
        assert!(
            !ligados.contains("spdlog"),
            "contou uma linha COMENTADA como ligada"
        );
    }

    #[test]
    fn projeto_sem_link_nenhum_devolve_vazio() {
        let root = temp_root("vazio");
        std::fs::write(
            root.join("CMakeLists.txt"),
            "add_executable(app main.cpp)\n",
        )
        .unwrap();
        assert!(applied_ids(&root).is_empty());
    }

    /// Multilinha e' o formato que a propria IDE escreve.
    #[test]
    fn chamada_em_varias_linhas_tambem_conta() {
        let root = temp_root("multilinha");
        std::fs::write(
            root.join("CMakeLists.txt"),
            "target_link_libraries(app\n    PRIVATE\n    nlohmann_json::nlohmann_json\n)\n",
        )
        .unwrap();
        assert!(applied_ids(&root).contains("nlohmann_json"));
    }
}
