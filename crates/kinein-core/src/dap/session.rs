//! A SESSAO DAP viva: sobe o adapter, faz o handshake e opera o alvo.
//!
//! Cortado em 2026-09-03 (etapa 15 do `roadmaps/34`), que estava em 672/500.
//! O corte separou o que o arquivo misturava, e cada parte tem dono:
//!
//! ```text
//! dap/wire.rs     o TRANSPORTE — leva e traz, nao interpreta nada
//! dap/parse.rs    INTERPRETAR a resposta (e o unico testavel sem processo)
//! dap/reader.rs   a thread LEITORA e os eventos do adapter
//! dap/session.rs  ESTE: ciclo de vida do alvo e as operacoes de debug
//! ```

use std::{
    collections::BTreeMap,
    io,
    path::Path,
    process::{Child, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::Duration,
};

use kinein_protocol::{
    BreakpointInfo, DebugEvaluateResult, SourceBreakpointParams, StackFrameInfo, VariableInfo,
};
use serde_json::{Value, json};

use super::DebugError;
use super::parse::{
    breakpoints_arguments, first_cheap_scope_reference, parse_breakpoints, parse_evaluate,
    parse_stack_frames, parse_variables,
};
use super::reader::{note_continued, send_event, spawn_reader};
use super::wire::Wire;
use crate::lsp::EventSender;

/// Adaptador usado quando o kit nao escolheu nenhum.
///
/// Era uma CONSTANTE ate 2026-09-03 (etapa 22 do `roadmaps/35`), e por isso nao
/// havia debug de embarcado nenhum: embarcado nao debuga com `lldb-dap`. Virou
/// padrao, nao imposicao — sem escolha, o comportamento e byte a byte o de
/// antes.
pub(super) const DEFAULT_ADAPTER: &str = "lldb-dap";

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
        _ => &[],
    }
}

/// Tempo maximo aguardando respostas comuns do adapter.
pub(super) const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// Tempo maximo aguardando a resposta de `launch` (carregar o alvo demora
/// mais que um request comum em binarios grandes).
const LAUNCH_TIMEOUT: Duration = Duration::from_secs(20);

/// Frames maximos devolvidos por `debug.stackTrace` (JetBrains-like; a
/// paginacao so entra se o uso real pedir).
const STACK_TRACE_LEVELS: u32 = 20;

/// Tempo maximo do `disconnect` no encerramento; depois o processo e morto.
const DISCONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// Uma sessao de debug viva (um adapter, um processo alvo).
#[derive(Debug)]
pub(super) struct DapSession {
    child: Child,
    wire: Wire,
    events: EventSender,
    alive: Arc<AtomicBool>,
    stopped_thread: Arc<Mutex<Option<i64>>>,
}

/// O adaptador escolhido: o que executar e com que argumentos.
#[derive(Debug, Clone)]
pub(super) struct Adapter {
    /// Executavel — caminho resolvido pelo kit, ou o nome nu para o `PATH`.
    pub(super) program: std::path::PathBuf,
    /// Argumentos que fazem esse executavel falar DAP.
    pub(super) arguments: &'static [&'static str],
    /// Chip do alvo, quando o kit escolheu um.
    ///
    /// Vai no `launch`, **nao** na linha de comando: verificado na doc do
    /// probe-rs, onde `chip` e' campo da configuracao de launch e nao flag do
    /// `dap-server`. Supor o contrario daria um processo que sobe e falha no
    /// primeiro request, com a causa longe do sintoma.
    pub(super) chip: Option<String>,
}

impl Adapter {
    /// Monta o adaptador a partir do id escolhido no kit.
    ///
    /// `id` `None` = o padrao. `resolved` e' o caminho que o kit fixou; sem
    /// ele, usa-se o nome nu e o `PATH` decide — o mesmo contrato de
    /// `Toolchain::program_for`.
    pub(super) fn from_choice(
        id: Option<&str>,
        resolved: Option<&Path>,
        chip: Option<&str>,
    ) -> Self {
        let id = id.unwrap_or(DEFAULT_ADAPTER);
        Self {
            program: resolved.map_or_else(|| std::path::PathBuf::from(id), Path::to_path_buf),
            arguments: adapter_arguments(id),
            chip: chip.map(str::to_owned),
        }
    }
}

impl Drop for DapSession {
    fn drop(&mut self) {
        drop(self.child.kill());
        drop(self.child.wait());
    }
}

impl DapSession {
    /// Sobe o adapter, faz a danca DAP (initialize -> launch -> initialized
    /// -> replay de breakpoints -> configurationDone) e emite
    /// `event.debug.started`.
    pub(super) fn launch(
        root: &Path,
        program: &Path,
        breakpoints: &BTreeMap<String, Vec<SourceBreakpointParams>>,
        events: EventSender,
        adapter: &Adapter,
    ) -> Result<Self, DebugError> {
        let mut child = Command::new(&adapter.program)
            .args(adapter.arguments)
            .current_dir(root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| {
                if error.kind() == io::ErrorKind::NotFound {
                    DebugError::MissingAdapter
                } else {
                    DebugError::Adapter {
                        message: format!("falha ao iniciar {}: {error}", adapter.program.display()),
                    }
                }
            })?;
        let (Some(stdin), Some(stdout)) = (child.stdin.take(), child.stdout.take()) else {
            drop(child.kill());
            return Err(DebugError::Adapter {
                message: format!(
                    "{} subiu sem stdin/stdout utilizaveis",
                    adapter.program.display()
                ),
            });
        };

        let wire = Wire::new(stdin);
        let alive = Arc::new(AtomicBool::new(true));
        let stopped_thread = Arc::new(Mutex::new(None));
        let (initialized_sender, initialized_receiver) = mpsc::channel();

        spawn_reader(
            stdout,
            &wire,
            &events,
            &alive,
            &stopped_thread,
            initialized_sender,
        );

        let session = Self {
            child,
            wire,
            events,
            alive,
            stopped_thread,
        };
        // Drop mata o adapter se qualquer passo do handshake falhar.
        session.handshake(
            root,
            program,
            breakpoints,
            &initialized_receiver,
            adapter.chip.as_deref(),
        )?;
        send_event(
            &session.events,
            "event.debug.started",
            json!({ "program": program.display().to_string() }),
        );
        Ok(session)
    }

    /// `true` enquanto o adapter esta vivo (nem `terminated` nem EOF).
    pub(super) fn is_alive(&self) -> bool {
        self.alive.load(Ordering::SeqCst)
    }

    /// Substitui os breakpoints de `file` no adapter e devolve o resultado.
    pub(super) fn set_breakpoints(
        &self,
        file: &str,
        breakpoints: &[SourceBreakpointParams],
    ) -> Result<Vec<BreakpointInfo>, DebugError> {
        let body = self.wire.request(
            "setBreakpoints",
            &breakpoints_arguments(file, breakpoints),
            REQUEST_TIMEOUT,
        )?;
        let lines: Vec<u32> = breakpoints.iter().map(|bp| bp.line).collect();
        Ok(parse_breakpoints(&body, &lines))
    }

    /// Avalia uma expressao no frame pedido (watch).
    ///
    /// Sem `frame_id`, usa o frame do TOPO da thread parada — e' o que a UI
    /// quer quando o usuario digita um watch sem ter clicado num frame. Se
    /// nao ha' thread parada, `stack_trace` devolve `NotStopped`, que e' a
    /// resposta honesta: nao existe contexto para avaliar.
    pub(super) fn evaluate(
        &self,
        expression: &str,
        frame_id: Option<i64>,
    ) -> Result<DebugEvaluateResult, DebugError> {
        let frame = match frame_id {
            Some(id) => id,
            None => self
                .stack_trace()?
                .first()
                .map(|frame| frame.id)
                .ok_or(DebugError::NotStopped)?,
        };
        let body = self.wire.request(
            "evaluate",
            &json!({
                "expression": expression,
                "frameId": frame,
                "context": "watch",
            }),
            REQUEST_TIMEOUT,
        )?;
        Ok(parse_evaluate(expression, &body))
    }

    /// Retoma a execucao a partir da thread pausada.
    pub(super) fn continue_run(&self) -> Result<(), DebugError> {
        let thread = self.stopped_thread_id()?;
        self.wire
            .request("continue", &json!({ "threadId": thread }), REQUEST_TIMEOUT)?;
        note_continued(&self.stopped_thread, &self.events);
        Ok(())
    }

    /// Executa um passo (`next`, `stepIn` ou `stepOut`) na thread pausada.
    pub(super) fn step(&self, command: &str) -> Result<(), DebugError> {
        let thread = self.stopped_thread_id()?;
        self.wire
            .request(command, &json!({ "threadId": thread }), REQUEST_TIMEOUT)?;
        note_continued(&self.stopped_thread, &self.events);
        Ok(())
    }

    /// Pausa o processo em execucao (primeira thread listada pelo adapter).
    pub(super) fn pause(&self) -> Result<(), DebugError> {
        let body = self.wire.request("threads", &json!({}), REQUEST_TIMEOUT)?;
        let thread = body
            .get("threads")
            .and_then(Value::as_array)
            .and_then(|threads| threads.first())
            .and_then(|thread| thread.get("id"))
            .and_then(Value::as_i64)
            .ok_or_else(|| DebugError::Adapter {
                message: "o adapter nao listou threads para pausar".to_owned(),
            })?;
        self.wire
            .request("pause", &json!({ "threadId": thread }), REQUEST_TIMEOUT)?;
        Ok(())
    }

    /// Frames da thread pausada, do topo para baixo.
    pub(super) fn stack_trace(&self) -> Result<Vec<StackFrameInfo>, DebugError> {
        let thread = self.stopped_thread_id()?;
        let body = self.wire.request(
            "stackTrace",
            &json!({
                "threadId": thread,
                "startFrame": 0,
                "levels": STACK_TRACE_LEVELS,
            }),
            REQUEST_TIMEOUT,
        )?;
        Ok(parse_stack_frames(&body))
    }

    /// Variaveis do primeiro escopo nao-caro do frame (Locals no lldb-dap).
    ///
    /// A UI nao conhece "scopes": o core resolve `scopes(frameId)` aqui e
    /// devolve direto as variaveis. Globals/Registers ficam pos-M2.
    pub(super) fn frame_variables(&self, frame_id: i64) -> Result<Vec<VariableInfo>, DebugError> {
        let _ = self.stopped_thread_id()?;
        let scopes =
            self.wire
                .request("scopes", &json!({ "frameId": frame_id }), REQUEST_TIMEOUT)?;
        let Some(reference) = first_cheap_scope_reference(&scopes) else {
            return Ok(Vec::new());
        };
        self.reference_variables(reference)
    }

    /// Expande uma variavel estruturada (handle `ref` de resposta anterior).
    pub(super) fn reference_variables(
        &self,
        reference: i64,
    ) -> Result<Vec<VariableInfo>, DebugError> {
        let _ = self.stopped_thread_id()?;
        let body = self.wire.request(
            "variables",
            &json!({ "variablesReference": reference }),
            REQUEST_TIMEOUT,
        )?;
        Ok(parse_variables(&body))
    }

    /// Pede o encerramento educado; `terminated`/EOF emitem o `finished`.
    pub(super) fn shutdown(&self) {
        drop(self.wire.request(
            "disconnect",
            &json!({ "terminateDebuggee": true }),
            DISCONNECT_TIMEOUT,
        ));
    }

    /// Thread pausada atual, ou o erro que explica que nada esta pausado.
    fn stopped_thread_id(&self) -> Result<i64, DebugError> {
        self.stopped_thread
            .lock()
            .ok()
            .and_then(|guard| *guard)
            .ok_or(DebugError::NotStopped)
    }

    /// A danca de inicializacao do DAP, na ordem que os adapters esperam.
    fn handshake(
        &self,
        root: &Path,
        program: &Path,
        breakpoints: &BTreeMap<String, Vec<SourceBreakpointParams>>,
        initialized: &mpsc::Receiver<()>,
        chip: Option<&str>,
    ) -> Result<(), DebugError> {
        self.wire.request(
            "initialize",
            &json!({
                "clientID": "kinein-vectis",
                "adapterID": "lldb",
                "linesStartAt1": true,
                "columnsStartAt1": true,
                "pathFormat": "path",
            }),
            REQUEST_TIMEOUT,
        )?;
        // A resposta de launch so chega depois do configurationDone; o
        // request vai agora e a espera fica para o final.
        let mut launch_args = json!({
            "program": program.display().to_string(),
            "cwd": root.display().to_string(),
            "stopOnEntry": false,
        });
        // O chip so' entra quando o kit escolheu um. Mandar `"chip": null` para
        // um adaptador que nao o espera e' pedir para ele tratar como valor —
        // a mesma regra da condicao de breakpoint (0.66.0).
        if let Some(chip) = chip {
            launch_args["chip"] = json!(chip);
        }
        let (launch_seq, launch_receiver) = self.wire.send_request("launch", &launch_args)?;
        if initialized.recv_timeout(REQUEST_TIMEOUT).is_err() {
            return Err(DebugError::Adapter {
                message: "o adapter nao publicou o evento initialized".to_owned(),
            });
        }
        for (file, lines) in breakpoints {
            self.wire.request(
                "setBreakpoints",
                &breakpoints_arguments(file, lines),
                REQUEST_TIMEOUT,
            )?;
        }
        self.wire
            .request("configurationDone", &json!({}), REQUEST_TIMEOUT)?;
        self.wire
            .wait_response("launch", launch_seq, &launch_receiver, LAUNCH_TIMEOUT)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    /// Sem escolha no kit, o adaptador e o de sempre e SEM argumento — e' o
    /// que garante que o desktop nao mudou quando o embarcado entrou.
    #[test]
    fn no_choice_keeps_the_previous_behaviour() {
        let padrao = super::Adapter::from_choice(None, None, None);
        assert_eq!(padrao.program, std::path::PathBuf::from("lldb-dap"));
        assert!(
            padrao.arguments.is_empty(),
            "o padrao ganhou argumento: {:?}",
            padrao.arguments
        );
    }

    /// `probe-rs` so' fala DAP com o subcomando `dap-server`. Sem ele o
    /// processo sobe e nao responde ao `initialize` — falha que so' apareceria
    /// como timeout, longe da causa.
    #[test]
    fn probe_rs_gets_the_subcommand_that_makes_it_speak_dap() {
        let escolhido = super::Adapter::from_choice(Some("probe-rs"), None, None);
        assert_eq!(escolhido.program, std::path::PathBuf::from("probe-rs"));
        assert_eq!(escolhido.arguments, &["dap-server"]);
    }

    /// O caminho fixado pelo kit vence o nome nu; os argumentos continuam
    /// vindo do ID, nao do caminho — um `probe-rs` em /opt ainda precisa do
    /// subcomando.
    #[test]
    fn a_pinned_path_keeps_the_arguments_of_its_id() {
        let fixado = super::Adapter::from_choice(
            Some("probe-rs"),
            Some(std::path::Path::new("/opt/embarcado/probe-rs")),
            None,
        );
        assert_eq!(
            fixado.program,
            std::path::PathBuf::from("/opt/embarcado/probe-rs")
        );
        assert_eq!(fixado.arguments, &["dap-server"]);
    }

    /// Adaptador que nao conhecemos NAO e' recusado: roda sem argumento, que e'
    /// a forma da maioria dos adaptadores DAP. Recusar impediria o usuario de
    /// apontar um que nos nao listamos.
    #[test]
    fn an_unknown_adapter_runs_bare_instead_of_being_refused() {
        let outro = super::Adapter::from_choice(Some("meu-dap"), None, None);
        assert_eq!(outro.program, std::path::PathBuf::from("meu-dap"));
        assert!(outro.arguments.is_empty());
    }

    /// O chip NAO vira argumento de linha de comando. Verificado na doc do
    /// probe-rs: `chip` e' campo da configuracao de LAUNCH, nao flag do
    /// `dap-server`. Um adaptador que recebesse `--chip` inesperado sobe e
    /// falha no primeiro request, com a causa longe do sintoma.
    #[test]
    fn the_chip_never_becomes_a_command_line_argument() {
        let com_chip = super::Adapter::from_choice(Some("probe-rs"), None, Some("STM32H745ZITx"));
        assert_eq!(com_chip.arguments, &["dap-server"]);
        assert!(
            !com_chip.arguments.iter().any(|a| a.contains("chip")),
            "o chip vazou para a linha de comando: {:?}",
            com_chip.arguments
        );
        assert_eq!(com_chip.chip.as_deref(), Some("STM32H745ZITx"));
    }

    /// Sem chip escolhido, o campo nao existe — e' a mesma regra da condicao
    /// de breakpoint: campo ausente nao e' campo nulo.
    #[test]
    fn no_chip_means_no_field() {
        assert_eq!(super::Adapter::from_choice(None, None, None).chip, None);
    }
}
