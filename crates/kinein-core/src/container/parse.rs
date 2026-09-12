//! Interpreta o que `ps` e `images` respondem. Funcoes puras: e' o que se testa
//! sem motor, sem processo e sem container.
//!
//! # Por que o parser aceita DUAS formas
//!
//! O Podman 5.8.4 (medido em 2026-09-12) responde `ps --format json` com um
//! ARRAY de objetos e chaves `Id`, `Names` (lista), `State`, `Status`, `Ports`
//! (lista de objetos `host_port`/`container_port`/`protocol`), `Created` (RFC
//! 3339). O Docker responde `ps --format '{{json .}}'` com UM OBJETO POR LINHA e
//! chaves `ID`, `Names` (string), `State`, `Status`, `Ports` (string
//! `0.0.0.0:5432->5432/tcp`), `CreatedAt` — formato lido da documentacao do
//! Docker, NAO verificado contra um Docker Engine real (esta maquina so' tem o
//! shim `podman-docker`). Um parser rigido quebraria num dos dois dizendo
//! "nenhum container", que e' a pior mentira possivel aqui; por isso linha que
//! nao e' JSON e' ignorada, chave ausente vira vazio, e a saida CRUA volta
//! sempre.

use kinein_protocol::{ContainerInfo, ImageInfo};
use serde_json::Value;

/// Um array JSON, ou um objeto JSON por linha — os dois viram a mesma lista.
fn objetos(texto: &str) -> Vec<Value> {
    let aparado = texto.trim();
    if let Ok(Value::Array(itens)) = serde_json::from_str::<Value>(aparado) {
        return itens;
    }
    aparado
        .lines()
        .filter_map(|linha| serde_json::from_str::<Value>(linha.trim()).ok())
        .filter(Value::is_object)
        .collect()
}

/// Primeira chave presente, como texto. Numero vira texto; lista vira a
/// juncao por virgula; `null` e chave ausente viram vazio.
fn texto(objeto: &Value, chaves: &[&str]) -> String {
    chaves
        .iter()
        .find_map(|chave| objeto.get(*chave))
        .map_or_else(String::new, valor_como_texto)
}

fn valor_como_texto(valor: &Value) -> String {
    match valor {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Array(itens) => itens
            .iter()
            .map(valor_como_texto)
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(", "),
        Value::Null | Value::Object(_) => String::new(),
    }
}

/// `Names`: lista (Podman) ou string com virgulas (Docker), sem a barra
/// inicial que o Docker antigo punha.
fn nomes(objeto: &Value) -> Vec<String> {
    match objeto.get("Names") {
        Some(Value::Array(itens)) => itens
            .iter()
            .filter_map(Value::as_str)
            .map(|n| n.trim_start_matches('/').to_owned())
            .collect(),
        Some(Value::String(s)) => s
            .split(',')
            .map(|n| n.trim().trim_start_matches('/').to_owned())
            .filter(|n| !n.is_empty())
            .collect(),
        _ => Vec::new(),
    }
}

/// `Ports`: objetos (Podman) viram `host:port->port/proto`; string (Docker)
/// vira uma linha por virgula.
fn portas(objeto: &Value) -> Vec<String> {
    match objeto.get("Ports") {
        Some(Value::Array(itens)) => itens
            .iter()
            .filter_map(|p| {
                let container = p.get("container_port")?.as_u64()?;
                let host = p.get("host_port").and_then(Value::as_u64);
                let proto = p.get("protocol").and_then(Value::as_str).unwrap_or("tcp");
                let ip = p.get("host_ip").and_then(Value::as_str).unwrap_or("");
                Some(host.map_or_else(
                    || format!("{container}/{proto}"),
                    |h| {
                        let ip = if ip.is_empty() { "0.0.0.0" } else { ip };
                        format!("{ip}:{h}->{container}/{proto}")
                    },
                ))
            })
            .collect(),
        Some(Value::String(s)) => s
            .split(',')
            .map(str::trim)
            .filter(|p| !p.is_empty())
            .map(ToOwned::to_owned)
            .collect(),
        _ => Vec::new(),
    }
}

/// `State` quando o motor a da'; senao deduz de `Exited` (Podman em
/// `{{json .}}` traz `Exited: true` e `State` vazio em algumas versoes).
fn estado(objeto: &Value) -> String {
    let declarado = texto(objeto, &["State"]);
    if !declarado.is_empty() {
        return declarado;
    }
    match objeto.get("Exited").and_then(Value::as_bool) {
        Some(true) => "exited".to_owned(),
        Some(false) => "running".to_owned(),
        None => String::new(),
    }
}

/// Os containers de uma saida de `ps`.
#[must_use]
pub fn containers(saida: &str) -> Vec<ContainerInfo> {
    objetos(saida)
        .iter()
        .filter_map(|objeto| {
            let id = texto(objeto, &["Id", "ID"]);
            if id.is_empty() {
                return None;
            }
            Some(ContainerInfo {
                id,
                names: nomes(objeto),
                image: texto(objeto, &["Image"]),
                state: estado(objeto),
                status: texto(objeto, &["Status"]),
                ports: portas(objeto),
                created: {
                    let criado = texto(objeto, &["CreatedAt"]);
                    if criado.is_empty() {
                        texto(objeto, &["Created"])
                    } else {
                        criado
                    }
                },
            })
        })
        .collect()
}

/// As imagens de uma saida de `images`.
#[must_use]
pub fn images(saida: &str) -> Vec<ImageInfo> {
    objetos(saida)
        .iter()
        .filter_map(|objeto| {
            let id = texto(objeto, &["Id", "ID"]);
            if id.is_empty() {
                return None;
            }
            // Podman em `--format json` (array) NAO traz `repository`/`tag`:
            // traz `Names` ["repo:tag"] (medido em 2026-09-12). O `{{json .}}`
            // dele e o Docker trazem os dois campos. Sem nenhum, e' `<none>`.
            let (repository, tag) = {
                let repository = texto(objeto, &["repository", "Repository"]);
                let tag = texto(objeto, &["tag", "Tag"]);
                if repository.is_empty() {
                    repo_e_tag_do_nome(objeto)
                } else {
                    (repository, tag)
                }
            };
            Some(ImageInfo {
                id,
                repository,
                tag,
                size: texto(objeto, &["Size"]),
                created: {
                    let criado = texto(objeto, &["CreatedAt", "CreatedSince"]);
                    if criado.is_empty() {
                        texto(objeto, &["Created"])
                    } else {
                        criado
                    }
                },
            })
        })
        .collect()
}

/// `Names[0]` ou `RepoTags[0]` como `repo:tag`; a divisao e' no ULTIMO `:` que
/// vem depois da ultima `/`, porque `localhost:5000/x:1` tem dois-pontos no
/// registro. Sem nome, `<none>`/`<none>`, como o proprio motor imprime.
fn repo_e_tag_do_nome(objeto: &Value) -> (String, String) {
    let nome = ["Names", "RepoTags"]
        .iter()
        .filter_map(|chave| objeto.get(*chave))
        .filter_map(|v| v.as_array())
        .filter_map(|lista| lista.first())
        .find_map(Value::as_str)
        .unwrap_or("");
    if nome.is_empty() {
        return ("<none>".to_owned(), "<none>".to_owned());
    }
    let inicio_do_nome = nome.rfind('/').map_or(0, |i| i + 1);
    nome[inicio_do_nome..].rfind(':').map_or_else(
        || (nome.to_owned(), "latest".to_owned()),
        |i| {
            (
                nome[..inicio_do_nome + i].to_owned(),
                nome[inicio_do_nome + i + 1..].to_owned(),
            )
        },
    )
}

/// Remove sequencias de escape ANSI — o mesmo remedio do `probe.rs`.
#[must_use]
pub fn strip_ansi(texto: &str) -> String {
    let mut saida = String::with_capacity(texto.len());
    let mut chars = texto.chars();
    while let Some(c) = chars.next() {
        if c != '\u{1b}' {
            saida.push(c);
            continue;
        }
        for seguinte in chars.by_ref() {
            if seguinte.is_ascii_alphabetic() {
                break;
            }
        }
    }
    saida
}
