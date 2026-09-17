//! O stderr de um processo filho de LONGA VIDA (adaptador DAP, servidor de
//! debug, servidor LSP): guardado numa cauda limitada e, quando o dono quer,
//! entregue linha a linha a um coletor.
//!
//! POR QUE EXISTE (40 §4, lacuna vista em 2026-09-13 e fechada em 2026-09-17).
//! Os tres filhos nasciam com `Stdio::null()` no stderr. O gate Python falhou
//! uma vez com "o adapter nao respondeu a `initialize`" e NAO HAVIA COMO
//! SABER POR QUE — o adaptador tinha dito, no stderr, e ninguem ouviu. O
//! `process.rs` ja' drena stdout/stderr, mas para processos que TERMINAM
//! (build, test): ele espera o fim. Aqui o filho vive enquanto a sessao
//! vive, e o que se quer e' (a) a cauda no momento do erro e (b) a linha ao
//! vivo para a aba IDE.
//!
//! Uma thread por filho (nao por linha), termina no EOF — o pipe fecha quando
//! o filho morre. `Drop` nao faz `join`: o filho e' morto pelo dono do
//! `Child`, e a thread ve o EOF logo depois.

use std::{
    collections::VecDeque,
    io::{BufRead, BufReader, Read},
    sync::{Arc, Mutex},
    thread,
};

/// Quantas linhas a cauda guarda por padrao. Cabe numa mensagem de erro e
/// ainda mostra o traceback inteiro de um `python -m debugpy` que morreu.
pub const CAPACIDADE_PADRAO: usize = 64;

/// Acima disto a linha e' cortada com marcador: um filho em loop escrevendo
/// sem `\n` nao pode encher a memoria do core.
const LINHA_MAXIMA: usize = 4096;

/// Coletor de linha ao vivo: recebe a linha ja' limpa (sem `\n`, truncada).
pub type Coletor = Box<dyn Fn(&str) + Send + 'static>;

/// A cauda do stderr de um filho, viva enquanto o pipe estiver aberto.
#[derive(Debug, Clone)]
pub struct StderrTail {
    linhas: Arc<Mutex<VecDeque<String>>>,
    capacidade: usize,
}

impl StderrTail {
    /// Toma o pipe e sobe a thread leitora. `coletor`, quando dado, recebe
    /// cada linha assim que ela chega — e' o que vira evento para a UI.
    #[must_use]
    pub fn spawn(
        stderr: impl Read + Send + 'static,
        capacidade: usize,
        coletor: Option<Coletor>,
    ) -> Self {
        let capacidade = capacidade.max(1);
        let linhas = Arc::new(Mutex::new(VecDeque::with_capacity(capacidade)));
        let cauda = Self {
            linhas: Arc::clone(&linhas),
            capacidade,
        };
        thread::spawn(move || {
            let mut reader = BufReader::new(stderr);
            let mut bruto = Vec::new();
            loop {
                bruto.clear();
                // `read_until` em vez de `lines()`: uma linha invalida em
                // UTF-8 (clangd em locale estranho) nao encerra a leitura.
                match reader.read_until(b'\n', &mut bruto) {
                    Ok(0) | Err(_) => break,
                    Ok(_) => {}
                }
                let linha = limpar(&bruto);
                if let Ok(mut guard) = linhas.lock() {
                    if guard.len() == capacidade {
                        guard.pop_front();
                    }
                    guard.push_back(linha.clone());
                }
                if let Some(coletor) = &coletor {
                    coletor(&linha);
                }
            }
        });
        cauda
    }

    /// As ultimas linhas, unidas por `\n`; vazia quando o filho nao disse nada.
    #[must_use]
    pub fn tail(&self) -> String {
        self.linhas
            .lock()
            .map(|guard| guard.iter().cloned().collect::<Vec<_>>().join("\n"))
            .unwrap_or_default()
    }

    /// Quantas linhas a cauda guarda no maximo.
    #[must_use]
    pub const fn capacidade(&self) -> usize {
        self.capacidade
    }

    /// `message` com a cauda anexada sob um titulo — ou `message` intacta
    /// quando o filho nao escreveu nada (um erro sem stderr nao ganha um
    /// cabecalho vazio).
    #[must_use]
    pub fn anexar(&self, message: &str, titulo: &str) -> String {
        let cauda = self.tail();
        if cauda.is_empty() {
            return message.to_owned();
        }
        format!("{message}\n--- {titulo} ---\n{cauda}")
    }
}

/// Sem o `\n`/`\r\n` final, UTF-8 com perda, e cortada em `LINHA_MAXIMA`.
fn limpar(bruto: &[u8]) -> String {
    let sem_fim = bruto
        .strip_suffix(b"\n")
        .map_or(bruto, |b| b.strip_suffix(b"\r").unwrap_or(b));
    let texto = String::from_utf8_lossy(sem_fim);
    if texto.len() <= LINHA_MAXIMA {
        return texto.into_owned();
    }
    let mut corte = LINHA_MAXIMA;
    while !texto.is_char_boundary(corte) {
        corte -= 1;
    }
    format!(
        "{}… [linha cortada em {LINHA_MAXIMA} bytes]",
        &texto[..corte]
    )
}

#[cfg(test)]
mod tests {
    use std::{
        io::Write,
        sync::{Arc, Mutex, mpsc},
        time::Duration,
    };

    use super::*;

    /// Um pipe de verdade (os.pipe), para a thread ler como leria do filho.
    fn pipe() -> (std::io::PipeReader, std::io::PipeWriter) {
        std::io::pipe().unwrap()
    }

    fn esperar(condicao: impl Fn() -> bool) {
        let inicio = std::time::Instant::now();
        while !condicao() {
            assert!(
                inicio.elapsed() < Duration::from_secs(5),
                "a thread leitora nao entregou a tempo"
            );
            thread::sleep(Duration::from_millis(5));
        }
    }

    #[test]
    fn keeps_only_the_last_lines_and_forwards_each_one() {
        let (leitura, mut escrita) = pipe();
        let (sender, receiver) = mpsc::channel();
        let cauda = StderrTail::spawn(
            leitura,
            3,
            Some(Box::new(move |linha: &str| {
                drop(sender.send(linha.to_owned()));
            })),
        );
        for i in 1..=5 {
            writeln!(escrita, "linha {i}").unwrap();
        }
        drop(escrita);
        let vistas: Vec<String> = (0..5)
            .map(|_| receiver.recv_timeout(Duration::from_secs(5)).unwrap())
            .collect();
        assert_eq!(
            vistas,
            ["linha 1", "linha 2", "linha 3", "linha 4", "linha 5"]
        );
        esperar(|| cauda.tail().ends_with("linha 5"));
        assert_eq!(cauda.tail(), "linha 3\nlinha 4\nlinha 5");
        assert_eq!(cauda.capacidade(), 3);
    }

    #[test]
    fn empty_tail_leaves_the_message_alone() {
        let (leitura, escrita) = pipe();
        let cauda = StderrTail::spawn(leitura, CAPACIDADE_PADRAO, None);
        drop(escrita);
        assert_eq!(cauda.anexar("falhou", "stderr"), "falhou");
        assert_eq!(cauda.tail(), "");
    }

    #[test]
    fn attaches_the_tail_under_a_title() {
        let (leitura, mut escrita) = pipe();
        let cauda = StderrTail::spawn(leitura, CAPACIDADE_PADRAO, None);
        write!(escrita, "Traceback\r\nModuleNotFoundError: x\n").unwrap();
        drop(escrita);
        esperar(|| cauda.tail().contains("ModuleNotFound"));
        assert_eq!(
            cauda.anexar("o adapter nao respondeu", "stderr do adaptador"),
            "o adapter nao respondeu\n--- stderr do adaptador ---\nTraceback\nModuleNotFoundError: x"
        );
    }

    #[test]
    fn invalid_utf8_and_giant_lines_do_not_stop_the_reader() {
        let (leitura, mut escrita) = pipe();
        let vistas = Arc::new(Mutex::new(Vec::new()));
        let dono = Arc::clone(&vistas);
        let cauda = StderrTail::spawn(
            leitura,
            CAPACIDADE_PADRAO,
            Some(Box::new(move |linha: &str| {
                dono.lock().unwrap().push(linha.len());
            })),
        );
        escrita.write_all(b"ok \xff\xfe fim\n").unwrap();
        escrita.write_all(&vec![b'a'; 10_000]).unwrap();
        escrita.write_all(b"\ndepois\n").unwrap();
        drop(escrita);
        esperar(|| cauda.tail().ends_with("depois"));
        let linhas: Vec<String> = cauda.tail().lines().map(str::to_owned).collect();
        assert_eq!(linhas.len(), 3);
        assert!(linhas[0].contains('\u{FFFD}'), "{:?}", linhas[0]);
        assert!(linhas[1].ends_with("[linha cortada em 4096 bytes]"));
        assert!(linhas[1].len() < 4200);
        assert_eq!(linhas[2], "depois");
        assert_eq!(vistas.lock().unwrap().len(), 3);
    }
}
