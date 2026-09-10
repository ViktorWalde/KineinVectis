//! O TRANSPORTE: subir o processo, escrever, esperar com teto, ler o que veio.
//!
//! **Arquivo proprio desde 2026-09-10.** "Como falar com o processo" e "o que
//! aceitar de volta" sao duas perguntas, e a segunda mora no [`super::portao`].
//!
//! O que este arquivo garante, e que a medicao obrigou: **o teto nao descarta o
//! que ja' chegou**. O processo responde em DUAS linhas, a barata primeiro, e
//! um `dsolve` que trava mata a segunda sem levar a primeira junto.

use std::io::{BufReader, Read, Write};
use std::process::{Command, Stdio};
use std::time::Instant;

use super::{Config, NaoSei, RESPIRO, programa::PROGRAMA};

/// Roda o processo com o [`TETO`] e devolve o que ele escreveu.
///
/// **O teto NAO descarta o que ja' chegou.** O segundo campo diz se ele
/// estourou; o primeiro traz as linhas que o processo alcancou descarregar
/// antes de morrer, e a checagem de unidade e' justamente a que chega primeiro.
pub(super) fn executar(config: &Config, corpo: &str) -> Result<(String, bool), NaoSei> {
    let mut filho = Command::new(&config.interpretador)
        .arg("-c")
        .arg(PROGRAMA)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| NaoSei::SemPython)?;

    if let Some(mut entrada) = filho.stdin.take() {
        // Erro de escrita nao e' motivo proprio: o processo morreu, e o que
        // interessa e' o que ele respondeu (ou nao).
        let _ = entrada.write_all(corpo.as_bytes());
    }

    let Some(saida) = filho.stdout.take() else {
        let _ = filho.kill();
        let _ = filho.wait();
        return Err(NaoSei::NaoResolve);
    };
    // Thread leitora: a resposta e' pequena, mas um traceback nao e', e um pipe
    // cheio com ninguem lendo trava o filho ate' o teto — que seria um
    // "TempoEsgotado" mentindo sobre a causa.
    let leitor = std::thread::spawn(move || {
        let mut texto = String::new();
        let _ = BufReader::new(saida).read_to_string(&mut texto);
        texto
    });

    let limite = Instant::now() + config.teto;
    loop {
        // Terminou, ou nao da' mais para perguntar: nos dois casos o que
        // interessa esta no pipe, e quem decide se e' resposta e' o parse.
        if !matches!(filho.try_wait(), Ok(None)) {
            break;
        }
        if Instant::now() >= limite {
            let _ = filho.kill();
            let _ = filho.wait();
            return Ok((leitor.join().unwrap_or_default(), true));
        }
        std::thread::sleep(RESPIRO);
    }
    Ok((leitor.join().unwrap_or_default(), false))
}
