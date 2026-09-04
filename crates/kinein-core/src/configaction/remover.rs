//! Desfazer o que uma acao escreveu: tirar bibliotecas de um `target`.
//!
//! POR QUE ESTE MODULO EXISTE (2026-09-04). Relato de uso do autor: *"quando
//! clicar em Aplicar ter visivelmente que foi aplicado com a bolinha verde
//! ficando ativada... e tambem o botao de Aplicar ficar como Desativar"*.
//!
//! A bolinha verde ja' existe (`library::applied`), mas ela so' fecha o ciclo
//! se houver como VOLTAR. Ate' aqui a IDE sabia acrescentar e nao sabia tirar:
//! o autor que ativasse a biblioteca errada tinha de editar o `CMakeLists.txt`
//! a mao — que e' exatamente o que este dominio existe para evitar.
//!
//! O nome do botao ficou **Remover**, e nao "Desativar", por ser o que de fato
//! acontece: a linha sai do arquivo. "Desativar" sugeriria um interruptor que
//! guarda o estado em algum lugar, e nao ha' lugar nenhum — o `CMakeLists.txt`
//! E' o estado.
//!
//! A REMOCAO E' CIRURGICA: tira os alvos pedidos de dentro do
//! `target_link_libraries` e, se nao sobrar biblioteca nenhuma, tira a chamada
//! inteira. Deixar `target_link_libraries(app PRIVATE)` para tras seria deixar
//! lixo que o `CMake` aceita e ninguem entende depois.

use std::collections::BTreeMap;

use super::cmakelists::CMAKELISTS;
use super::error::ConfigActionError;
use super::plan::{ActionPlan, PlannedFile, read_required};
use super::required_param;

/// Palavras que sao MODIFICADOR e nao biblioteca dentro da chamada.
const KEYWORDS: [&str; 3] = ["PRIVATE", "PUBLIC", "INTERFACE"];

/// Tira bibliotecas de um `target_link_libraries`.
///
/// # Errors
/// `CMakeLists.txt` ausente, parametro faltando, ou nenhuma das bibliotecas
/// pedidas esta' linkada — recusar nesse caso e' melhor que "aplicar" um plano
/// que nao muda nada.
pub(super) fn remove_target_link_libraries(
    root: &std::path::Path,
    params: &BTreeMap<String, String>,
) -> Result<ActionPlan, ConfigActionError> {
    let target = required_param(params, "target")?.trim().to_owned();
    let pedidas: Vec<String> = required_param(params, "libraries")?
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|parte| !parte.is_empty())
        .map(str::to_owned)
        .collect();
    if pedidas.is_empty() {
        return Err(ConfigActionError::InvalidParam {
            name: "libraries",
            reason: "informe ao menos uma biblioteca".to_owned(),
        });
    }

    let before = read_required(root, CMAKELISTS)?;
    let (after, removidas) = sem_as_bibliotecas(&before, &target, &pedidas);
    if removidas == 0 {
        return Err(ConfigActionError::NotApplicable {
            reason: format!(
                "{} nao esta linkada em {target} no {CMAKELISTS}",
                pedidas.join(", ")
            ),
        });
    }
    Ok(ActionPlan::edit(
        format!("Remove {} de {target}", pedidas.join(", ")),
        PlannedFile {
            path: CMAKELISTS.to_owned(),
            before: Some(before),
            after,
        },
    ))
}

/// Reescreve o arquivo sem as bibliotecas pedidas; devolve quantas saiu.
fn sem_as_bibliotecas(before: &str, target: &str, pedidas: &[String]) -> (String, usize) {
    let mut saida = String::with_capacity(before.len());
    let mut removidas = 0;
    let mut resto = before;

    while let Some(inicio) = proxima_chamada(resto, target) {
        let Some(abre) = resto[inicio..].find('(') else {
            break;
        };
        let corpo_inicio = inicio + abre + 1;
        let Some(fecha) = resto[corpo_inicio..].find(')') else {
            break;
        };
        let corpo_fim = corpo_inicio + fecha;
        let corpo = &resto[corpo_inicio..corpo_fim];

        let mantidos: Vec<&str> = corpo
            .split_whitespace()
            .filter(|palavra| {
                let sai = pedidas.iter().any(|pedida| pedida == palavra);
                if sai {
                    removidas += 1;
                }
                !sai
            })
            .collect();
        // Sobrou biblioteca de verdade, ou so' o alvo e as palavras-chave?
        let sobrou_biblioteca = mantidos
            .iter()
            .skip(1)
            .any(|palavra| !KEYWORDS.contains(&palavra.to_ascii_uppercase().as_str()));

        saida.push_str(&resto[..inicio]);
        if sobrou_biblioteca {
            saida.push_str("target_link_libraries(");
            saida.push_str(&mantidos.join(" "));
            saida.push(')');
            resto = &resto[corpo_fim + 1..];
        } else {
            // Chamada inteira sai, junto da quebra de linha que a seguia, para
            // nao deixar linha em branco no lugar.
            resto = &resto[corpo_fim + 1..];
            resto = resto.strip_prefix('\n').unwrap_or(resto);
            // E a quebra que a PRECEDIA, quando ela ficou sozinha na linha.
            while saida.ends_with("\n\n") {
                saida.pop();
            }
        }
    }
    saida.push_str(resto);
    (saida, removidas)
}

/// Onde comeca a proxima chamada `target_link_libraries` deste alvo.
fn proxima_chamada(texto: &str, target: &str) -> Option<usize> {
    let minusculo = texto.to_ascii_lowercase();
    let mut de = 0;
    while let Some(posicao) = minusculo[de..].find("target_link_libraries") {
        let inicio = de + posicao;
        let apos = &texto[inicio..];
        if let Some(abre) = apos.find('(') {
            let primeiro = apos[abre + 1..]
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim_end_matches(')');
            if primeiro == target {
                return Some(inicio);
            }
        }
        de = inicio + "target_link_libraries".len();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params(target: &str, libraries: &str) -> BTreeMap<String, String> {
        BTreeMap::from([
            ("target".to_owned(), target.to_owned()),
            ("libraries".to_owned(), libraries.to_owned()),
        ])
    }

    fn root_com(conteudo: &str, nome: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir()
            .join("kinein-configaction-remover")
            .join(format!("{}-{nome}", std::process::id()));
        if dir.exists() {
            std::fs::remove_dir_all(&dir).unwrap();
        }
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("CMakeLists.txt"), conteudo).unwrap();
        dir
    }

    fn depois(plano: &ActionPlan) -> String {
        plano.files[0].after.clone()
    }

    /// Sobrando outra biblioteca, a chamada FICA — so' o nome pedido sai.
    #[test]
    fn tira_so_a_biblioteca_pedida_e_mantem_a_chamada() {
        let root = root_com(
            "add_executable(app main.cpp)\ntarget_link_libraries(app PRIVATE fmt::fmt spdlog::spdlog)\n",
            "uma",
        );
        let texto =
            depois(&remove_target_link_libraries(&root, &params("app", "fmt::fmt")).unwrap());
        assert!(texto.contains("target_link_libraries(app PRIVATE spdlog::spdlog)"));
        assert!(!texto.contains("fmt::fmt"));
    }

    /// Sem sobrar biblioteca, deixar `target_link_libraries(app PRIVATE)` para
    /// tras seria lixo que o `CMake` aceita e ninguem entende depois.
    #[test]
    fn chamada_que_ficaria_vazia_sai_inteira() {
        let root = root_com(
            "add_executable(app main.cpp)\ntarget_link_libraries(app PRIVATE fmt::fmt)\nadd_test(NAME t COMMAND app)\n",
            "vazia",
        );
        let texto =
            depois(&remove_target_link_libraries(&root, &params("app", "fmt::fmt")).unwrap());
        assert!(!texto.contains("target_link_libraries"), "sobrou: {texto}");
        assert!(texto.contains("add_test(NAME t COMMAND app)"));
    }

    /// Remover de OUTRO alvo nao pode mexer neste.
    #[test]
    fn nao_encosta_no_alvo_errado() {
        let root = root_com(
            "target_link_libraries(app PRIVATE fmt::fmt)\ntarget_link_libraries(outro PRIVATE fmt::fmt)\n",
            "alvo",
        );
        let texto =
            depois(&remove_target_link_libraries(&root, &params("outro", "fmt::fmt")).unwrap());
        assert!(texto.contains("target_link_libraries(app PRIVATE fmt::fmt)"));
        assert_eq!(
            texto.matches("fmt::fmt").count(),
            1,
            "mexeu no alvo errado: {texto}"
        );
    }

    /// "Aplicar" um plano que nao muda nada e' pior que recusar: o autor fica
    /// achando que removeu.
    #[test]
    fn remover_o_que_nao_esta_linkado_e_recusado() {
        let root = root_com("target_link_libraries(app PRIVATE fmt::fmt)\n", "ausente");
        let erro = remove_target_link_libraries(&root, &params("app", "zlib::zlib")).unwrap_err();
        assert!(format!("{erro}").contains("nao esta linkada"), "{erro}");
    }
}
