//! O alvo Linux por SSH como recurso do projeto (P6 do `roadmaps/42`, fatia
//! 1, 2026-09-17): a Raspberry Pi, a placa com imagem propria.
//!
//! Tudo aqui e' o `ssh`/`rsync`/`scp` do SISTEMA como processo (OpenSSH BSD,
//! rsync GPL-3 — nunca crate). Esta unidade e' PURA: valida perfis, compoe
//! linhas de comando e LE a saida do probe; quem roda e' o job do handler.
//!
//! ```text
//! probe    ssh -o BatchMode=yes -o ConnectTimeout=5 [-p P] [-i K] [user@]host
//!              'uname -m; uname -sr; for t in gdbserver python3 rsync; do
//!               printf "%s=" $t; command -v $t || echo; done'
//!          BatchMode: sem chave nao pergunta senha — falha rapido e a IDE diz
//!          `ssh-copy-id`; a chave e' do usuario, nunca da IDE
//! deploy   rsync -az --delete -e 'ssh [-p P] [-i K]' <origem> [user@]host:<dest>/
//!          (rsync nos dois lados; sem ele no alvo, o probe diz e cai no scp)
//!          scp [-P P] [-i K] -r <origem> [user@]host:<dest>/
//! rodar    ssh [-p P] [-i K] [user@]host '<dest>/<binario>'
//! gdb      ssh … [user@]host 'gdbserver :2345 <dest>/<binario>'   remoteTarget host:2345
//! debugpy  ssh … [user@]host 'python3 -m debugpy --listen 0.0.0.0:5678 --wait-for-client <script>'
//! ```
//!
//! O que NAO entra nesta fatia (dito): workspace remoto, LSP do outro lado,
//! mapeamento de caminhos — o `RemoteContext` inteiro do 28 §4.

pub mod discover;
pub mod mirror;
mod store;

use std::path::Path;

use kinein_protocol::{RemoteCommandKind, RemoteTarget, RemoteTool};

/// Porta padrao do `gdbserver`.
pub const GDBSERVER_PORT: u16 = 2345;
/// Porta padrao do `debugpy --listen`.
pub const DEBUGPY_PORT: u16 = 5678;
/// O que o probe pergunta ao alvo (`command -v`).
pub const TOOLS: [&str; 3] = ["gdbserver", "python3", "rsync"];

/// Os alvos, por nome.
#[must_use]
pub fn list(root: &Path) -> Vec<RemoteTarget> {
    let mut targets = store::load(root);
    targets.sort_by(|a, b| a.name.cmp(&b.name));
    targets
}

/// Um alvo pelo nome.
#[must_use]
pub fn find(root: &Path, name: &str) -> Option<RemoteTarget> {
    store::load(root).into_iter().find(|t| t.name == name)
}

/// Valida o que a UI mandou — a mensagem diz o que fazer.
///
/// # Errors
///
/// Nome ou host vazios, porta zero, nome com barra ou espaco.
pub fn validate(target: &RemoteTarget) -> Result<(), String> {
    let name = target.name.trim();
    if name.is_empty() {
        return Err("de um nome ao alvo (ex.: pi)".to_owned());
    }
    if name.contains('/') || name.contains(char::is_whitespace) {
        return Err("o nome do alvo nao pode ter barra nem espaco".to_owned());
    }
    let host = target.host.trim();
    if host.is_empty() {
        return Err("informe o host ou IP do alvo".to_owned());
    }
    if host.contains(char::is_whitespace) || host.contains('@') {
        return Err("o host nao leva usuario nem espaco — o usuario tem campo proprio".to_owned());
    }
    if target.port == Some(0) {
        return Err("a porta SSH nao pode ser 0".to_owned());
    }
    Ok(())
}

fn normalize(target: &RemoteTarget) -> RemoteTarget {
    let limpo = |s: &Option<String>| {
        s.as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
    };
    RemoteTarget {
        name: target.name.trim().to_owned(),
        host: target.host.trim().to_owned(),
        user: limpo(&target.user),
        port: target.port.filter(|p| *p != 22),
        identity_file: limpo(&target.identity_file),
        deploy_dir: limpo(&target.deploy_dir),
    }
}

/// Cria ou substitui pelo nome.
///
/// # Errors
///
/// Validacao ou disco.
pub fn save(root: &Path, target: &RemoteTarget) -> Result<Vec<RemoteTarget>, String> {
    validate(target)?;
    let normalizado = normalize(target);
    let mut targets = store::load(root);
    match targets.iter().position(|t| t.name == normalizado.name) {
        Some(i) => targets[i] = normalizado,
        None => targets.push(normalizado),
    }
    targets.sort_by(|a, b| a.name.cmp(&b.name));
    store::save(root, &targets)?;
    Ok(targets)
}

/// Remove pelo nome (o que nao existe e' sucesso com o catalogo atual).
///
/// # Errors
///
/// Disco.
pub fn remove(root: &Path, name: &str) -> Result<Vec<RemoteTarget>, String> {
    let mut targets = store::load(root);
    targets.retain(|t| t.name != name);
    store::save(root, &targets)?;
    Ok(targets)
}

/// `[user@]host`.
#[must_use]
pub fn destination(target: &RemoteTarget) -> String {
    target.user.as_ref().map_or_else(
        || target.host.clone(),
        |user| format!("{user}@{}", target.host),
    )
}

/// Os argumentos do `ssh` ate' o destino: `-o BatchMode=yes -o ConnectTimeout=5
/// [-p P] [-i K] [user@]host`. `batch` desliga o `BatchMode` para um shell
/// interativo (onde perguntar a senha e' o certo).
#[must_use]
pub fn ssh_args(target: &RemoteTarget, batch: bool) -> Vec<String> {
    let mut args = Vec::new();
    if batch {
        args.extend(["-o", "BatchMode=yes", "-o", "ConnectTimeout=5"].map(str::to_owned));
    }
    if let Some(port) = target.port {
        args.push("-p".to_owned());
        args.push(port.to_string());
    }
    if let Some(key) = &target.identity_file {
        args.push("-i".to_owned());
        args.push(key.clone());
    }
    args.push(destination(target));
    args
}

/// O script do probe: uma linha por fato, para o parser nao adivinhar.
#[must_use]
pub fn probe_script() -> String {
    let ferramentas = TOOLS.join(" ");
    format!(
        "uname -m; uname -sr; for t in {ferramentas}; do printf '%s=' \"$t\"; command -v \"$t\" || echo; done"
    )
}

/// O que o probe leu: `arch`, `kernel` e as ferramentas.
#[must_use]
pub fn parse_probe(raw: &str) -> (Option<String>, Option<String>, Vec<RemoteTool>) {
    let mut linhas = raw.lines().map(str::trim).filter(|l| !l.is_empty());
    let arch = linhas.next().map(str::to_owned);
    let kernel = linhas.next().map(str::to_owned);
    let mut tools: Vec<RemoteTool> = TOOLS
        .iter()
        .map(|id| RemoteTool {
            id: (*id).to_owned(),
            found: false,
            path: None,
        })
        .collect();
    for linha in linhas {
        if let Some((id, caminho)) = linha.split_once('=')
            && let Some(tool) = tools.iter_mut().find(|t| t.id == id)
        {
            let caminho = caminho.trim();
            tool.found = !caminho.is_empty();
            tool.path = (!caminho.is_empty()).then(|| caminho.to_owned());
        }
    }
    (arch, kernel, tools)
}

/// A falha do `ssh` em palavras que dizem o proximo passo.
#[must_use]
pub fn describe_ssh_failure(target: &RemoteTarget, raw: &str) -> String {
    let baixo = raw.to_lowercase();
    let destino = destination(target);
    if baixo.contains("permission denied") || baixo.contains("host key verification failed") {
        return format!(
            "o alvo recusou a chave (BatchMode): copie a sua com `ssh-copy-id {destino}` e aceite \
             o host key uma vez no terminal — a IDE nunca digita senha"
        );
    }
    if baixo.contains("could not resolve") || baixo.contains("name or service not known") {
        return format!(
            "nao resolvi o host `{}` — IP ou nome errado, ou sem rede",
            target.host
        );
    }
    if baixo.contains("connection timed out")
        || baixo.contains("connection refused")
        || baixo.contains("no route")
    {
        return format!("nao alcancei {destino} em 5 s: a placa esta' ligada e o sshd ativo?");
    }
    let cauda: Vec<&str> = raw.lines().filter(|l| !l.trim().is_empty()).collect();
    let inicio = cauda.len().saturating_sub(3);
    let texto = cauda[inicio..].join("\n");
    if texto.is_empty() {
        format!("o ssh para {destino} saiu com erro sem escrever nada")
    } else {
        texto
    }
}

/// A pasta de deploy: a do alvo, senao `~/kinein/<projeto>` (o `~` e' do
/// shell remoto, por isso fica sem aspas no destino).
#[must_use]
pub fn deploy_dir(target: &RemoteTarget, project: &str) -> String {
    target
        .deploy_dir
        .clone()
        .unwrap_or_else(|| format!("~/kinein/{project}"))
}

/// A linha do deploy: `rsync -az --delete -e 'ssh …' <origem> <destino>:<dest>/`
/// ou `scp [-P] [-i] -r <origem> <destino>:<dest>/`.
#[must_use]
pub fn deploy_command(
    target: &RemoteTarget,
    source: &Path,
    dest: &str,
    use_rsync: bool,
) -> (String, Vec<String>) {
    let destino = format!("{}:{}/", destination(target), dest);
    if use_rsync {
        let mut e = vec!["ssh".to_owned()];
        if let Some(port) = target.port {
            e.push(format!("-p {port}"));
        }
        if let Some(key) = &target.identity_file {
            e.push(format!("-i {key}"));
        }
        (
            "rsync".to_owned(),
            vec![
                "-az".to_owned(),
                "--delete".to_owned(),
                "-e".to_owned(),
                e.join(" "),
                source.display().to_string(),
                destino,
            ],
        )
    } else {
        let mut args = Vec::new();
        if let Some(port) = target.port {
            args.push("-P".to_owned());
            args.push(port.to_string());
        }
        if let Some(key) = &target.identity_file {
            args.push("-i".to_owned());
            args.push(key.clone());
        }
        args.push("-r".to_owned());
        args.push(source.display().to_string());
        args.push(destino);
        ("scp".to_owned(), args)
    }
}

/// Aspas simples POSIX para o comando REMOTO (roda no shell do alvo).
fn quote(texto: &str) -> String {
    format!("'{}'", texto.replace('\'', "'\\''"))
}

/// A linha `ssh … [user@]host '<comando remoto>'` que `run.start` roda por
/// `sh -c` — o `~` do deployDir fica fora das aspas para o shell REMOTO o
/// expandir.
#[must_use]
pub fn ssh_shell_line(target: &RemoteTarget, remote_command: Option<&str>) -> String {
    let mut partes = vec!["ssh".to_owned()];
    // Interativo (shell) sem BatchMode. Comando com `-tt`: pty forcado
    // mesmo sem terminal local (o `debugServer` do kit roda por `sh -c`
    // com stdio fechado), para que matar o `ssh` local derrube o processo
    // REMOTO por SIGHUP — sem isso o gdbserver ficaria orfao na placa.
    if remote_command.is_some() {
        partes.push("-tt".to_owned());
    }
    partes.extend(ssh_args(target, false));
    if let Some(cmd) = remote_command {
        partes.push(quote(cmd));
    }
    partes.join(" ")
}

/// O comando remoto de cada tipo. `program` e' o caminho NO ALVO.
#[must_use]
pub fn remote_command(kind: RemoteCommandKind, program: &str, port: u16) -> Option<String> {
    match kind {
        RemoteCommandKind::Run => Some(program.to_owned()),
        RemoteCommandKind::DebugServer => Some(format!("gdbserver :{port} {program}")),
        RemoteCommandKind::Debugpy => Some(format!(
            "python3 -m debugpy --listen 0.0.0.0:{port} --wait-for-client {program}"
        )),
        RemoteCommandKind::Shell => None,
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use kinein_protocol::{RemoteCommandKind, RemoteTarget};

    use super::{
        deploy_command, deploy_dir, describe_ssh_failure, parse_probe, probe_script,
        remote_command, ssh_args, ssh_shell_line, validate,
    };

    fn pi() -> RemoteTarget {
        RemoteTarget {
            name: "pi".to_owned(),
            host: "192.168.0.42".to_owned(),
            user: Some("pi".to_owned()),
            port: Some(2222),
            identity_file: Some("/home/u/.ssh/pi".to_owned()),
            deploy_dir: None,
        }
    }

    #[test]
    fn ssh_lines_carry_port_key_and_destination() {
        assert_eq!(
            ssh_args(&pi(), true),
            [
                "-o",
                "BatchMode=yes",
                "-o",
                "ConnectTimeout=5",
                "-p",
                "2222",
                "-i",
                "/home/u/.ssh/pi",
                "pi@192.168.0.42"
            ]
        );
        let simples = RemoteTarget {
            name: "b".to_owned(),
            host: "bancada".to_owned(),
            user: None,
            port: None,
            identity_file: None,
            deploy_dir: Some("/opt/app".to_owned()),
        };
        assert_eq!(ssh_args(&simples, false), ["bancada"]);
        assert_eq!(
            ssh_shell_line(&pi(), Some("~/kinein/demo/app")),
            "ssh -tt -p 2222 -i /home/u/.ssh/pi pi@192.168.0.42 '~/kinein/demo/app'"
        );
        assert_eq!(ssh_shell_line(&simples, None), "ssh bancada");
        assert_eq!(deploy_dir(&pi(), "demo"), "~/kinein/demo");
        assert_eq!(deploy_dir(&simples, "demo"), "/opt/app");
        assert!(
            probe_script().starts_with("uname -m; uname -sr; for t in gdbserver python3 rsync;")
        );
    }

    #[test]
    fn deploy_uses_rsync_over_ssh_or_scp() {
        let (p, a) = deploy_command(&pi(), Path::new("/w/build/app"), "~/kinein/demo", true);
        assert_eq!(p, "rsync");
        assert_eq!(
            a,
            [
                "-az",
                "--delete",
                "-e",
                "ssh -p 2222 -i /home/u/.ssh/pi",
                "/w/build/app",
                "pi@192.168.0.42:~/kinein/demo/"
            ]
        );
        let (p, a) = deploy_command(&pi(), Path::new("/w/build/app"), "/opt/app", false);
        assert_eq!(p, "scp");
        assert_eq!(
            a,
            [
                "-P",
                "2222",
                "-i",
                "/home/u/.ssh/pi",
                "-r",
                "/w/build/app",
                "pi@192.168.0.42:/opt/app/"
            ]
        );
    }

    #[test]
    fn the_probe_is_read_line_by_line_and_failures_say_the_next_step() {
        let (arch, kernel, tools) = parse_probe(
            "aarch64\nLinux 6.6.31+rpt-rpi-v8\ngdbserver=/usr/bin/gdbserver\npython3=/usr/bin/python3\nrsync=\n",
        );
        assert_eq!(arch.as_deref(), Some("aarch64"));
        assert_eq!(kernel.as_deref(), Some("Linux 6.6.31+rpt-rpi-v8"));
        assert_eq!(tools.len(), 3);
        assert!(tools[0].found && tools[0].path.as_deref() == Some("/usr/bin/gdbserver"));
        assert!(!tools[2].found && tools[2].path.is_none());
        let (arch, _, tools) = parse_probe("");
        assert_eq!(arch, None);
        assert!(tools.iter().all(|t| !t.found));
        assert!(
            describe_ssh_failure(&pi(), "pi@192.168.0.42: Permission denied (publickey).")
                .contains("ssh-copy-id pi@192.168.0.42")
        );
        assert!(
            describe_ssh_failure(
                &pi(),
                "ssh: connect to host 192.168.0.42 port 2222: Connection timed out"
            )
            .contains("5 s")
        );
        assert!(
            describe_ssh_failure(
                &pi(),
                "ssh: Could not resolve hostname x: Name or service not known"
            )
            .contains("nao resolvi")
        );
        assert_eq!(
            describe_ssh_failure(&pi(), ""),
            "o ssh para pi@192.168.0.42 saiu com erro sem escrever nada"
        );
    }

    #[test]
    fn remote_commands_and_validation() {
        assert_eq!(
            remote_command(RemoteCommandKind::Run, "~/kinein/d/app", 0).as_deref(),
            Some("~/kinein/d/app")
        );
        assert_eq!(
            remote_command(RemoteCommandKind::DebugServer, "/opt/app", 2345).as_deref(),
            Some("gdbserver :2345 /opt/app")
        );
        assert_eq!(
            remote_command(RemoteCommandKind::Debugpy, "/opt/main.py", 5678).as_deref(),
            Some("python3 -m debugpy --listen 0.0.0.0:5678 --wait-for-client /opt/main.py")
        );
        assert_eq!(remote_command(RemoteCommandKind::Shell, "", 0), None);
        let mut t = pi();
        assert!(validate(&t).is_ok());
        t.name = " ".to_owned();
        assert!(validate(&t).unwrap_err().contains("nome"));
        t.name = "a b".to_owned();
        assert!(validate(&t).is_err());
        t.name = "pi".to_owned();
        t.host = "pi@host".to_owned();
        assert!(
            validate(&t)
                .unwrap_err()
                .contains("usuario tem campo proprio")
        );
        t.host = "h".to_owned();
        t.port = Some(0);
        assert!(validate(&t).is_err());
    }
}
