//! Os volumes de um container, para o "Remover" que pergunta (16b.3,
//! `roadmaps/59` §2.5.2; decisao do autor: "Remover com os volumes (N, X MB)"
//! ou "So' o container").
//!
//! Duas perguntas curtas, com o prazo do `probe`: o `inspect` diz quais
//! volumes o container monta, e a tabela do `system df -v` diz o tamanho de
//! cada um. Sem a tabela (o motor nao a da', ou estourou o prazo), os
//! volumes vem sem tamanho e a tela mostra so' a contagem.

use kinein_protocol::ContainerVolume;

use super::{Engine, parse, probe};

/// Os volumes que `id` monta.
///
/// O motor ja' respondeu ao `ps` (quem chama
/// escolheu pelo `answering`): uma falha aqui e' do container — nao existe,
/// ou o `inspect` estourou o prazo —, e o `Err` traz o que o motor disse.
///
/// # Errors
/// O motivo do `inspect` que falhou.
pub fn volumes(engine: &Engine, id: &str) -> Result<Vec<ContainerVolume>, String> {
    let mounts = probe::ask(engine, &["inspect", "--format", "{{json .Mounts}}", id]);
    if !mounts.ok {
        return Err(mounts.reason());
    }
    let names = parse::mounted_volumes(&mounts.stdout);
    if names.is_empty() {
        return Ok(Vec::new());
    }
    let table = probe::ask(engine, &["system", "df", "-v"]);
    let sizes = if table.ok {
        parse::volume_sizes(&table.stdout)
    } else {
        std::collections::HashMap::new()
    };
    Ok(names
        .into_iter()
        .map(|name| ContainerVolume {
            anonymous: parse::is_anonymous_volume(&name),
            size_bytes: sizes.get(&name).copied(),
            name,
        })
        .collect())
}
