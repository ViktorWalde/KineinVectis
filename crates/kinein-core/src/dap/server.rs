//! O processo SERVIDOR do alvo remoto: QEMU, `OpenOCD`, `pyOCD`, `probe-rs gdb`.
//!
//! Sobe ANTES do adaptador, espera a porta abrir, e morre com a sessao. O
//! comando e' DECLARADO pelo usuario no kit (`debugServer`), nunca deduzido: a
//! IDE nao sabe qual maquina do QEMU e' a placa dele, nem qual `.cfg` do
//! `OpenOCD` (`roadmaps/35` §5.1: nada adivinhado). `{program}` e' o unico
//! ponto de substituicao, e ele vira o ELF resolvido pelo `debug.start`.
//!
//! Por que `sh -c "exec ..."` e nao `sh -c "..."`: o run domain sobe comando
//! do usuario por `sh -c`, e aqui e' o mesmo contrato — so' que este processo
//! precisa MORRER quando a sessao acaba, e matar o `sh` nao mata o filho dele.
//! O `exec` faz o pid ser o do servidor — e por isso o comando tem de ser um
//! EXECUTAVEL com argumentos (o que QEMU, `OpenOCD` e `pyOCD` sao), nao um
//! pipeline nem um builtin do shell. Medido em 2026-09-11: o QEMU
//! (`lm3s6965evb -S -gdb tcp::3333`) abre a porta em 0,05 s.

use std::{
    net::{TcpStream, ToSocketAddrs},
    path::Path,
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use super::DebugError;

/// Quanto se espera pela porta do servidor antes de desistir.
///
/// O QEMU abre em 50 ms; o `OpenOCD` com uma sonda real leva ate' uns 2 s para
/// enumerar o USB. Cinco segundos cobre os dois com folga e ainda e' curto o
/// bastante para "a porta nunca abriu" virar erro em vez de espera eterna.
const PORT_TIMEOUT: Duration = Duration::from_secs(5);
const PORT_POLL: Duration = Duration::from_millis(50);

/// Um servidor vivo; `Drop` o mata.
#[derive(Debug)]
pub(super) struct DebugServer {
    child: Child,
}

impl DebugServer {
    /// Sobe `template` (com `{program}` substituido) em `root` e espera
    /// `remote_target` aceitar conexao.
    pub(super) fn spawn(
        root: &Path,
        template: &str,
        program: &Path,
        remote_target: &str,
    ) -> Result<Self, DebugError> {
        let command = render_command(template, program);
        let child = Command::new("sh")
            .arg("-c")
            .arg(format!("exec {command}"))
            .current_dir(root)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| DebugError::Adapter {
                message: format!("falha ao subir o servidor de debug `{command}`: {error}"),
            })?;
        let mut server = Self { child };
        if let Err(error) = server.wait_for_port(remote_target) {
            // O erro ja' explica; o processo nao pode ficar orfao.
            drop(server.child.kill());
            return Err(error);
        }
        Ok(server)
    }

    /// Espera `host:porta` aceitar TCP, ou o processo morrer antes disso.
    fn wait_for_port(&mut self, remote_target: &str) -> Result<(), DebugError> {
        let address = socket_address(remote_target)?;
        let started = Instant::now();
        loop {
            if TcpStream::connect_timeout(&address, PORT_POLL).is_ok() {
                return Ok(());
            }
            if let Ok(Some(status)) = self.child.try_wait() {
                return Err(DebugError::Adapter {
                    message: format!(
                        "o servidor de debug saiu ({status}) antes de abrir {remote_target}"
                    ),
                });
            }
            if started.elapsed() >= PORT_TIMEOUT {
                return Err(DebugError::Adapter {
                    message: format!(
                        "o servidor de debug nao abriu {remote_target} em {} s",
                        PORT_TIMEOUT.as_secs()
                    ),
                });
            }
            thread::sleep(PORT_POLL);
        }
    }
}

impl Drop for DebugServer {
    fn drop(&mut self) {
        drop(self.child.kill());
        drop(self.child.wait());
    }
}

/// `host:porta` -> endereco. O texto vai cru para o `target remote` do GDB,
/// mas para ESPERAR a porta e' preciso entende-lo — e um alvo que nao e'
/// `host:porta` (um `/dev/ttyUSB0` serial, por exemplo) nao tem porta para
/// esperar, entao e' recusado com o motivo.
pub(super) fn socket_address(remote_target: &str) -> Result<std::net::SocketAddr, DebugError> {
    remote_target
        .to_socket_addrs()
        .ok()
        .and_then(|mut addresses| addresses.next())
        .ok_or_else(|| DebugError::Adapter {
            message: format!(
                "remoteTarget `{remote_target}` nao e' host:porta; e' o que o servidor de \
                 debug precisa para a IDE esperar a porta abrir"
            ),
        })
}

/// O unico ponto de substituicao do comando: o ELF resolvido.
///
/// Nao e' `format!`: e' texto do usuario, e o clippy tem razao em suspeitar
/// de chaves fora de macro — por isso o marcador mora numa constante.
const PROGRAM_PLACEHOLDER: &str = "{program}";

/// Substitui o marcador pelo ELF, entre aspas simples para o `sh`.
pub(super) fn render_command(template: &str, program: &Path) -> String {
    template.replace(
        PROGRAM_PLACEHOLDER,
        &shell_quote(&program.display().to_string()),
    )
}

/// Aspas simples POSIX: a unica coisa que precisa de escape dentro delas e' a
/// propria aspa simples.
fn shell_quote(text: &str) -> String {
    format!("'{}'", text.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use std::{net::TcpListener, path::Path};

    use super::{DebugServer, render_command, socket_address};

    /// `{program}` vira o ELF entre aspas — um caminho com espaco nao pode
    /// virar dois argumentos do QEMU.
    #[test]
    fn program_is_substituted_and_quoted() {
        let rendered = render_command(
            "qemu-system-arm -kernel {program} -S",
            Path::new("/tmp/meu projeto/fw.elf"),
        );
        assert_eq!(
            rendered,
            "qemu-system-arm -kernel '/tmp/meu projeto/fw.elf' -S"
        );
        assert_eq!(
            render_command("x {program}", Path::new("a'b")),
            "x 'a'\\''b'",
            "aspa simples dentro do caminho tem de ser escapada"
        );
    }

    /// Alvo que nao e' host:porta e' recusado com o motivo, nao esperado
    /// para sempre.
    #[test]
    fn remote_target_must_be_host_and_port() {
        assert!(socket_address("localhost:3333").is_ok());
        let erro = socket_address("/dev/ttyUSB0").unwrap_err().to_string();
        assert!(erro.contains("host:porta"), "{erro}");
    }

    /// O servidor que MORRE antes de abrir a porta vira erro na hora, com o
    /// status — nao cinco segundos de espera por uma porta que nunca vem.
    #[test]
    fn server_that_exits_early_is_reported_with_its_status() {
        let erro = DebugServer::spawn(
            Path::new("/tmp"),
            "sh -c 'exit 3'",
            Path::new("fw.elf"),
            "127.0.0.1:1",
        )
        .unwrap_err()
        .to_string();
        assert!(erro.contains("saiu") && erro.contains('3'), "{erro}");
    }

    /// Porta ja' aberta: a espera termina no primeiro connect. Aqui o
    /// "servidor" e' um `sleep` e quem abre a porta e' o teste — o que se
    /// prova e' a espera, nao o QEMU.
    #[test]
    fn open_port_ends_the_wait() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = DebugServer::spawn(
            Path::new("/tmp"),
            "sleep 30",
            Path::new("fw.elf"),
            &address.to_string(),
        );
        let server = server.unwrap();
        let pid = server.child.id();
        // `Drop` MATA o servidor e volta rapido. A mutacao que importa e' tirar
        // o `kill`: ai o `wait` bloqueia nos 30 s do `sleep`, entao o que
        // distingue as duas versoes e' o TEMPO do drop. Essencial porque o
        // QEMU se mata sozinho no pacote `k` do GDB, mas o OpenOCD NAO — sem
        // este kill, cada sessao deixaria um servidor vivo na porta.
        let inicio = std::time::Instant::now();
        drop(server);
        let decorrido = inicio.elapsed();
        drop(listener);
        assert!(
            decorrido < std::time::Duration::from_secs(5),
            "o Drop levou {decorrido:?}: sem o kill, ele espera o servidor sair sozinho"
        );
        std::thread::sleep(std::time::Duration::from_millis(100));
        let vivo = std::process::Command::new("kill")
            .args(["-0", &pid.to_string()])
            .status()
            .unwrap()
            .success();
        assert!(!vivo, "o servidor (pid {pid}) sobreviveu ao Drop da sessao");
    }
}
