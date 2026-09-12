//! QUAL adaptador sobe, com que argumentos, e com que pedido inicial.
//!
//! Saiu do `session.rs` em 2026-09-11 (fatia 2 da frente F, `roadmaps/35`
//! §5.7), quando o adaptador deixou de ser "um executavel e seus argumentos" e
//! passou a decidir tambem o PEDIDO: `launch` para o desktop e `attach` para o
//! alvo remoto. A sessao continua sendo dona do ciclo de vida; este modulo e'
//! dono da escolha.

use std::path::{Path, PathBuf};

use serde_json::{Value, json};

/// Adaptador usado quando o kit nao escolheu nenhum.
///
/// Era uma CONSTANTE ate 2026-09-03 (etapa 22 do `roadmaps/35`), e por isso nao
/// havia debug de embarcado nenhum: embarcado nao debuga com `lldb-dap`. Virou
/// padrao, nao imposicao — sem escolha, o comportamento e byte a byte o de
/// antes.
pub(super) const DEFAULT_ADAPTER: &str = "lldb-dap";

/// O que o kit escolheu, como chega do handler de `debug.start`.
#[derive(Debug, Clone, Copy, Default)]
pub struct AdapterChoice<'a> {
    /// Id do candidato (`lldb-dap`, `probe-rs`, `gdb`); `None` = o padrao.
    pub id: Option<&'a str>,
    /// Caminho que o kit fixou; `None` = o nome nu, e o `PATH` decide.
    pub path: Option<&'a Path>,
    /// Chip do alvo, para o `launch` do probe-rs.
    pub chip: Option<&'a str>,
    /// `host:porta` do servidor GDB — quando existe, o pedido e' `attach`.
    pub remote_target: Option<&'a str>,
    /// Comando do servidor que a IDE sobe antes de conectar.
    pub debug_server: Option<&'a str>,
}

/// Como cada adaptador conhecido e' invocado.
///
/// **Por que este mapa mora aqui e nao no catalogo da toolchain.** O catalogo
/// responde *"quais binarios interessam a cada papel"*; ele nao sabe — nem deve
/// saber — que o `probe-rs` precisa do subcomando `dap-server` para falar DAP.
/// Quem sobe o processo e' este modulo, e o argumento e' parte de subir.
///
/// Adaptador desconhecido nao e' erro: ele e' executado sem argumento, que e' a
/// forma da maioria dos adaptadores DAP. Recusar o que nao esta na lista
/// impediria o usuario de apontar um adaptador que nos nao conhecemos.
fn adapter_arguments(id: &str) -> &'static [&'static str] {
    match id {
        // `probe-rs dap-server` sem `--port` fala DAP por stdin/stdout — a
        // MESMA forma que este modulo ja usa (`integracoes/36` §3).
        "probe-rs" => &["dap-server"],
        // O GDB fala DAP desde a v14 (`/usr/share/doc/gdb/NEWS`, "Changes in
        // GDB 14"), por stdin/stdout, com `-i dap`. Os dois `-iex` vem de
        // medicao em 2026-09-11: com `DEBUGINFOD_URLS` no ambiente (o Fedora
        // o exporta) o `file` PERGUNTA se pode baixar debuginfo e o `attach`
        // trava mudo; um ELF de embarcado nao tem debuginfod nenhum. Vale para
        // TODO GDB — `gdb-multiarch`, `arm-none-eabi-gdb`, `xtensa-esp-elf-gdb`
        // (integracoes/39): sao o mesmo programa com outro alvo compilado.
        id if e_um_gdb(id) => &[
            "-q",
            "-iex",
            "set debuginfod enabled off",
            "-iex",
            "set confirm off",
            "-i",
            "dap",
        ],
        _ => &[],
    }
}

/// `gdb`, `gdb-multiarch`, `<triple>-gdb`: o GDB em qualquer grafia.
fn e_um_gdb(id: &str) -> bool {
    id == "gdb" || id == "gdb-multiarch" || id.ends_with("-gdb")
}

/// O adaptador escolhido: o que executar, com que argumentos, e como pedir.
#[derive(Debug, Clone)]
pub(super) struct Adapter {
    /// Id do candidato — vai no `adapterID` do `initialize`.
    pub(super) id: String,
    /// Executavel — caminho resolvido pelo kit, ou o nome nu para o `PATH`.
    pub(super) program: PathBuf,
    /// Argumentos que fazem esse executavel falar DAP.
    pub(super) arguments: &'static [&'static str],
    /// Chip do alvo, quando o kit escolheu um.
    ///
    /// Vai no `launch`, **nao** na linha de comando: verificado na doc do
    /// probe-rs, onde `chip` e' campo da configuracao de launch e nao flag do
    /// `dap-server`. Supor o contrario daria um processo que sobe e falha no
    /// primeiro request, com a causa longe do sintoma.
    pub(super) chip: Option<String>,
    /// `host:porta` do servidor GDB. Presente = o pedido inicial e' `attach`.
    pub(super) remote_target: Option<String>,
    /// Comando do servidor que a sessao sobe antes do adaptador.
    pub(super) debug_server: Option<String>,
}

impl Adapter {
    /// Monta o adaptador a partir do que o kit escolheu.
    pub(super) fn from_choice(choice: &AdapterChoice<'_>) -> Self {
        let id = choice.id.unwrap_or(DEFAULT_ADAPTER);
        Self {
            id: id.to_owned(),
            program: choice
                .path
                .map_or_else(|| PathBuf::from(id), Path::to_path_buf),
            arguments: adapter_arguments(id),
            chip: choice.chip.map(str::to_owned),
            remote_target: choice.remote_target.map(str::to_owned),
            debug_server: choice.debug_server.map(str::to_owned),
        }
    }

    /// O pedido que inicia a sessao, e seus argumentos.
    ///
    /// `attach` quando ha' alvo remoto — o `target` e' *"passed to the `target
    /// remote` command"* (manual do GDB, capitulo Debugger Adapter Protocol),
    /// e `program` e' o ELF que da' os simbolos. `launch` caso contrario, que
    /// e' o desktop de sempre. Nos dois o adaptador ADIA a resposta ate' o
    /// `configurationDone` (medido no GDB 17.2 e no lldb-dap), e e' por isso
    /// que a sessao manda o pedido e so' espera a resposta no fim.
    pub(super) fn start_request(&self, root: &Path, program: &Path) -> (&'static str, Value) {
        if let Some(target) = &self.remote_target {
            return (
                "attach",
                json!({
                    "target": target,
                    "program": program.display().to_string(),
                }),
            );
        }
        let mut arguments = json!({
            "program": program.display().to_string(),
            "cwd": root.display().to_string(),
            "stopOnEntry": false,
        });
        // O chip so' entra quando o kit escolheu um. Mandar `"chip": null` para
        // um adaptador que nao o espera e' pedir para ele tratar como valor —
        // a mesma regra da condicao de breakpoint (0.66.0).
        if let Some(chip) = &self.chip {
            arguments["chip"] = json!(chip);
        }
        ("launch", arguments)
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{Adapter, AdapterChoice};

    fn adapter(choice: AdapterChoice<'_>) -> Adapter {
        Adapter::from_choice(&choice)
    }

    /// Sem escolha no kit, o adaptador e o de sempre e SEM argumento — e' o
    /// que garante que o desktop nao mudou quando o embarcado entrou.
    #[test]
    fn no_choice_keeps_the_previous_behaviour() {
        let padrao = adapter(AdapterChoice::default());
        assert_eq!(padrao.program, PathBuf::from("lldb-dap"));
        assert!(
            padrao.arguments.is_empty(),
            "o padrao ganhou argumento: {:?}",
            padrao.arguments
        );
        let (pedido, argumentos) = padrao.start_request(Path::new("/w"), Path::new("/w/app"));
        assert_eq!(pedido, "launch");
        assert_eq!(argumentos["program"], "/w/app");
        assert_eq!(argumentos["cwd"], "/w");
        assert!(
            argumentos.get("chip").is_none(),
            "campo ausente nao e' campo nulo"
        );
        assert!(argumentos.get("target").is_none());
    }

    /// `probe-rs` so' fala DAP com o subcomando `dap-server`. Sem ele o
    /// processo sobe e nao responde ao `initialize` — falha que so' apareceria
    /// como timeout, longe da causa.
    #[test]
    fn probe_rs_gets_the_subcommand_that_makes_it_speak_dap() {
        let escolhido = adapter(AdapterChoice {
            id: Some("probe-rs"),
            ..AdapterChoice::default()
        });
        assert_eq!(escolhido.program, PathBuf::from("probe-rs"));
        assert_eq!(escolhido.arguments, &["dap-server"]);
    }

    /// Todo GDB e' o mesmo programa com outro alvo: `gdb-multiarch` e os
    /// `<triple>-gdb` dos tarballs recebem os mesmos argumentos; um adaptador
    /// que so' TERMINA parecido (`mygdb`) nao.
    #[test]
    fn every_gdb_spelling_gets_the_dap_arguments() {
        for id in [
            "gdb-multiarch",
            "arm-none-eabi-gdb",
            "xtensa-esp-elf-gdb",
            "riscv32-esp-elf-gdb",
        ] {
            let variante = adapter(AdapterChoice {
                id: Some(id),
                ..AdapterChoice::default()
            });
            assert!(
                variante.arguments.ends_with(&["-i", "dap"]),
                "{id}: {:?}",
                variante.arguments
            );
        }
        let estranho = adapter(AdapterChoice {
            id: Some("mygdb"),
            ..AdapterChoice::default()
        });
        assert!(estranho.arguments.is_empty(), "{:?}", estranho.arguments);
    }

    /// O GDB precisa de `-i dap` para falar DAP, e dos dois `-iex` para nao
    /// travar mudo perguntando pelo debuginfod (medido em 2026-09-11).
    #[test]
    fn gdb_speaks_dap_with_the_interpreter_flag_and_no_prompts() {
        let gdb = adapter(AdapterChoice {
            id: Some("gdb"),
            ..AdapterChoice::default()
        });
        assert_eq!(gdb.program, PathBuf::from("gdb"));
        assert_eq!(
            gdb.arguments
                .iter()
                .rev()
                .take(2)
                .rev()
                .copied()
                .collect::<Vec<_>>(),
            ["-i", "dap"]
        );
        assert!(
            gdb.arguments.contains(&"set debuginfod enabled off"),
            "sem isto o attach trava perguntando: {:?}",
            gdb.arguments
        );
    }

    /// O caminho fixado pelo kit vence o nome nu; os argumentos continuam
    /// vindo do ID, nao do caminho — um `probe-rs` em /opt ainda precisa do
    /// subcomando.
    #[test]
    fn a_pinned_path_keeps_the_arguments_of_its_id() {
        let fixado = adapter(AdapterChoice {
            id: Some("probe-rs"),
            path: Some(Path::new("/opt/embarcado/probe-rs")),
            ..AdapterChoice::default()
        });
        assert_eq!(fixado.program, PathBuf::from("/opt/embarcado/probe-rs"));
        assert_eq!(fixado.arguments, &["dap-server"]);
    }

    /// Adaptador que nao conhecemos NAO e' recusado: roda sem argumento, que e'
    /// a forma da maioria dos adaptadores DAP. Recusar impediria o usuario de
    /// apontar um que nos nao listamos.
    #[test]
    fn an_unknown_adapter_runs_bare_instead_of_being_refused() {
        let outro = adapter(AdapterChoice {
            id: Some("meu-dap"),
            ..AdapterChoice::default()
        });
        assert_eq!(outro.program, PathBuf::from("meu-dap"));
        assert!(outro.arguments.is_empty());
    }

    /// O chip NAO vira argumento de linha de comando: e' campo do `launch`
    /// (doc do probe-rs). Um adaptador que recebesse `--chip` inesperado sobe
    /// e falha no primeiro request, com a causa longe do sintoma.
    #[test]
    fn the_chip_goes_in_the_launch_and_never_in_the_command_line() {
        let com_chip = adapter(AdapterChoice {
            id: Some("probe-rs"),
            chip: Some("STM32H745ZITx"),
            ..AdapterChoice::default()
        });
        assert_eq!(com_chip.arguments, &["dap-server"]);
        let (pedido, argumentos) = com_chip.start_request(Path::new("/w"), Path::new("/w/fw.elf"));
        assert_eq!(pedido, "launch");
        assert_eq!(argumentos["chip"], "STM32H745ZITx");
    }

    /// Alvo remoto = `attach`, com o `target` cru para o `target remote` e o
    /// ELF em `program`. Sem `cwd` nem `stopOnEntry`: sao do `launch`, e o GDB
    /// recusa argumento que nao conhece? Nao — ignora; mas mandar o que nao
    /// se aplica e' o que a regra do campo ausente proibe.
    #[test]
    fn a_remote_target_turns_the_start_into_an_attach() {
        let remoto = adapter(AdapterChoice {
            id: Some("gdb"),
            remote_target: Some("localhost:3333"),
            debug_server: Some("qemu-system-arm -S -gdb tcp::3333 -kernel {program}"),
            ..AdapterChoice::default()
        });
        let (pedido, argumentos) = remoto.start_request(Path::new("/w"), Path::new("/w/fw.elf"));
        assert_eq!(pedido, "attach");
        assert_eq!(argumentos["target"], "localhost:3333");
        assert_eq!(argumentos["program"], "/w/fw.elf");
        assert!(argumentos.get("cwd").is_none());
        assert_eq!(
            remoto.debug_server.as_deref(),
            Some("qemu-system-arm -S -gdb tcp::3333 -kernel {program}")
        );
    }
}
