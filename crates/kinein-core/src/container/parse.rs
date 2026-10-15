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
        .filter_map(|line| serde_json::from_str::<Value>(line.trim()).ok())
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
pub fn containers(output: &str) -> Vec<ContainerInfo> {
    objetos(output)
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
pub fn images(output: &str) -> Vec<ImageInfo> {
    objetos(output)
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
    let name = ["Names", "RepoTags"]
        .iter()
        .filter_map(|chave| objeto.get(*chave))
        .filter_map(|v| v.as_array())
        .filter_map(|lista| lista.first())
        .find_map(Value::as_str)
        .unwrap_or("");
    if name.is_empty() {
        return ("<none>".to_owned(), "<none>".to_owned());
    }
    let inicio_do_nome = name.rfind('/').map_or(0, |i| i + 1);
    name[inicio_do_nome..].rfind(':').map_or_else(
        || (name.to_owned(), "latest".to_owned()),
        |i| {
            (
                name[..inicio_do_nome + i].to_owned(),
                name[inicio_do_nome + i + 1..].to_owned(),
            )
        },
    )
}

/// Remove sequencias de escape ANSI — o mesmo remedio do `probe.rs`.
#[must_use]
pub fn strip_ansi(texto: &str) -> String {
    let mut output = String::with_capacity(texto.len());
    let mut chars = texto.chars();
    while let Some(c) = chars.next() {
        if c != '\u{1b}' {
            output.push(c);
            continue;
        }
        for seguinte in chars.by_ref() {
            if seguinte.is_ascii_alphabetic() {
                break;
            }
        }
    }
    output
}

/// O nome e a versao do servidor em `version --format '{{json .Server}}'`.
///
/// O Docker Engine diz `Platform.Name` "Docker Engine - Community" e o
/// componente "Engine"; o Podman servindo a API do Docker diz "Podman Engine"
/// nos dois. Vale o primeiro componente que se chama "Podman Engine", senao o
/// `Platform.Name`, senao o nome do primeiro componente.
#[must_use]
pub fn server_identity(output: &str) -> (Option<String>, Option<String>) {
    let Ok(server) = serde_json::from_str::<Value>(output.trim()) else {
        return (None, None);
    };
    let components = server
        .get("Components")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    let name_of = |v: &Value| {
        v.get("Name")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|n| !n.is_empty())
            .map(ToOwned::to_owned)
    };
    let name = components
        .iter()
        .filter_map(name_of)
        .find(|n| n.contains("Podman"))
        .or_else(|| server.get("Platform").and_then(name_of))
        .or_else(|| components.first().and_then(name_of));
    let version = server
        .get("Version")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(ToOwned::to_owned);
    (name, version)
}

/// A versao na linha do `--version`: "Docker version 29.9.1, build dfc520d"
/// da' `29.9.1`; "podman version 5.7.0" da' `5.7.0`.
#[must_use]
pub fn cli_version(line: &str) -> Option<String> {
    let mut words = line.split_whitespace();
    words.find(|p| p.eq_ignore_ascii_case("version"))?;
    words
        .next()
        .map(|v| v.trim_end_matches(','))
        .filter(|v| !v.is_empty())
        .map(ToOwned::to_owned)
}

/// Os volumes montados num container, do `inspect --format '{{json
/// .Mounts}}'`: so' os do tipo `volume` (bind e tmpfs nao sao do motor).
#[must_use]
pub fn mounted_volumes(output: &str) -> Vec<String> {
    let Ok(Value::Array(mounts)) = serde_json::from_str::<Value>(output.trim()) else {
        return Vec::new();
    };
    mounts
        .iter()
        .filter(|mount| mount.get("Type").and_then(Value::as_str) == Some("volume"))
        .filter_map(|mount| mount.get("Name").and_then(Value::as_str))
        .filter(|name| !name.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

/// Anonimo e' o volume que o motor batizou: 64 hexadecimais. Vale para os
/// dois motores; o `"Anonymous": true` do `volume inspect` e' so' do Podman.
#[must_use]
pub fn is_anonymous_volume(name: &str) -> bool {
    name.len() == 64 && name.bytes().all(|b| b.is_ascii_hexdigit())
}

/// O tamanho de cada volume na tabela de `system df -v`.
///
/// A secao "Local Volumes space usage:", linhas `<nome> <links> <tamanho>`. O JSON verboso
/// nao existe no Podman 5.7 ("cannot combine --format and --verbose",
/// medido em 2026-10-15); a tabela e' a mesma do Docker.
#[must_use]
pub fn volume_sizes(output: &str) -> std::collections::HashMap<String, u64> {
    let mut sizes = std::collections::HashMap::new();
    let mut in_volumes = false;
    for line in output.lines().map(str::trim) {
        if line.contains("usage:") {
            in_volumes = line.starts_with("Local Volumes");
            continue;
        }
        if !in_volumes || line.starts_with("VOLUME NAME") {
            continue;
        }
        if let [name, _links, size] = line.split_whitespace().collect::<Vec<_>>().as_slice()
            && let Some(bytes) = human_size(size)
        {
            sizes.insert((*name).to_owned(), bytes);
        }
    }
    sizes
}

/// "40.08MB" da' 40.080.000: unidades DECIMAIS, como o `HumanSize` que o
/// Docker e o Podman usam nessa tabela. Inteiro, sem `f64`.
#[must_use]
pub fn human_size(text: &str) -> Option<u64> {
    let text = text.trim();
    let (number, unit) = text.split_at(text.find(|c: char| c.is_ascii_alphabetic())?);
    let multiplier: u64 = match unit {
        "B" => 1,
        "kB" | "KB" => 1_000,
        "MB" => 1_000_000,
        "GB" => 1_000_000_000,
        "TB" => 1_000_000_000_000,
        "PB" => 1_000_000_000_000_000,
        _ => return None,
    };
    let (whole, fraction) = number.split_once('.').unwrap_or((number, ""));
    let mut bytes = whole.parse::<u64>().ok()?.checked_mul(multiplier)?;
    if !fraction.is_empty() {
        let scale = 10_u64.checked_pow(u32::try_from(fraction.len()).ok()?)?;
        let part = fraction.parse::<u64>().ok()?.checked_mul(multiplier)? / scale;
        bytes = bytes.checked_add(part)?;
    }
    Some(bytes)
}
