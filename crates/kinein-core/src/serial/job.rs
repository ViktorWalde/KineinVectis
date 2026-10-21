//! Uma ferramenta rodando NA PORTA, com prazo e cancelamento — o que
//! `serial.identify` (esptool) e `serial.files` (mpremote) tem em comum.
//!
//! A porta serial e' um recurso preso: enquanto o esptool sincroniza ou o
//! mpremote copia, nada mais fala com a placa. Por isso cada execucao aqui
//! tem PRAZO (a ferramenta que nao responde nao pode prender a porta para
//! sempre) e honra o cancelamento do job (`job.cancel` mata o processo).
//! O "expirou" e' distinto do "cancelado" para a tela dizer a verdade: e'
//! `parar`, nao o cancelamento do job, que o streamer honra.

use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use crate::{jobs, process};

/// O que uma execucao deixou.
#[derive(Debug)]
pub struct Attempt {
    /// A linha de comando, como a tela a mostra.
    pub command_line: String,
    /// Tudo que a ferramenta escreveu (stdout+stderr, na ordem em que veio).
    pub raw: String,
    /// Saiu com 0.
    pub success: bool,
    /// O prazo venceu antes de a ferramenta sair.
    pub timed_out: bool,
}

/// A linha de comando como a tela a mostra: programa e argumentos separados
/// por espaco (o que o usuario copiaria para o terminal).
#[must_use]
pub fn display(program: &str, args: &[String]) -> String {
    let mut partes = vec![program.to_owned()];
    partes.extend(args.iter().cloned());
    partes.join(" ")
}

/// Uses the existing process runner while the operation owns its serial lease.
pub(crate) struct Runner<'a> {
    context: &'a jobs::JobContext,
    reservation: super::ownership::Lease,
}

impl<'a> Runner<'a> {
    pub(crate) const fn new(
        context: &'a jobs::JobContext,
        reservation: super::ownership::Lease,
    ) -> Self {
        Self {
            context,
            reservation,
        }
    }

    pub(crate) const fn context(&self) -> &'a jobs::JobContext {
        self.context
    }

    pub(crate) fn run(
        &self,
        program: &Path,
        args: &[String],
        timeout: Duration,
        tool_name: &str,
    ) -> Attempt {
        let ctx = self.context;
        let invocation = self.reservation.invocation(program, args);
        let program = program.display().to_string();
        let command_line = display(&program, args);
        ctx.emit_output(&format!("$ {command_line}"));
        let (executable, arguments) = match invocation {
            Ok(invocation) => invocation,
            Err(error) => {
                let raw = format!("Não foi possível preparar a reserva do processo: {error}");
                ctx.emit_output(&raw);
                return Attempt {
                    command_line,
                    raw,
                    success: false,
                    timed_out: false,
                };
            }
        };
        let mut command = std::process::Command::new(executable);
        command.args(arguments);

        let stop = Arc::new(AtomicBool::new(false));
        let timed_out = Arc::new(AtomicBool::new(false));
        let finished = Arc::new(AtomicBool::new(false));
        {
            let stop = Arc::clone(&stop);
            let timed_out = Arc::clone(&timed_out);
            let finished = Arc::clone(&finished);
            let cancelled = ctx.cancellation();
            let deadline = Instant::now() + timeout;
            std::thread::spawn(move || {
                while !finished.load(Ordering::SeqCst) {
                    if cancelled.load(Ordering::SeqCst) {
                        stop.store(true, Ordering::SeqCst);
                        break;
                    }
                    if Instant::now() >= deadline {
                        timed_out.store(true, Ordering::SeqCst);
                        stop.store(true, Ordering::SeqCst);
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
            });
        }
        let mut raw = String::new();
        let result = process::stream_command_lines_cancelable(command, &stop, &mut |_, line| {
            ctx.emit_output(&line);
            raw.push_str(&line);
            raw.push('\n');
        });
        finished.store(true, Ordering::SeqCst);
        let success = match result {
            Ok(status) => status.success(),
            Err(error) => {
                let text = match error {
                    process::ProcessError::Spawn(e) => {
                        format!("{tool_name} não pode ser iniciado: {e}")
                    }
                    process::ProcessError::Wait(e) => {
                        format!("falha aguardando o {tool_name}: {e}")
                    }
                };
                ctx.emit_output(&text);
                raw.push_str(&text);
                false
            }
        };
        Attempt {
            command_line,
            raw,
            success,
            timed_out: timed_out.load(Ordering::SeqCst),
        }
    }
}

/// As ultimas linhas nao vazias da saida: e' onde o esptool diz `A fatal
/// error occurred: ...` e o mpremote diz `mpremote: cp: ...`.
#[must_use]
pub fn ultimas_linhas(raw: &str, tool_name: &str) -> String {
    let linhas: Vec<&str> = raw.lines().filter(|l| !l.trim().is_empty()).collect();
    let inicio = linhas.len().saturating_sub(3);
    let cauda = linhas[inicio..].join("\n");
    if cauda.is_empty() {
        format!("o {tool_name} saiu com erro sem escrever nada")
    } else {
        cauda
    }
}
