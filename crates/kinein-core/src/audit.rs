//! Auditoria de seguranca das dependencias Rust (L2 fatia 5).
//!
//! Orquestra o `cargo-audit` e normaliza os achados no contrato de
//! diagnostico que a aba Problemas ja entende. Nada aqui reimplementa
//! analise de advisory: a base e a do `RustSec`, e a ferramenta e madura.
//!
//! **A rede e opt-in.** Ver `crate::audit::run_audit`.

use std::{
    error::Error,
    fmt,
    path::Path,
    process::Command,
    sync::{Arc, atomic::AtomicBool},
};

use kinein_protocol::{AuditDatabaseInfo, BuildDiagnostic, BuildDiagnosticSeverity};

use crate::process::{self, ProcessError};

/// Resultado normalizado de uma auditoria.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditReport {
    /// Estado da base de advisories usada.
    pub database: AuditDatabaseInfo,
    /// Achados, ja no formato do funil de diagnostico.
    pub findings: Vec<BuildDiagnostic>,
    /// Quantos achados sao vulnerabilidades (o resto sao avisos).
    pub vulnerabilities: u64,
}

/// Erro ao auditar.
#[derive(Debug)]
pub enum AuditError {
    /// A ferramenta nao pode ser iniciada (provavelmente ausente).
    Spawn {
        /// Comando que falhou.
        command: String,
        /// Erro de IO subjacente.
        source: std::io::Error,
    },
    /// Falha de IO durante a execucao.
    Io(std::io::Error),
    /// A base de advisories nao esta disponivel localmente e a rede nao foi
    /// autorizada. E o caso mais comum de primeira execucao.
    SemBaseLocal,
    /// O `cargo` existe mas o SUBCOMANDO nao esta instalado.
    ///
    /// Caso proprio porque o erro de spawn nao pega: quem falha nao e o
    /// `cargo`, e o despacho dele. Sem isto o usuario recebia "nao devolveu
    /// relatorio JSON: error: no such command", que descreve o sintoma e
    /// esconde o gesto — e este e o caso mais comum de PRIMEIRA execucao,
    /// justamente quando a mensagem mais importa.
    SubcomandoAusente {
        /// Nome da ferramenta a instalar.
        ferramenta: String,
    },
    /// A ferramenta rodou mas nao produziu o relatorio JSON esperado.
    SemRelatorio {
        /// Trecho do que ela imprimiu, para o erro ser investigavel.
        detail: String,
    },
}

impl fmt::Display for AuditError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spawn { command, source } => {
                write!(formatter, "falha ao iniciar '{command}': {source}")
            }
            Self::Io(error) => write!(formatter, "falha de IO durante a auditoria: {error}"),
            // A mensagem DIZ O QUE FAZER. Uma auditoria que falha sem explicar
            // como habilita-la vira uma funcionalidade que ninguem usa duas vezes.
            Self::SemBaseLocal => write!(
                formatter,
                "a base de advisories do RustSec nao esta neste computador e a \
                 atualizacao pela rede nao foi autorizada. Autorize em \
                 integration.config (id 'cargo-audit', chave 'allowNetwork', \
                 valor 'true') ou baixe a base uma vez com: cargo audit"
            ),
            Self::SubcomandoAusente { ferramenta } => write!(
                formatter,
                "{ferramenta} nao esta instalado. Instale com: cargo install {ferramenta}"
            ),
            Self::SemRelatorio { detail } => {
                write!(
                    formatter,
                    "cargo-audit nao devolveu relatorio JSON: {detail}"
                )
            }
        }
    }
}

impl Error for AuditError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Spawn { source, .. } | Self::Io(source) => Some(source),
            Self::SemBaseLocal | Self::SemRelatorio { .. } | Self::SubcomandoAusente { .. } => None,
        }
    }
}

/// Audita as dependencias do workspace.
///
/// `rede_autorizada` e a decisao do autor de 2026-08-21 materializada: quando
/// `false` — o padrao — a chamada recebe `--no-fetch` e `--stale`, entao o
/// `cargo-audit` usa a copia local da base e **nao abre conexao**. Quando
/// `true`, a base pode ser atualizada.
///
/// A flag NAO e cosmetica: e o unico ponto do core em que uma decisao de
/// produto ("a IDE nao fala com a internet sozinha") vira comportamento. Se
/// alguem remover o `--no-fetch`, a IDE passa a buscar rede sem consentimento
/// — e e exatamente isso que a mutacao do teste desta fatia reprova.
pub fn run_audit(
    root: &Path,
    rede_autorizada: bool,
    cancel: &Arc<AtomicBool>,
    on_output: &mut dyn FnMut(&str),
) -> Result<AuditReport, AuditError> {
    let mut command = Command::new("cargo");
    command.args(audit_args(rede_autorizada)).current_dir(root);

    let mut saida = String::new();
    let mut erro = String::new();
    let mut sink = |stream: &'static str, linha: String| {
        if stream == "stdout" {
            saida.push_str(&linha);
            saida.push('\n');
        } else {
            erro.push_str(&linha);
            erro.push('\n');
            on_output(&linha);
        }
    };
    // O codigo de saida do cargo-audit e NAO-ZERO quando ha achado. Isso nao e
    // falha da auditoria — e o resultado dela. Quem decide severidade e a UI,
    // pelo diagnostico; mesma regra do --error-exitcode=0 do Cppcheck.
    process::stream_command_lines_cancelable(command, cancel, &mut sink).map_err(|error| {
        match error {
            ProcessError::Spawn(source) => AuditError::Spawn {
                command: "cargo audit".to_owned(),
                source,
            },
            ProcessError::Wait(source) => AuditError::Io(source),
        }
    })?;

    if let Some(report) = parse_audit_json(&saida, !rede_autorizada) {
        return Ok(report);
    }
    if subcomando_ausente(&erro) {
        return Err(AuditError::SubcomandoAusente {
            ferramenta: "cargo-audit".to_owned(),
        });
    }
    if !rede_autorizada && erro.to_lowercase().contains("advisory") {
        return Err(AuditError::SemBaseLocal);
    }
    Err(AuditError::SemRelatorio {
        detail: erro.lines().next().unwrap_or("sem saida").to_owned(),
    })
}

/// Roda o `cargo-deny` nos checks LOCAIS e normaliza os achados.
///
/// Segundo braco da auditoria, e ele responde outra pergunta. O `cargo-audit`
/// diz "alguma dependencia tem vulnerabilidade conhecida?"; o `cargo-deny` diz
/// "as dependencias obedecem a POLITICA deste projeto?" — licenca aceita,
/// crate banido, fonte permitida. Sao perguntas de supply chain diferentes, e
/// uma nao substitui a outra.
///
/// **So roda se o projeto tiver `deny.toml`**, pela mesma razao do
/// `.clang-tidy` na fatia 4: sem config, o cargo-deny aplica os PROPRIOS
/// padroes de licenca, e impor politica de licenca a quem nunca declarou uma
/// e a mesma invasao que a IDE recusa em `--coverage` e `-Werror`. Politica de
/// dependencia e decisao de projeto, nao de ferramenta.
///
/// `advisories` fica de FORA do conjunto: o `cargo-audit` ja cobre, e o
/// cargo-deny precisaria da mesma base pela rede — o que reabriria, por uma
/// porta lateral, a decisao de rede que a fatia 5 fechou.
pub fn run_deny(
    root: &Path,
    cancel: &Arc<AtomicBool>,
    on_output: &mut dyn FnMut(&str),
) -> Result<Vec<BuildDiagnostic>, AuditError> {
    if !projeto_declara_politica(root) {
        on_output(
            "cargo-deny pulado: o projeto nao tem deny.toml. A politica de \
             licenca e de crates e declarada pelo projeto, nao imposta pela IDE.",
        );
        return Ok(Vec::new());
    }

    let mut command = Command::new("cargo");
    command.args(deny_args()).current_dir(root);

    // O cargo-deny escreve o JSON no STDERR: medido na versao 0.20.2.
    let mut saida = String::new();
    let mut erro = String::new();
    let mut sink = |stream: &'static str, linha: String| {
        if stream == "stderr" && linha.trim_start().starts_with('{') {
            saida.push_str(&linha);
            saida.push('\n');
        } else {
            erro.push_str(&linha);
            erro.push('\n');
            on_output(&linha);
        }
    };
    // Codigo de saida nao-zero aqui significa "achou algo", nao "falhou".
    // Mesma regra do resto do funil: severidade e da UI, pelo diagnostico.
    process::stream_command_lines_cancelable(command, cancel, &mut sink).map_err(|error| {
        match error {
            ProcessError::Spawn(source) => AuditError::Spawn {
                command: "cargo deny".to_owned(),
                source,
            },
            ProcessError::Wait(source) => AuditError::Io(source),
        }
    })?;

    let achados = parse_deny_json(&saida);
    if achados.is_empty() && subcomando_ausente(&erro) {
        return Err(AuditError::SubcomandoAusente {
            ferramenta: "cargo-deny".to_owned(),
        });
    }
    Ok(achados)
}

/// O projeto declara a propria politica de dependencias?
///
/// Funcao propria porque isto e um PORTAO, e portao se testa direto. Sem
/// `deny.toml` o cargo-deny aplica os PROPRIOS padroes de licenca, e impor
/// politica a quem nunca declarou uma e a mesma invasao que a IDE recusa em
/// `--coverage` e `-Werror`. Politica de dependencia e decisao de projeto.
///
/// Extraida depois de a mutacao "rodar mesmo sem deny.toml" SOBREVIVER: nada
/// exercitava a guarda, so o seu efeito. Segunda vez nesta familia de fatias
/// que uma decisao de produto embutida num `if` escapou das provas.
#[must_use]
pub fn projeto_declara_politica(root: &Path) -> bool {
    root.join("deny.toml").is_file()
}

/// Reconhece "o cargo existe mas o subcomando nao".
///
/// Texto do proprio cargo (`error: no such command`). Casar por texto nao e
/// elegante, mas a alternativa seria checar o PATH por conta propria — o que
/// duplicaria a resolucao do cargo e erraria em toolchain com override.
fn subcomando_ausente(erro: &str) -> bool {
    erro.contains("no such command")
}

/// Argumentos do `cargo deny`, com o conjunto LOCAL de checks.
#[must_use]
pub fn deny_args() -> Vec<String> {
    vec![
        "deny".to_owned(),
        "--format".to_owned(),
        "json".to_owned(),
        "check".to_owned(),
        // `advisories` NAO entra: e do cargo-audit, e exigiria rede.
        "licenses".to_owned(),
        "bans".to_owned(),
        "sources".to_owned(),
    ]
}

/// Parse do JSON por linha do `cargo deny --format json` (medido em 0.20.2).
///
/// Forma real: um objeto por linha, `{"type":"diagnostic","fields":{...}}`,
/// com `severity`, `message`, `code` e `graphs[].Krate.name/version`. NAO ha
/// campo de arquivo — os `labels` apontam linha e coluna sem dizer de que
/// arquivo —, entao o achado e ancorado no `Cargo.lock`, que e onde a
/// dependencia de fato esta declarada.
#[must_use]
pub fn parse_deny_json(raw: &str) -> Vec<BuildDiagnostic> {
    let mut findings = Vec::new();
    for linha in raw.lines() {
        let Ok(valor) = serde_json::from_str::<serde_json::Value>(linha.trim()) else {
            continue;
        };
        if valor.get("type").and_then(serde_json::Value::as_str) != Some("diagnostic") {
            continue;
        }
        let Some(fields) = valor.get("fields") else {
            continue;
        };
        let severidade = match fields.get("severity").and_then(serde_json::Value::as_str) {
            // `bug` e problema do proprio cargo-deny, mas o usuario precisa
            // ver: silencia-lo esconderia uma auditoria que nao completou.
            Some("error" | "bug") => BuildDiagnosticSeverity::Error,
            Some("warning") => BuildDiagnosticSeverity::Warning,
            // note/help elaboram o achado anterior; virariam ruido duplicado.
            _ => continue,
        };
        let mensagem = fields
            .get("message")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("achado do cargo-deny");
        let codigo = fields
            .get("code")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("deny");

        let mut texto = format!("{codigo}: {mensagem}");
        if let Some(krate) = fields
            .get("graphs")
            .and_then(serde_json::Value::as_array)
            .and_then(|graphs| graphs.first())
            .and_then(|graph| graph.get("Krate"))
        {
            let nome = krate.get("name").and_then(serde_json::Value::as_str);
            let versao = krate.get("version").and_then(serde_json::Value::as_str);
            if let Some(nome) = nome {
                texto.push_str(" (");
                texto.push_str(nome);
                if let Some(versao) = versao {
                    texto.push(' ');
                    texto.push_str(versao);
                }
                texto.push(')');
            }
        }

        findings.push(BuildDiagnostic {
            severity: severidade,
            message: texto,
            file: Some("Cargo.lock".to_owned()),
            line: None,
            column: None,
        });
    }
    findings
}

/// Argumentos do `cargo audit`, e o unico lugar onde a rede e decidida.
///
/// Funcao propria, e nao um `if` dentro do `run_audit`, porque isto e o ponto
/// em que uma decisao de PRODUTO vira comportamento — e ponto assim tem que
/// ser testavel diretamente, nao so por efeito. Medido no cargo-audit 0.22.2:
/// `--no-fetch` nao faz o git fetch da base, `--stale` aceita base velha.
///
/// Sem autorizacao NAO existe caminho que abra conexao: `--no-fetch` sempre
/// entra. A mutacao "tirar o --no-fetch" reprova o teste abaixo, e e por isso
/// que ele existe.
#[must_use]
pub fn audit_args(rede_autorizada: bool) -> Vec<String> {
    let mut args = vec!["audit".to_owned(), "--json".to_owned()];
    if !rede_autorizada {
        args.push("--no-fetch".to_owned());
        // Sem `--stale` o cargo-audit RECUSA base velha, e como nao pode
        // buscar, a auditoria offline falharia sempre que a copia local
        // tivesse alguns dias. Os dois andam juntos.
        args.push("--stale".to_owned());
    }
    args
}

/// Parse do relatorio do `cargo audit --json` (medido na versao 0.22.2).
///
/// Forma real: `database{advisory-count,last-updated}`,
/// `vulnerabilities{count,list[]}` e `warnings{<tipo>: []}`. Cada entrada tem
/// `package{name,version}` e `advisory{id,title}`.
#[must_use]
pub fn parse_audit_json(raw: &str, offline: bool) -> Option<AuditReport> {
    let value: serde_json::Value = serde_json::from_str(raw.trim()).ok()?;
    let database = value.get("database")?;

    let mut findings = Vec::new();
    let vulnerabilidades = value
        .get("vulnerabilities")
        .and_then(|v| v.get("list"))
        .and_then(serde_json::Value::as_array)
        .map_or(0, |lista| {
            for entrada in lista {
                if let Some(achado) = achado(entrada, BuildDiagnosticSeverity::Error, "vulneravel")
                {
                    findings.push(achado);
                }
            }
            lista.len() as u64
        });

    // `warnings` e um mapa TIPO -> lista (unmaintained, yanked, unsound...).
    // Percorrer o mapa em vez de listar os tipos conhecidos: tipo novo na
    // ferramenta aparece sozinho, em vez de sumir calado.
    if let Some(mapa) = value.get("warnings").and_then(serde_json::Value::as_object) {
        for (tipo, lista) in mapa {
            let Some(lista) = lista.as_array() else {
                continue;
            };
            for entrada in lista {
                if let Some(achado) = achado(entrada, BuildDiagnosticSeverity::Warning, tipo) {
                    findings.push(achado);
                }
            }
        }
    }

    Some(AuditReport {
        database: AuditDatabaseInfo {
            advisory_count: database
                .get("advisory-count")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0),
            last_updated: database
                .get("last-updated")
                .and_then(serde_json::Value::as_str)
                .map(ToOwned::to_owned),
            offline,
        },
        findings,
        vulnerabilities: vulnerabilidades,
    })
}

/// Converte uma entrada do relatorio num diagnostico do funil.
fn achado(
    entrada: &serde_json::Value,
    severity: BuildDiagnosticSeverity,
    tipo: &str,
) -> Option<BuildDiagnostic> {
    let package = entrada.get("package")?;
    let nome = package.get("name")?.as_str()?;
    let versao = package.get("version").and_then(serde_json::Value::as_str);
    let advisory = entrada.get("advisory");
    let id = advisory
        .and_then(|a| a.get("id"))
        .and_then(serde_json::Value::as_str);
    let titulo = advisory
        .and_then(|a| a.get("title"))
        .and_then(serde_json::Value::as_str);

    let mut message = String::new();
    if let Some(id) = id {
        message.push_str(id);
        message.push_str(": ");
    }
    message.push_str(titulo.unwrap_or(tipo));
    message.push_str(" (");
    message.push_str(nome);
    if let Some(versao) = versao {
        message.push(' ');
        message.push_str(versao);
    }
    message.push(')');

    Some(BuildDiagnostic {
        severity,
        message,
        // O advisory fala de um PACOTE, nao de uma linha de codigo nossa. O
        // Cargo.lock e o arquivo onde a dependencia realmente esta declarada,
        // entao e para la que o clique da aba Problemas leva.
        file: Some("Cargo.lock".to_owned()),
        line: None,
        column: None,
    })
}

/// Linha do `Cargo.lock` onde o pacote e declarado, para o clique abrir no
/// lugar certo em vez do topo do arquivo.
///
/// Busca textual de proposito: o formato do lockfile e estavel e um parser de
/// TOML aqui so acrescentaria dependencia para achar um numero de linha. Se
/// nao achar, devolve `None` — e a UI abre o arquivo sem posicionar, que e
/// melhor que posicionar errado.
#[must_use]
pub fn linha_do_pacote(lockfile: &str, pacote: &str) -> Option<u64> {
    let alvo = format!("name = \"{pacote}\"");
    lockfile
        .lines()
        .position(|linha| linha.trim() == alvo)
        .map(|indice| indice as u64 + 1)
}

#[cfg(test)]
mod tests {
    use super::{linha_do_pacote, parse_audit_json};
    use kinein_protocol::BuildDiagnosticSeverity;

    // Recorte FIEL do `cargo audit --json` 0.22.2, medido em 2026-08-21 no
    // proprio repositorio (o `serial 0.4.0` e um achado real deste projeto).
    const RELATORIO: &str = r#"{
      "database": {"advisory-count": 1225, "last-updated": "2026-08-21T08:28:40+02:00"},
      "vulnerabilities": {"found": true, "count": 1, "list": [
        {"package": {"name": "vulneravel", "version": "1.0.0"},
         "advisory": {"id": "RUSTSEC-2026-0001", "title": "estouro de buffer"}}
      ]},
      "warnings": {"unmaintained": [
        {"kind": "unmaintained",
         "package": {"name": "serial", "version": "0.4.0"},
         "advisory": {"id": "RUSTSEC-2017-0008", "title": "`serial` crate is unmaintained"}}
      ]}
    }"#;

    #[test]
    fn parse_separa_vulnerabilidade_de_aviso_e_aponta_o_lockfile() {
        let report = parse_audit_json(RELATORIO, true).unwrap();

        assert_eq!(report.database.advisory_count, 1225);
        assert!(report.database.offline, "offline vem de quem chamou");
        assert_eq!(report.vulnerabilities, 1);
        assert_eq!(report.findings.len(), 2, "1 vulnerabilidade + 1 aviso");

        let vuln = report
            .findings
            .iter()
            .find(|f| f.severity == BuildDiagnosticSeverity::Error)
            .expect("vulnerabilidade vira ERRO");
        assert!(vuln.message.contains("RUSTSEC-2026-0001"));
        assert!(vuln.message.contains("vulneravel 1.0.0"));
        assert_eq!(vuln.file.as_deref(), Some("Cargo.lock"));

        let aviso = report
            .findings
            .iter()
            .find(|f| f.severity == BuildDiagnosticSeverity::Warning)
            .expect("unmaintained vira AVISO, nao erro");
        assert!(aviso.message.contains("RUSTSEC-2017-0008"));
        assert!(aviso.message.contains("serial 0.4.0"));
    }

    #[test]
    fn sem_autorizacao_nao_existe_caminho_que_abra_rede() {
        // O contrato desta fatia, e ele e sobre PRODUTO, nao sobre estilo:
        // decisao do autor em 2026-08-21 — a IDE nao fala com a internet sem
        // o usuario ligar. O padrao (false) TEM que carregar --no-fetch.
        let offline = super::audit_args(false);
        assert!(
            offline.contains(&"--no-fetch".to_owned()),
            "sem opt-in, a base NAO pode ser buscada pela rede"
        );
        assert!(
            offline.contains(&"--stale".to_owned()),
            "offline sem --stale recusaria a base local por estar velha"
        );

        // Com autorizacao explicita, a base pode ser atualizada.
        let online = super::audit_args(true);
        assert!(!online.contains(&"--no-fetch".to_owned()));
        assert!(!online.contains(&"--stale".to_owned()));

        // Nos dois casos o relatorio e JSON: o parser nao le texto humano.
        for args in [&offline, &online] {
            assert_eq!(args[0], "audit");
            assert!(args.contains(&"--json".to_owned()));
        }
    }

    // Recorte FIEL do `cargo deny --format json` 0.20.2, medido em 2026-08-21
    // no PROPRIO repositorio (o `inotify`/ISC e um achado real deste projeto).
    const DENY: &str = r#"{"fields":{"code":"rejected","graphs":[{"Krate":{"name":"inotify","version":"0.11.4"}}],"message":"failed to satisfy license requirements","severity":"error"},"type":"diagnostic"}
{"fields":{"code":"duplicate","graphs":[{"Krate":{"name":"bitflags","version":"1.3.2"}}],"message":"found 2 duplicate entries for crate 'bitflags'","severity":"warning"},"type":"diagnostic"}
{"fields":{"code":"unresolved-workspace-dependency","graphs":[{"Krate":{"name":"kinein-cli","version":"0.1.0"}}],"message":"failed to resolve a workspace dependency","severity":"bug"},"type":"diagnostic"}
{"fields":{"message":"elaboracao do achado acima","severity":"note"},"type":"diagnostic"}
{"type":"summary","fields":{"licenses":{"errors":1}}}"#;

    #[test]
    fn parse_deny_separa_severidade_e_ancora_no_lockfile() {
        let achados = super::parse_deny_json(DENY);

        assert_eq!(
            achados.len(),
            3,
            "erro, aviso e bug entram; `note` e elaboracao e viraria ruido              duplicado; a linha `summary` nao e diagnostico"
        );

        assert_eq!(achados[0].severity, BuildDiagnosticSeverity::Error);
        assert!(achados[0].message.contains("rejected"));
        assert!(
            achados[0].message.contains("inotify 0.11.4"),
            "sem o nome do crate o achado nao e acionavel: {}",
            achados[0].message
        );
        assert_eq!(achados[0].file.as_deref(), Some("Cargo.lock"));

        assert_eq!(achados[1].severity, BuildDiagnosticSeverity::Warning);
        assert!(achados[1].message.contains("bitflags"));

        // `bug` e problema do proprio cargo-deny, mas o usuario precisa ver:
        // silencia-lo esconderia uma auditoria que nao completou.
        assert_eq!(achados[2].severity, BuildDiagnosticSeverity::Error);
        assert!(
            achados[2]
                .message
                .contains("unresolved-workspace-dependency")
        );
    }

    #[test]
    fn sem_deny_toml_a_politica_do_projeto_nao_e_inventada() {
        // O portao da fatia 7: a IDE nao impoe politica de licenca a quem
        // nunca declarou uma. Sem `deny.toml`, o cargo-deny nem roda.
        let base = std::env::temp_dir()
            .join("kinein-core-tests")
            .join(format!("{}-deny-guard", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();

        assert!(
            !super::projeto_declara_politica(&base),
            "projeto sem deny.toml NAO tem politica declarada"
        );

        std::fs::write(base.join("deny.toml"), "[licenses]\n").unwrap();
        assert!(
            super::projeto_declara_politica(&base),
            "com deny.toml, quem manda e o projeto"
        );

        // Diretorio com o nome nao conta: tem que ser arquivo.
        let outro = base.join("outro");
        std::fs::create_dir_all(outro.join("deny.toml")).unwrap();
        assert!(!super::projeto_declara_politica(&outro));

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn parse_deny_ignora_lixo_sem_inventar_achado() {
        assert!(super::parse_deny_json("").is_empty());
        assert!(
            super::parse_deny_json(
                "nao e json
outra linha"
            )
            .is_empty()
        );
        // JSON valido que nao e diagnostico nao vira achado.
        assert!(super::parse_deny_json(r#"{"type":"summary","fields":{}}"#).is_empty());
    }

    #[test]
    fn deny_args_ficam_nos_checks_locais() {
        let args = super::deny_args();
        assert_eq!(args[0], "deny");
        assert!(args.contains(&"json".to_owned()));
        for local in ["licenses", "bans", "sources"] {
            assert!(args.contains(&local.to_owned()), "falta o check {local}");
        }
        assert!(
            !args.contains(&"advisories".to_owned()),
            "advisories e do cargo-audit e exigiria REDE — incluir aqui \
             reabriria por porta lateral a decisao que a fatia 5 fechou"
        );
    }

    #[test]
    fn parse_recusa_saida_que_nao_e_relatorio() {
        // Sem isto, uma saida de erro viraria "auditoria limpa" — o modo de
        // falha mais perigoso possivel numa ferramenta de seguranca.
        assert!(parse_audit_json("{}", true).is_none());
        assert!(parse_audit_json("nao e json", true).is_none());
        assert!(parse_audit_json("", true).is_none());
    }

    #[test]
    fn tipo_de_aviso_novo_aparece_sozinho() {
        // O mapa de warnings e percorrido, nao listado: um tipo que a
        // ferramenta passe a emitir nao pode sumir calado.
        let raw = r#"{"database": {"advisory-count": 1},
          "warnings": {"tipo-que-ainda-nao-existe": [
            {"package": {"name": "x", "version": "1.0.0"},
             "advisory": {"id": "RUSTSEC-9999-0001", "title": "coisa nova"}}]}}"#;
        let report = parse_audit_json(raw, false).unwrap();
        assert_eq!(report.findings.len(), 1);
        assert!(report.findings[0].message.contains("coisa nova"));
        assert!(!report.database.offline);
    }

    #[test]
    fn linha_do_pacote_acha_a_declaracao_e_nao_chuta() {
        let lock = "# comentario\n\n[[package]]\nname = \"serial\"\nversion = \"0.4.0\"\n\
                    \n[[package]]\nname = \"outro\"\nversion = \"1.0.0\"\n";
        assert_eq!(linha_do_pacote(lock, "serial"), Some(4));
        assert_eq!(linha_do_pacote(lock, "outro"), Some(8));
        // Nao achou: None. Posicionar errado e pior que nao posicionar.
        assert_eq!(linha_do_pacote(lock, "inexistente"), None);
        // Nao casa por prefixo: "seria" nao e "serial".
        assert_eq!(linha_do_pacote(lock, "seria"), None);
    }
}
