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
pub struct Tentativa {
    /// A linha de comando, como a tela a mostra.
    pub comando: String,
    /// Tudo que a ferramenta escreveu (stdout+stderr, na ordem em que veio).
    pub raw: String,
    /// Saiu com 0.
    pub sucesso: bool,
    /// O prazo venceu antes de a ferramenta sair.
    pub expirou: bool,
}

/// A linha de comando como a tela a mostra: programa e argumentos separados
/// por espaco (o que o usuario copiaria para o terminal).
#[must_use]
pub fn display(program: &str, args: &[String]) -> String {
    let mut partes = vec![program.to_owned()];
    partes.extend(args.iter().cloned());
    partes.join(" ")
}

/// Roda `programa args` ecoando cada linha em `event.job.output`; `nome` e'
/// como a ferramenta e' chamada nas mensagens ("esptool", "mpremote").
#[must_use]
pub fn rodar(
    ctx: &jobs::JobContext,
    programa: &Path,
    args: &[String],
    prazo: Duration,
    nome: &str,
) -> Tentativa {
    let programa = programa.display().to_string();
    let comando = display(&programa, args);
    ctx.emit_output(&format!("$ {comando}"));
    let mut command = std::process::Command::new(&programa);
    command.args(args);

    let parar = Arc::new(AtomicBool::new(false));
    let expirou = Arc::new(AtomicBool::new(false));
    let terminou = Arc::new(AtomicBool::new(false));
    {
        let parar = Arc::clone(&parar);
        let expirou = Arc::clone(&expirou);
        let terminou = Arc::clone(&terminou);
        let cancelado = ctx.cancellation();
        let limite = Instant::now() + prazo;
        std::thread::spawn(move || {
            while !terminou.load(Ordering::SeqCst) {
                if cancelado.load(Ordering::SeqCst) {
                    parar.store(true, Ordering::SeqCst);
                    break;
                }
                if Instant::now() >= limite {
                    expirou.store(true, Ordering::SeqCst);
                    parar.store(true, Ordering::SeqCst);
                    break;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        });
    }
    let mut raw = String::new();
    let resultado = process::stream_command_lines_cancelable(command, &parar, &mut |_, linha| {
        ctx.emit_output(&linha);
        raw.push_str(&linha);
        raw.push('\n');
    });
    terminou.store(true, Ordering::SeqCst);
    let sucesso = match resultado {
        Ok(status) => status.success(),
        Err(error) => {
            let texto = match error {
                process::ProcessError::Spawn(e) => format!("{nome} nao pode ser iniciado: {e}"),
                process::ProcessError::Wait(e) => format!("falha aguardando o {nome}: {e}"),
            };
            ctx.emit_output(&texto);
            raw.push_str(&texto);
            false
        }
    };
    Tentativa {
        comando,
        raw,
        sucesso,
        expirou: expirou.load(Ordering::SeqCst),
    }
}

/// As ultimas linhas nao vazias da saida: e' onde o esptool diz `A fatal
/// error occurred: ...` e o mpremote diz `mpremote: cp: ...`.
#[must_use]
pub fn ultimas_linhas(raw: &str, nome: &str) -> String {
    let linhas: Vec<&str> = raw.lines().filter(|l| !l.trim().is_empty()).collect();
    let inicio = linhas.len().saturating_sub(3);
    let cauda = linhas[inicio..].join("\n");
    if cauda.is_empty() {
        format!("o {nome} saiu com erro sem escrever nada")
    } else {
        cauda
    }
}
