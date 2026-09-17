//! O provedor de instalacao de toolchain (`integracoes/39` §5).
//!
//! Baixa um release PINADO do catalogo para a pasta da IDE, confere o SHA-256
//! que a fonte publicou ANTES de desempacotar, e desempacota com o `tar` do
//! sistema (GPL, processo). Nunca sem clique; nunca no sistema; nunca sem
//! checksum; nunca "latest".
//!
//! ```text
//! <raiz>/<id>/<versao>.part/       onde o tarball e a extracao nascem
//! <raiz>/<id>/<versao>/bin         o que o detector le (tools/search_dirs)
//! ```
//!
//! A pasta final so' existe depois de o checksum bater e o `tar` sair com 0:
//! uma instalacao interrompida (cancelamento, rede, disco) deixa no maximo um
//! `.part`, que a proxima tentativa apaga. O `--strip-components=1` tira a
//! pasta de topo que todos os tarballs do catalogo trazem
//! (`arm-gnu-toolchain-15.2.rel1-x86_64-arm-none-eabi/`, `xpack-.../`,
//! `ATfE-.../`, `aarch64--glibc--stable-.../`), para o `bin` ficar onde o
//! detector procura.

pub mod catalog;
pub mod firmware;

use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use sha2::{Digest, Sha256};

pub use catalog::{CATALOGO, Entrada, Firmware, Kind, entrada};
pub use firmware::FIRMWARE;

/// Tempo maximo sem um byte novo (nao o total: um tarball de 400 MB numa
/// rede lenta leva o que levar, mas uma conexao muda nao pode prender o job).
const PRAZO_ENTRE_BYTES: Duration = Duration::from_secs(60);
/// Bloco de leitura do corpo.
const BLOCO: usize = 256 * 1024;

/// O que o job conta enquanto instala.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallEvent {
    /// Comecou a baixar: a URL e o tamanho esperado.
    Downloading {
        /// URL exata.
        url: String,
        /// Tamanho anunciado pelo catalogo.
        expected_bytes: u64,
    },
    /// Bytes recebidos ate agora.
    Progress {
        /// Recebidos.
        received: u64,
        /// Total anunciado pelo servidor (ou pelo catalogo).
        total: u64,
    },
    /// O SHA-256 bateu com o publicado.
    Verified {
        /// O digest, para a saida.
        sha256: String,
    },
    /// Desempacotando com o `tar` do sistema.
    Extracting {
        /// A linha de comando, para a saida.
        command: String,
    },
}

/// Por que uma instalacao falhou — sempre com o proximo passo.
#[derive(Debug)]
pub enum InstallError {
    /// A pasta de destino ja' tem `bin/`: nao se sobrescreve uma toolchain.
    AlreadyInstalled(PathBuf),
    /// Rede ou HTTP.
    Download(String),
    /// O digest do arquivo baixado nao e' o publicado.
    Checksum {
        /// O publicado no catalogo.
        expected: String,
        /// O que o arquivo baixado tem.
        actual: String,
    },
    /// `tar` ausente ou saiu com erro.
    Extract(String),
    /// Disco.
    Io(io::Error),
    /// `job.cancel`.
    Cancelled,
}

impl std::fmt::Display for InstallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyInstalled(p) => write!(
                f,
                "ja' existe uma toolchain em {} — apague a pasta para reinstalar",
                p.display()
            ),
            Self::Download(m) => write!(f, "download falhou: {m}"),
            Self::Checksum { expected, actual } => write!(
                f,
                "o SHA-256 do arquivo baixado ({actual}) NAO e' o publicado pela fonte ({expected}); \
                 nada foi desempacotado — o release pode ter mudado ou o download veio corrompido"
            ),
            Self::Extract(m) => write!(f, "desempacotar falhou: {m}"),
            Self::Io(e) => write!(f, "disco: {e}"),
            Self::Cancelled => write!(f, "instalacao cancelada; nada ficou fora do .part"),
        }
    }
}

impl std::error::Error for InstallError {}

impl From<io::Error> for InstallError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

/// A pasta final de uma entrada sob `raiz`.
#[must_use]
pub fn install_dir(raiz: &Path, e: &Entrada) -> PathBuf {
    raiz.join(e.id).join(e.version)
}

/// O que o detector (toolchain) ou o motor de gravar (firmware) procura:
/// `<pasta>/bin` ou `<pasta>/<arquivo da URL>`.
#[must_use]
pub fn installed_path(raiz: &Path, e: &Entrada) -> PathBuf {
    match e.kind {
        Kind::Toolchain => install_dir(raiz, e).join("bin"),
        Kind::Firmware => install_dir(raiz, e).join(firmware::file_name(e)),
    }
}

/// `true` quando `<raiz>/<id>/<versao>/bin` existe (toolchain) ou o arquivo
/// do firmware esta' na pasta.
#[must_use]
pub fn is_installed(raiz: &Path, e: &Entrada) -> bool {
    match e.kind {
        Kind::Toolchain => installed_path(raiz, e).is_dir(),
        Kind::Firmware => installed_path(raiz, e).is_file(),
    }
}

/// Instala `e` sob `raiz`: baixa, confere, desempacota.
///
/// Uma toolchain e' desempacotada pelo `tar` (o do PATH, ou um falso no
/// teste); um firmware fica inteiro na pasta e `tar` pode ser `None`. O
/// `sink` recebe o andamento; `cancel` e' o do job.
pub fn install(
    e: &Entrada,
    raiz: &Path,
    tar: Option<&Path>,
    cancel: &Arc<AtomicBool>,
    sink: &mut dyn FnMut(InstallEvent),
) -> Result<PathBuf, InstallError> {
    let destino = install_dir(raiz, e);
    if is_installed(raiz, e) {
        return Err(InstallError::AlreadyInstalled(destino));
    }
    let parcial = raiz.join(e.id).join(format!("{}.part", e.version));
    let _ = fs::remove_dir_all(&parcial);
    fs::create_dir_all(&parcial)?;
    let nome = e.url.rsplit('/').next().unwrap_or("toolchain.tar");
    let arquivo = parcial.join(nome);

    let resultado = (|| {
        sink(InstallEvent::Downloading {
            url: e.url.to_owned(),
            expected_bytes: e.size_bytes,
        });
        let digest = download(e.url, e.size_bytes, &arquivo, cancel, sink)?;
        if digest != e.sha256 {
            return Err(InstallError::Checksum {
                expected: e.sha256.to_owned(),
                actual: digest,
            });
        }
        sink(InstallEvent::Verified { sha256: digest });
        if e.kind == Kind::Firmware {
            // O firmware e' um arquivo so': a pasta final nasce com ele
            // dentro, e nada e' desempacotado.
            fs::create_dir_all(&destino)?;
            fs::rename(&arquivo, destino.join(nome))?;
            return Ok(destino.clone());
        }
        let tar = tar.ok_or_else(|| {
            InstallError::Extract("sem `tar` para desempacotar a toolchain".to_owned())
        })?;
        let extraido = parcial.join("extraido");
        fs::create_dir_all(&extraido)?;
        extract(tar, &arquivo, &extraido, sink)?;
        if !extraido.join("bin").is_dir() {
            return Err(InstallError::Extract(
                "o tarball nao trouxe uma pasta bin/ no topo (depois de tirar a pasta de topo)"
                    .to_owned(),
            ));
        }
        // O tarball fica no `.part`, que cai inteiro logo abaixo.
        fs::rename(&extraido, &destino)?;
        Ok(destino.clone())
    })();
    let _ = fs::remove_dir_all(&parcial);
    resultado
}

/// Baixa `url` para `arquivo`, calculando o SHA-256 no caminho; devolve o
/// digest em hexadecimal minusculo.
fn download(
    url: &str,
    esperado: u64,
    arquivo: &Path,
    cancel: &Arc<AtomicBool>,
    sink: &mut dyn FnMut(InstallEvent),
) -> Result<String, InstallError> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_recv_body(Some(PRAZO_ENTRE_BYTES))
        .timeout_connect(Some(Duration::from_secs(30)))
        .http_status_as_error(false)
        .build()
        .into();
    let mut resposta = agent
        .get(url)
        .call()
        .map_err(|erro| InstallError::Download(descrever(&erro.to_string(), url)))?;
    let status = resposta.status().as_u16();
    if status != 200 {
        return Err(InstallError::Download(format!(
            "{url} respondeu HTTP {status}"
        )));
    }
    let total = resposta.body().content_length().unwrap_or(esperado);
    let mut leitor = resposta.body_mut().with_config().reader();
    let mut saida = File::create(arquivo)?;
    let mut hasher = Sha256::new();
    let mut recebidos: u64 = 0;
    let mut bloco = vec![0u8; BLOCO];
    loop {
        if cancel.load(Ordering::SeqCst) {
            return Err(InstallError::Cancelled);
        }
        let n = leitor
            .read(&mut bloco)
            .map_err(|erro| InstallError::Download(format!("a conexao caiu: {erro}")))?;
        if n == 0 {
            break;
        }
        saida.write_all(&bloco[..n])?;
        hasher.update(&bloco[..n]);
        recebidos += n as u64;
        sink(InstallEvent::Progress {
            received: recebidos,
            total,
        });
    }
    saida.flush()?;
    Ok(hex(&hasher.finalize()))
}

/// Hexadecimal minusculo, como os arquivos `.sha256` das fontes escrevem.
fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    bytes.iter().fold(String::with_capacity(64), |mut s, b| {
        let _ = write!(s, "{b:02x}");
        s
    })
}

/// `tar -xf <arquivo> --strip-components=1 -C <pasta>`: o `tar` do sistema
/// reconhece xz e gz pelo conteudo.
fn extract(
    tar: &Path,
    arquivo: &Path,
    pasta: &Path,
    sink: &mut dyn FnMut(InstallEvent),
) -> Result<(), InstallError> {
    let mut comando = Command::new(tar);
    comando
        .arg("-xf")
        .arg(arquivo)
        .arg("--strip-components=1")
        .arg("-C")
        .arg(pasta);
    sink(InstallEvent::Extracting {
        command: format!(
            "{} -xf {} --strip-components=1 -C {}",
            tar.display(),
            arquivo.display(),
            pasta.display()
        ),
    });
    let saida = comando.output().map_err(|erro| {
        InstallError::Extract(format!("nao consegui executar {}: {erro}", tar.display()))
    })?;
    if !saida.status.success() {
        return Err(InstallError::Extract(format!(
            "{} saiu com {}: {}",
            tar.display(),
            saida.status,
            String::from_utf8_lossy(&saida.stderr).trim()
        )));
    }
    Ok(())
}

/// A falha de rede em palavras que dizem o proximo passo.
fn descrever(erro: &str, url: &str) -> String {
    let baixo = erro.to_lowercase();
    if baixo.contains("dns") || baixo.contains("resolve") || baixo.contains("name") {
        return format!("nao resolvi o nome de {url} — sem rede, ou sem DNS?");
    }
    if baixo.contains("connection refused") || baixo.contains("timed out") {
        return format!("nao alcancei {url} ({erro})");
    }
    format!("{url}: {erro}")
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;

    use super::{Entrada, InstallError, InstallEvent, Kind, install, is_installed};

    fn pasta(nome: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("kinein-core-tests")
            .join(format!("{}-install-{nome}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Um tarball .tar.xz REAL (feito pelo `tar` do sistema) com a forma das
    /// fontes: uma pasta de topo, e `bin/` dentro dela.
    fn tarball(dir: &Path) -> (PathBuf, String, u64) {
        let topo = dir.join("arm-gnu-toolchain-1.0-x86_64-arm-none-eabi");
        std::fs::create_dir_all(topo.join("bin")).unwrap();
        std::fs::write(topo.join("bin/arm-none-eabi-gcc"), "#!/bin/sh\necho gcc\n").unwrap();
        std::fs::write(topo.join("README"), "toolchain de teste\n").unwrap();
        let arquivo = dir.join("toolchain.tar.xz");
        let saida = std::process::Command::new("tar")
            .arg("-cJf")
            .arg(&arquivo)
            .arg("-C")
            .arg(dir)
            .arg(topo.file_name().unwrap())
            .output()
            .expect("tar do sistema");
        assert!(
            saida.status.success(),
            "{}",
            String::from_utf8_lossy(&saida.stderr)
        );
        let bytes = std::fs::read(&arquivo).unwrap();
        let digest = super::hex(&<sha2::Sha256 as sha2::Digest>::digest(&bytes));
        (arquivo, digest, bytes.len() as u64)
    }

    /// Um servidor HTTP minimo numa porta livre: responde UMA requisicao com
    /// o arquivo (200 + Content-Length) ou com 404. Real o bastante para o
    /// ureq de verdade falar com ele.
    fn servir(arquivo: PathBuf, status: u16) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let porta = listener.local_addr().unwrap().port();
        std::thread::spawn(move || {
            let (mut conexao, _) = listener.accept().unwrap();
            let mut pedido = [0u8; 4096];
            let _ = conexao.read(&mut pedido);
            let corpo = if status == 200 {
                std::fs::read(&arquivo).unwrap()
            } else {
                Vec::new()
            };
            let cabecalho = format!(
                "HTTP/1.1 {status} {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                if status == 200 { "OK" } else { "Not Found" },
                corpo.len()
            );
            conexao.write_all(cabecalho.as_bytes()).unwrap();
            conexao.write_all(&corpo).unwrap();
            let _ = conexao.flush();
        });
        format!("http://127.0.0.1:{porta}/toolchain.tar.xz")
    }

    /// Um firmware e' um ARQUIVO SO' (C5): baixado, conferido e guardado
    /// inteiro na pasta — sem `tar`, sem `bin/`. Checksum errado nao deixa
    /// nada; instalar duas vezes e' recusado pelo arquivo existente.
    #[test]
    fn a_firmware_is_a_single_verified_file_and_needs_no_tar() {
        let dir = pasta("firmware");
        let bin = dir.join("ESP32_GENERIC-20260824-v1.29.0.bin");
        let bytes: Vec<u8> = (0..70_000u32).map(|i| (i % 251) as u8).collect();
        std::fs::write(&bin, &bytes).unwrap();
        let digest = super::hex(&<sha2::Sha256 as sha2::Digest>::digest(&bytes));
        // O nome do arquivo vem da URL: o servidor de teste serve o mesmo
        // corpo em qualquer caminho.
        let url =
            servir(bin, 200).replace("toolchain.tar.xz", "ESP32_GENERIC-20260824-v1.29.0.bin");
        let mut e = entrada(vazar(url), vazar(digest.clone()), bytes.len() as u64);
        e.kind = Kind::Firmware;
        e.firmware = Some(super::Firmware {
            board: "ESP32_GENERIC",
            engine: "esptool",
            offset: Some("0x1000"),
            chip: Some("esp32"),
        });
        let raiz = dir.join("toolchains");
        let cancel = Arc::new(AtomicBool::new(false));
        let mut eventos = Vec::new();
        let destino = install(&e, &raiz, None, &cancel, &mut |ev| eventos.push(ev)).unwrap();
        assert_eq!(destino, raiz.join("teste-arm-none-eabi/1.0"));
        let arquivo = super::installed_path(&raiz, &e);
        assert_eq!(arquivo, destino.join("ESP32_GENERIC-20260824-v1.29.0.bin"));
        assert!(arquivo.is_file() && is_installed(&raiz, &e));
        assert_eq!(std::fs::read(&arquivo).unwrap(), bytes);
        assert!(!destino.join("bin").exists());
        assert!(!raiz.join("teste-arm-none-eabi/1.0.part").exists());
        assert!(
            eventos
                .iter()
                .any(|ev| matches!(ev, InstallEvent::Verified { sha256 } if *sha256 == digest))
        );
        assert!(
            !eventos
                .iter()
                .any(|ev| matches!(ev, InstallEvent::Extracting { .. }))
        );
        assert!(matches!(
            install(&e, &raiz, None, &cancel, &mut |_| {}).unwrap_err(),
            InstallError::AlreadyInstalled(_)
        ));
    }

    fn entrada(url: &'static str, sha256: &'static str, size: u64) -> Entrada {
        Entrada {
            kind: Kind::Toolchain,
            firmware: None,
            id: "teste-arm-none-eabi",
            label: "toolchain de teste",
            version: "1.0",
            family: "cortex-m",
            url,
            size_bytes: size,
            sha256,
            license: "teste",
            source: "teste",
        }
    }

    fn vazar(s: String) -> &'static str {
        Box::leak(s.into_boxed_str())
    }

    /// O caminho feliz, de ponta a ponta com as pecas REAIS: HTTP pelo ureq,
    /// SHA-256 conferido, `tar` do sistema com --strip-components=1, e o
    /// `bin/` onde o detector procura. Nenhum `.part` sobra.
    #[test]
    fn a_real_download_is_verified_extracted_and_lands_where_the_detector_looks() {
        let dir = pasta("feliz");
        let (arquivo, digest, tamanho) = tarball(&dir);
        let url = servir(arquivo, 200);
        let e = entrada(vazar(url.clone()), vazar(digest.clone()), tamanho);
        let raiz = dir.join("toolchains");
        let cancel = Arc::new(AtomicBool::new(false));
        let mut eventos = Vec::new();
        assert!(!is_installed(&raiz, &e));
        let destino = install(&e, &raiz, Some(Path::new("tar")), &cancel, &mut |ev| {
            eventos.push(ev);
        })
        .unwrap();
        assert_eq!(destino, raiz.join("teste-arm-none-eabi/1.0"));
        assert!(
            destino.join("bin/arm-none-eabi-gcc").is_file(),
            "sem a pasta de topo"
        );
        assert!(destino.join("README").is_file());
        assert!(is_installed(&raiz, &e));
        assert!(
            !raiz.join("teste-arm-none-eabi/1.0.part").exists(),
            "o .part (e o tarball dentro dele) sumiu"
        );

        assert!(
            matches!(&eventos[0], InstallEvent::Downloading { url: u, expected_bytes } if *u == url && *expected_bytes == tamanho)
        );
        let recebido = eventos
            .iter()
            .filter_map(|e| match e {
                InstallEvent::Progress { received, total } => Some((*received, *total)),
                _ => None,
            })
            .next_back()
            .unwrap();
        assert_eq!(
            recebido,
            (tamanho, tamanho),
            "o progresso chega ao total anunciado"
        );
        assert!(
            eventos
                .iter()
                .any(|e| matches!(e, InstallEvent::Verified { sha256 } if *sha256 == digest))
        );
        assert!(eventos.iter().any(|e| matches!(e, InstallEvent::Extracting { command } if command.contains("--strip-components=1"))));

        // Instalar de novo: recusa, sem tocar no que existe.
        let erro = install(&e, &raiz, Some(Path::new("tar")), &cancel, &mut |_| {}).unwrap_err();
        assert!(matches!(erro, InstallError::AlreadyInstalled(_)), "{erro}");
    }

    /// O checksum errado e' a REGRA do provedor: nada e' desempacotado, o
    /// erro mostra os dois digests, e nao sobra pasta final nem `.part`.
    #[test]
    fn a_checksum_mismatch_extracts_nothing_and_names_both_digests() {
        let dir = pasta("checksum");
        let (arquivo, digest, tamanho) = tarball(&dir);
        let url = servir(arquivo, 200);
        let errado = "0".repeat(64);
        let e = entrada(vazar(url), vazar(errado.clone()), tamanho);
        let raiz = dir.join("toolchains");
        let cancel = Arc::new(AtomicBool::new(false));
        let mut eventos = Vec::new();
        let erro = install(&e, &raiz, Some(Path::new("tar")), &cancel, &mut |ev| {
            eventos.push(ev);
        })
        .unwrap_err();
        match &erro {
            InstallError::Checksum { expected, actual } => {
                assert_eq!(expected, &errado);
                assert_eq!(actual, &digest);
            }
            outro => panic!("esperava Checksum, veio {outro}"),
        }
        assert!(
            erro.to_string().contains(&digest)
                && erro.to_string().contains("nada foi desempacotado")
        );
        assert!(
            !eventos
                .iter()
                .any(|e| matches!(e, InstallEvent::Extracting { .. })),
            "nao chamou o tar"
        );
        assert!(!raiz.join("teste-arm-none-eabi/1.0").exists());
        assert!(!raiz.join("teste-arm-none-eabi/1.0.part").exists());
    }

    /// Um tarball cuja pasta de topo NAO tem `bin/` nao vira instalacao: o
    /// detector nao a acharia e a tela diria "instalada" de uma pasta inutil.
    #[test]
    fn a_tarball_without_a_bin_folder_is_refused_after_extraction() {
        let dir = pasta("sem-bin");
        let topo = dir.join("sdk-1.0");
        std::fs::create_dir_all(topo.join("share")).unwrap();
        std::fs::write(topo.join("share/README"), "nada de bin\n").unwrap();
        let arquivo = dir.join("sdk.tar.xz");
        let saida = std::process::Command::new("tar")
            .arg("-cJf")
            .arg(&arquivo)
            .arg("-C")
            .arg(&dir)
            .arg("sdk-1.0")
            .output()
            .unwrap();
        assert!(saida.status.success());
        let bytes = std::fs::read(&arquivo).unwrap();
        let digest = super::hex(&<sha2::Sha256 as sha2::Digest>::digest(&bytes));
        let url = servir(arquivo, 200);
        let e = entrada(vazar(url), vazar(digest), bytes.len() as u64);
        let raiz = dir.join("toolchains");
        let erro = install(
            &e,
            &raiz,
            Some(Path::new("tar")),
            &Arc::new(AtomicBool::new(false)),
            &mut |_| {},
        )
        .unwrap_err();
        assert!(
            matches!(&erro, InstallError::Extract(m) if m.contains("bin/")),
            "{erro}"
        );
        assert!(!raiz.join("teste-arm-none-eabi/1.0").exists());
        assert!(!raiz.join("teste-arm-none-eabi/1.0.part").exists());
    }

    /// HTTP que nao e' 200 e' erro que diz o status; cancelamento no meio nao
    /// deixa pasta final; um `tar` que falha (ou nao existe) e' erro de
    /// extracao, e o tarball baixado nao vira instalacao pela metade.
    #[test]
    fn http_errors_cancellation_and_a_broken_tar_leave_nothing_behind() {
        let dir = pasta("erros");
        let (arquivo, digest, tamanho) = tarball(&dir);
        let raiz = dir.join("toolchains");

        let url = servir(arquivo.clone(), 404);
        let e = entrada(vazar(url), vazar(digest.clone()), tamanho);
        let erro = install(
            &e,
            &raiz,
            Some(Path::new("tar")),
            &Arc::new(AtomicBool::new(false)),
            &mut |_| {},
        )
        .unwrap_err();
        assert!(
            matches!(&erro, InstallError::Download(m) if m.contains("HTTP 404")),
            "{erro}"
        );

        let url = servir(arquivo.clone(), 200);
        let e = entrada(vazar(url), vazar(digest.clone()), tamanho);
        let cancelado = Arc::new(AtomicBool::new(true));
        let erro = install(&e, &raiz, Some(Path::new("tar")), &cancelado, &mut |_| {}).unwrap_err();
        assert!(matches!(erro, InstallError::Cancelled));
        assert!(!raiz.join("teste-arm-none-eabi/1.0").exists());

        let url = servir(arquivo, 200);
        let e = entrada(vazar(url), vazar(digest), tamanho);
        let erro = install(
            &e,
            &raiz,
            Some(Path::new("/nao/existe/tar")),
            &Arc::new(AtomicBool::new(false)),
            &mut |_| {},
        )
        .unwrap_err();
        assert!(
            matches!(&erro, InstallError::Extract(m) if m.contains("nao consegui executar")),
            "{erro}"
        );
        assert!(!raiz.join("teste-arm-none-eabi/1.0").exists());
        assert!(!raiz.join("teste-arm-none-eabi/1.0.part").exists());
    }
}
