//! Bibliotecas C/C++ curadas: o que existe no mundo, o que existe nesta
//! maquina, e o que seria preciso para usar uma delas.
//!
//! Nasceu em 2026-09-03 (etapa 19 do `roadmaps/35`), da decisao do autor de
//! que a IDE **endossa** as bibliotecas que oferece, em vez de apenas listar o
//! que esta instalado.
//!
//! # A fronteira que este dominio NAO cruza
//!
//! Este modulo **nao escreve no arquivo de build**. Ele produz um plano — "para
//! usar `fmt`, e preciso um `find_package` e um `target_link_libraries` neste
//! alvo" — e quem escreve continua sendo o `configaction`, que ja tem escritor,
//! preview e consentimento.
//!
//! Dois escritores do mesmo arquivo e a forma de eles divergirem em silencio, e
//! por isso o criterio de aceite desta fatia e um grep pelo nome do arquivo de
//! build dentro desta pasta: ele tem que voltar ZERO, inclusive em comentario.
//! Um criterio que exige interpretar o que e' codigo e o que e' prosa nao e'
//! criterio — e' opiniao com aparencia de medida.
//!
//! Se este dominio aprender a escrever, o corte falhou.

mod availability;
mod catalog;

use kinein_protocol::{LibraryInfo, LibraryPlan, LibraryStatus, LibraryStep};

use availability::Availability;

/// O catalogo inteiro, ja cruzado com o que existe nesta maquina.
#[must_use]
pub fn list() -> Vec<LibraryInfo> {
    catalog::definitions()
        .iter()
        .map(|entry| {
            let status = match availability::detect(entry.package_name) {
                Availability::Detected => LibraryStatus::Detected,
                Availability::NotDetected => LibraryStatus::NotDetected,
            };
            LibraryInfo {
                id: entry.id.to_owned(),
                name: entry.name.to_owned(),
                summary: entry.summary.to_owned(),
                category: entry.category.to_owned(),
                license: entry.license.to_owned(),
                pinned_version: entry.pinned_version.to_owned(),
                released_at: entry.released_at.to_owned(),
                documentation: entry.documentation.to_owned(),
                repository: entry.repository.to_owned(),
                status,
            }
        })
        .collect()
}

/// O que seria preciso para o alvo `target` usar a biblioteca `id`.
///
/// Devolve PASSOS, nao texto de arquivo: cada passo nomeia a Configuration
/// Action que o executaria. Quem aplica e o dominio `configaction`, com o
/// preview e o consentimento que ele ja tem.
///
/// A escolha entre `find_package` e `FetchContent` sai da MEDICAO, nao da
/// preferencia: se o config package esta no sistema, usa-se ele — baixar e
/// compilar o que ja esta instalado e desperdicio que o usuario paga em tempo
/// de build.
pub fn plan(id: &str, target: &str) -> Result<LibraryPlan, String> {
    let Some(entry) = catalog::find(id) else {
        return Err(format!("biblioteca desconhecida: {id}"));
    };
    if target.trim().is_empty() {
        return Err("o alvo do CMake nao pode ser vazio".to_owned());
    }

    // O caminho sai da MEDICAO: se o config package esta no sistema, usa-se
    // ele. Baixar e compilar o que ja esta instalado e desperdicio que o
    // usuario paga em tempo de build.
    let detectada = availability::detect(entry.package_name) == Availability::Detected;
    let usa_find_package = detectada;

    let mut steps = Vec::new();
    if usa_find_package {
        steps.push(LibraryStep {
            action_id: "cmake.findPackage".to_owned(),
            summary: format!(
                "find_package({} CONFIG REQUIRED) — o pacote esta instalado nesta maquina",
                entry.package_name
            ),
        });
    } else {
        steps.push(LibraryStep {
            action_id: "cmake.fetchContent".to_owned(),
            summary: format!(
                "FetchContent de {} na tag {} — o pacote nao foi encontrado no sistema, \
                 entao ele e baixado e compilado junto, com a versao PINADA",
                entry.repository, entry.pinned_version
            ),
        });
    }
    steps.push(LibraryStep {
        action_id: "cmake.addTargetLinkLibraries".to_owned(),
        summary: format!(
            "target_link_libraries({target} PRIVATE {})",
            entry.targets.join(" ")
        ),
    });

    Ok(LibraryPlan {
        id: entry.id.to_owned(),
        target: target.to_owned(),
        uses_find_package: usa_find_package,
        detected: detectada,
        pinned_version: entry.pinned_version.to_owned(),
        targets: entry.targets.iter().map(|t| (*t).to_owned()).collect(),
        steps,
        searched_paths: if detectada {
            Vec::new()
        } else {
            availability::search_paths()
                .iter()
                .map(|p| p.display().to_string())
                .collect()
        },
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use kinein_protocol::LibraryStatus;

    use super::{availability, catalog, list, plan};

    /// O catalogo AFIRMA que a biblioteca e' solida; esta afirmacao tem
    /// requisitos, e este teste e' quem os cobra. Uma entrada nova que
    /// esqueca a licenca ou a data reprova aqui, nao no uso.
    #[test]
    fn every_catalog_entry_carries_its_audit() {
        for entry in catalog::definitions() {
            assert!(!entry.license.is_empty(), "{} sem licenca", entry.id);
            assert!(
                !entry.pinned_version.is_empty() && entry.pinned_version != "latest",
                "{} sem versao PINADA — 'latest' nao e' pino",
                entry.id
            );
            // Data ISO: a idade da entrada tem que ser visivel para quem le.
            assert_eq!(
                entry.released_at.len(),
                10,
                "{} sem data de release em AAAA-MM-DD",
                entry.id
            );
            assert!(
                entry.summary.len() > 20,
                "{} sem a frase que explica o que ela FAZ",
                entry.id
            );
            assert!(
                entry.documentation.starts_with("https://"),
                "{} sem doc oficial",
                entry.id
            );
            assert!(!entry.targets.is_empty(), "{} sem alvo a linkar", entry.id);
        }
    }

    /// Ids duplicados quebrariam `plan` em silencio: `find` acha o primeiro e
    /// o segundo nunca seria alcancavel.
    #[test]
    fn catalog_ids_are_unique() {
        let mut ids: Vec<&str> = catalog::definitions().iter().map(|e| e.id).collect();
        let total = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), total, "id repetido no catalogo");
    }

    /// A deteccao mede diretorio de config package. O teste injeta os
    /// prefixos para NAO depender do que esta instalado em quem roda.
    #[test]
    fn detection_matches_package_directories_and_versioned_ones() {
        let raiz = std::env::temp_dir().join(format!("kinein-lib-{}", std::process::id()));
        drop(std::fs::remove_dir_all(&raiz));
        std::fs::create_dir_all(raiz.join("fmt")).unwrap();
        std::fs::create_dir_all(raiz.join("Catch2-3.16.0")).unwrap();
        std::fs::create_dir_all(raiz.join("spdlogextra")).unwrap();

        let prefixos = || std::iter::once(raiz.as_path());
        assert_eq!(
            availability::detect_in("fmt", prefixos()),
            availability::Availability::Detected
        );
        // Diretorio com versao no nome tambem conta: `<Pacote>-<versao>`.
        assert_eq!(
            availability::detect_in("Catch2", prefixos()),
            availability::Availability::Detected
        );
        // Prefixo comum NAO conta: `spdlogextra` nao e' `spdlog`.
        assert_eq!(
            availability::detect_in("spdlog", prefixos()),
            availability::Availability::NotDetected
        );
        assert_eq!(
            availability::detect_in("naoexiste", prefixos()),
            availability::Availability::NotDetected
        );
        drop(std::fs::remove_dir_all(&raiz));
    }

    /// O plano nunca sai sem o passo que LINKA — achar a biblioteca e nao
    /// linka-la deixaria o projeto compilando e falhando no linker.
    #[test]
    fn a_plan_always_ends_by_linking_the_target() {
        let plano = plan("fmt", "meu_app").unwrap();
        assert_eq!(plano.target, "meu_app");
        assert!(
            plano
                .steps
                .last()
                .is_some_and(|s| s.action_id == "cmake.addTargetLinkLibraries"),
            "o ultimo passo tem que ser o link: {:?}",
            plano.steps
        );
        assert!(
            plano.steps.len() >= 2,
            "faltou o passo de obter a biblioteca"
        );
        assert_eq!(plano.pinned_version, "12.2.0");
    }

    /// Quando nada foi encontrado, o plano diz ONDE se olhou — "nao achei" sem
    /// dizer onde manda o usuario adivinhar.
    #[test]
    fn a_plan_that_found_nothing_says_where_it_looked() {
        let plano = plan("benchmark", "app").unwrap();
        if plano.detected {
            assert!(
                plano.searched_paths.is_empty(),
                "achou e ainda listou caminho"
            );
            assert!(plano.uses_find_package, "achou e nao usou find_package");
        } else {
            assert!(
                !plano.searched_paths.is_empty(),
                "nao achou e nao disse onde procurou"
            );
            assert!(!plano.uses_find_package, "nao achou e usou find_package");
        }
    }

    #[test]
    fn unknown_library_and_empty_target_are_refused() {
        assert!(plan("nao-existe", "app").is_err());
        assert!(plan("fmt", "  ").is_err(), "alvo em branco passou");
    }

    #[test]
    fn list_crosses_the_catalog_with_this_machine() {
        let libs = list();
        assert_eq!(libs.len(), catalog::definitions().len());
        assert!(
            libs.iter().all(|l| matches!(
                l.status,
                LibraryStatus::Detected | LibraryStatus::NotDetected
            )),
            "status fora do dominio"
        );
        // Nao afirmamos QUAL esta instalada: depende da maquina que roda.
        assert!(libs.iter().any(|l| l.id == "fmt"));
    }

    #[test]
    fn search_paths_are_absolute() {
        for caminho in availability::search_paths() {
            assert!(
                Path::new(&caminho).is_absolute(),
                "{caminho:?} nao e absoluto"
            );
        }
    }
}
