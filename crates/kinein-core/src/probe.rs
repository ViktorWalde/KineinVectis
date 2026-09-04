//! Sondas de debug conectadas: quem esta plugado nesta maquina.
//!
//! Nasceu em 2026-09-03 (etapa 24 do `roadmaps/35`), e e' o "plug" do plug and
//! play: a IDE DETECTA a sonda em vez de pedir configuracao.
//!
//! # Por que o parser e' TOLERANTE, e isso e' decisao e nao preguica
//!
//! O formato do `probe-rs list` foi levantado de fontes da comunidade, **nao**
//! da documentacao oficial nem do binario instalado — ele nao esta nesta
//! maquina (medido em 2026-09-03). Um parser rigido contra formato nao
//! verificado quebraria na primeira versao que mudasse o espacamento, e
//! quebraria dizendo "nenhuma sonda", que e' a pior mentira possivel aqui.
//!
//! Entao: linha que casa vira sonda; linha que nao casa e' IGNORADA; e quando
//! nada casa, a saida CRUA volta junto para a UI mostrar. "Nao entendi o que o
//! probe-rs respondeu, e aqui esta o que ele disse" e' acionavel. "Nenhuma
//! sonda" quando ha uma plugada e' o item 4 da §5.1 sendo violado.

use std::process::Command;

use kinein_protocol::{ProbeInfo, ProbeListResult};

/// Executavel padrao. O kit pode ter fixado outro (papel `debugAdapter`).
const DEFAULT_PROBE_TOOL: &str = "probe-rs";

/// Lista as sondas conectadas executando `<programa> list`.
///
/// `program` vem do kit quando o usuario fixou um; senao, o `PATH` decide.
#[must_use]
pub fn list(program: Option<&std::path::Path>) -> ProbeListResult {
    let executavel = program.map_or_else(
        || std::path::PathBuf::from(DEFAULT_PROBE_TOOL),
        std::path::Path::to_path_buf,
    );
    let saida = Command::new(&executavel).arg("list").output();

    let Ok(saida) = saida else {
        return ProbeListResult {
            probes: Vec::new(),
            tool_available: false,
            raw_output: String::new(),
            hint: Some(format!(
                "{} nao foi encontrado. Instale-o ou fixe o adaptador no kit.",
                executavel.display()
            )),
        };
    };

    // O `probe-rs list` escreve em stdout; alguns builds usam stderr para o
    // aviso de permissao. Os dois entram na saida crua, porque e' justamente
    // o aviso que explica "nenhuma sonda" quando ha uma plugada.
    let mut cru = String::from_utf8_lossy(&saida.stdout).into_owned();
    let erro = String::from_utf8_lossy(&saida.stderr);
    if !erro.trim().is_empty() {
        if !cru.is_empty() {
            cru.push('\n');
        }
        cru.push_str(&erro);
    }

    let cru = strip_ansi(&cru);
    let probes = parse_list(&cru);
    let hint = hint_for(&probes, &cru);
    ProbeListResult {
        probes,
        tool_available: true,
        raw_output: cru,
        hint,
    }
}

/// Remove sequencias de escape ANSI.
///
/// O `probe-rs 0.32.0` COLORE os avisos (medido em 2026-09-03). Hoje isso cai em
/// linha que o parser ignora, mas cor numa linha de sonda quebraria o casamento
/// — e a saida crua com `\u{1b}[33m` no meio e' ilegivel para quem le a tela.
fn strip_ansi(texto: &str) -> String {
    let mut saida = String::with_capacity(texto.len());
    let mut chars = texto.chars();
    while let Some(c) = chars.next() {
        if c != '\u{1b}' {
            saida.push(c);
            continue;
        }
        // `ESC [ ... <letra final>`; qualquer outra forma some junto.
        for seguinte in chars.by_ref() {
            if seguinte.is_ascii_alphabetic() {
                break;
            }
        }
    }
    saida
}

/// Interpreta a saida do `probe-rs list`. Funcao pura: e' o que se testa sem
/// sonda, sem processo e sem a ferramenta instalada.
///
/// Formato observado (probe-rs, fontes da comunidade — NAO verificado contra
/// binario instalado):
///
/// ```text
/// [0]: BBC micro:bit CMSIS-DAP -- 0d28:0204:9900000031 (CMSIS-DAP)
/// [0]: STLink V3 -- 0483:374e:002700123456 (ST-LINK)
/// ```
#[must_use]
pub fn parse_list(saida: &str) -> Vec<ProbeInfo> {
    let mut sondas = Vec::new();
    for linha in saida.lines() {
        let linha = linha.trim();
        // `[N]:` abre a linha de uma sonda. Sem isso, e' cabecalho ou aviso.
        let Some(resto) = linha.strip_prefix('[') else {
            continue;
        };
        let Some((_indice, resto)) = resto.split_once("]:") else {
            continue;
        };
        // `nome -- vid:pid:serial (tipo)`
        let Some((nome, identificador)) = resto.split_once("--") else {
            continue;
        };
        let nome = nome.trim();
        let identificador = identificador.trim();
        if nome.is_empty() || identificador.is_empty() {
            continue;
        }
        // O tipo vem entre parenteses no fim, quando vem.
        let (identificador, kind) = match identificador.rsplit_once('(') {
            Some((antes, depois)) => (antes.trim(), depois.trim_end_matches(')').trim().to_owned()),
            None => (identificador, String::new()),
        };
        let mut partes = identificador.split(':');
        let vid = partes.next().unwrap_or_default().trim().to_owned();
        let pid = partes.next().unwrap_or_default().trim().to_owned();
        // O serial pode conter `:`; o resto todo e' serial.
        let serial: String = partes.collect::<Vec<_>>().join(":").trim().to_owned();
        if vid.is_empty() || pid.is_empty() {
            continue;
        }
        sondas.push(ProbeInfo {
            name: nome.to_owned(),
            vid,
            pid,
            serial: (!serial.is_empty()).then_some(serial),
            kind: (!kind.is_empty()).then_some(kind),
        });
    }
    sondas
}

/// A dica que transforma "nao achei" em algo acionavel.
///
/// **O caso de udev e' o mais importante e o mais esquecido.** No Linux a sonda
/// so aparece sem `sudo` se houver regra de udev; sem ela, a ferramenta roda,
/// nao acha nada, e o usuario conclui que a placa esta com defeito. Plug and
/// play morre exatamente ai (`integracoes/36` §5).
fn hint_for(probes: &[ProbeInfo], cru: &str) -> Option<String> {
    if !probes.is_empty() {
        return None;
    }
    let baixo = cru.to_ascii_lowercase();

    // O `probe-rs 0.32.0` diz isto explicitamente quando nao ha sonda, E imprime
    // os avisos de udev JUNTO, sempre, como orientacao de setup no Linux
    // (medido em 2026-09-03). Culpar udev aqui seria diagnostico FALSO: na
    // maioria das vezes nao ha sonda plugada mesmo. A ferramenta e' quem sabe a
    // diferenca, e ela ja disse — entao repetimos a condicional dela em vez de
    // escolher um culpado.
    if baixo.contains("no debug probes were found") {
        return Some(
            "nenhuma sonda conectada. Se ha uma plugada, a propria ferramenta \
             aponta permissao: o usuario precisa de acesso ao dispositivo, o que \
             se resolve com regra de udev — nao rodando a IDE como root."
                .to_owned(),
        );
    }
    if baixo.contains("permission") || baixo.contains("denied") || baixo.contains("udev") {
        return Some(
            "a ferramenta viu um dispositivo mas nao pode abri-lo: falta regra de \
             udev. Sem ela a sonda so aparece com sudo, e rodar a IDE como root \
             nao e a solucao."
                .to_owned(),
        );
    }
    if cru.trim().is_empty() {
        return Some(
            "a ferramenta respondeu vazio. Sonda desconectada, ou cabo de dados \
             trocado por um de so carga."
                .to_owned(),
        );
    }
    Some(
        "nenhuma sonda foi reconhecida na resposta da ferramenta. A saida crua \
         esta abaixo — se ha uma sonda ali, o formato mudou e o parser precisa \
         acompanhar."
            .to_owned(),
    )
}

#[cfg(test)]
mod tests {
    use super::{hint_for, parse_list, strip_ansi};

    /// As duas formas observadas de saida do `probe-rs list`.
    #[test]
    fn the_two_observed_formats_are_parsed() {
        let sondas = parse_list(
            "[0]: BBC micro:bit CMSIS-DAP -- 0d28:0204:9900000031 (CMSIS-DAP)\n\
             [1]: STLink V3 -- 0483:374e:002700123456 (ST-LINK)\n",
        );
        assert_eq!(sondas.len(), 2);
        assert_eq!(sondas[0].name, "BBC micro:bit CMSIS-DAP");
        assert_eq!(sondas[0].vid, "0d28");
        assert_eq!(sondas[0].pid, "0204");
        assert_eq!(sondas[0].serial.as_deref(), Some("9900000031"));
        assert_eq!(sondas[0].kind.as_deref(), Some("CMSIS-DAP"));
        assert_eq!(sondas[1].name, "STLink V3");
        assert_eq!(sondas[1].kind.as_deref(), Some("ST-LINK"));
    }

    /// Linha que nao casa e IGNORADA, nunca vira sonda meia-boca. O formato
    /// veio de fonte da comunidade e pode mudar; inventar uma sonda a partir
    /// de um cabecalho seria pior que nao achar nenhuma.
    #[test]
    fn lines_that_do_not_match_are_ignored_not_guessed() {
        let sondas = parse_list(
            "The following debug probes were found:\n\
             [0]: sem identificador\n\
             [1]: nome -- (CMSIS-DAP)\n\
             lixo qualquer\n",
        );
        assert!(sondas.is_empty(), "inventou sonda: {sondas:?}");
    }

    /// Serial com `:` nao quebra o parse: vid e pid sao os DOIS primeiros
    /// campos, e o resto todo e' serial.
    #[test]
    fn a_serial_containing_colons_stays_whole() {
        let sondas = parse_list("[0]: Sonda -- 1234:5678:AA:BB:CC (CMSIS-DAP)\n");
        assert_eq!(sondas.len(), 1);
        assert_eq!(sondas[0].vid, "1234");
        assert_eq!(sondas[0].pid, "5678");
        assert_eq!(sondas[0].serial.as_deref(), Some("AA:BB:CC"));
    }

    /// Sonda sem tipo entre parenteses continua sendo sonda.
    #[test]
    fn a_probe_without_a_kind_is_still_a_probe() {
        let sondas = parse_list("[0]: Generica -- 1111:2222:S1\n");
        assert_eq!(sondas.len(), 1);
        assert_eq!(sondas[0].kind, None);
        assert_eq!(sondas[0].serial.as_deref(), Some("S1"));
    }

    /// A dica de UDEV e a mais importante: sem regra, a ferramenta roda, nao
    /// acha nada, e o usuario culpa a placa. Plug and play morre ai.
    #[test]
    fn permission_failure_points_at_udev_instead_of_blaming_the_board() {
        let dica = hint_for(&[], "Error: Permission denied (os error 13)").unwrap();
        assert!(dica.contains("udev"), "a dica nao fala de udev: {dica}");
        assert!(
            dica.contains("sudo"),
            "a dica precisa dizer que rodar como root NAO e a solucao: {dica}"
        );
    }

    #[test]
    fn an_empty_answer_suggests_the_cable_not_a_bug() {
        let dica = hint_for(&[], "   ").unwrap();
        assert!(dica.contains("cabo"), "{dica}");
    }

    /// Saida que nao casou mas NAO esta vazia: a dica tem que admitir que o
    /// formato pode ter mudado, em vez de afirmar que nao ha sonda.
    #[test]
    fn an_unrecognised_answer_admits_the_format_may_have_changed() {
        let dica = hint_for(&[], "algum formato novo aqui").unwrap();
        assert!(dica.contains("formato"), "{dica}");
    }

    /// Com sonda achada, nao ha dica: dica so existe quando ha o que resolver.
    #[test]
    fn no_hint_when_a_probe_was_found() {
        let sondas = parse_list("[0]: Sonda -- 1111:2222:S1 (CMSIS-DAP)\n");
        assert_eq!(hint_for(&sondas, "irrelevante"), None);
    }
    /// Saida REAL do `probe-rs 0.32.0` nesta maquina, sem sonda plugada,
    /// capturada em 2026-09-03. Os escapes ANSI estao preservados de proposito:
    /// e' o que a ferramenta emite de verdade.
    const SAIDA_REAL_SEM_SONDA: &str = concat!(
        "No debug probes were found.\n",
        "\u{1b}[33m WARN\u{1b}[0m \u{1b}[2mprobe_rs::util::setup_hints::linux\u{1b}[0m\u{1b}[2m:\u{1b}[0m ",
        "If your probe is plugged in but not listed, or shown as inaccessible, ",
        "it is most likely a permissions problem.\n",
        "\u{1b}[33m WARN\u{1b}[0m \u{1b}[2mprobe_rs::util::setup_hints::linux\u{1b}[0m\u{1b}[2m:\u{1b}[0m ",
        "See https://probe.rs/docs/getting-started/probe-setup/ for how to set up ",
        "the required udev rules and group membership.\n",
    );

    /// O `probe-rs` COLORE a saida. Cor numa linha de sonda quebraria o
    /// casamento, e cor na saida crua deixa a tela ilegivel.
    #[test]
    fn ansi_escapes_are_stripped() {
        let limpo = strip_ansi(SAIDA_REAL_SEM_SONDA);
        assert!(!limpo.contains('\u{1b}'), "sobrou escape: {limpo:?}");
        assert!(limpo.contains("No debug probes were found."));
        assert!(limpo.contains("permissions problem"));
    }

    /// O BUG QUE A EXERCITACAO ACHOU. O `probe-rs` imprime os avisos de udev
    /// SEMPRE no Linux, como orientacao de setup — nao porque haja uma sonda
    /// inacessivel. Culpar udev quando a ferramenta disse "No debug probes were
    /// found" e' diagnostico FALSO, e manda o usuario mexer em regra de sistema
    /// quando o cabo e' que nao esta plugado.
    #[test]
    fn the_setup_warnings_alone_do_not_accuse_udev() {
        let limpo = strip_ansi(SAIDA_REAL_SEM_SONDA);
        assert!(parse_list(&limpo).is_empty(), "aviso virou sonda");

        let dica = hint_for(&[], &limpo).unwrap();
        assert!(
            dica.starts_with("nenhuma sonda conectada"),
            "acusou udev com a ferramenta dizendo que nao ha sonda: {dica}"
        );
        // A condicional da propria ferramenta continua sendo repassada: se HA
        // uma plugada, ai sim e' permissao.
        assert!(dica.contains("Se ha uma plugada"), "{dica}");
        assert!(dica.contains("root"), "{dica}");
    }

    /// Sonda INACESSIVEL (nao "ausente") continua acusando udev direto.
    #[test]
    fn an_inaccessible_probe_still_points_at_udev() {
        let dica = hint_for(&[], "Error: Permission denied opening /dev/bus/usb/001/004").unwrap();
        assert!(dica.contains("udev"), "{dica}");
    }
}
