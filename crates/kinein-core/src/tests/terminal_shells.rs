//! INF-02 (62 §3.4): a command requested at `terminal.open` runs in real bash,
//! zsh and fish, whatever their startup does first.
//!
//! Every profile runs the shell installed on this machine through a wrapper
//! named after it, which gives the shell its own HOME/ZDOTDIR/XDG tree and a
//! minimal environment, so the author's configuration is never read nor
//! written. A profile whose shell or theme is absent is skipped by name.
//! Needles are what the command prints, never what was typed: `kv''-ran`
//! shows on screen as typed and only becomes `kv-ran` when executed.

use std::{
    path::{Path, PathBuf},
    sync::mpsc,
    time::{Duration, Instant},
};

use serde_json::{Value, json};

use super::core_with_empty_search_path;
use kinein_protocol::{JsonRpcErrorCode, JsonRpcRequest};

const P10K: &str = "/usr/share/zsh-theme-powerlevel10k/powerlevel10k.zsh-theme";
const OH_MY_ZSH: &str = "/usr/share/oh-my-zsh/oh-my-zsh.sh";
const AUTOSUGGESTIONS: &str = "/usr/share/zsh/plugins/zsh-autosuggestions/zsh-autosuggestions.zsh";
const HIGHLIGHTING: &str =
    "/usr/share/zsh/plugins/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh";
const LIMIT: Duration = Duration::from_secs(20);

struct Profile {
    name: &'static str,
    shell: &'static str,
    files: &'static [(&'static str, &'static str)],
    requires: &'static [&'static str],
    /// The ZDOTDIR the user's environment has, relative to HOME.
    user_zdotdir: Option<&'static str>,
    /// Startup that asks before the prompt: (text on screen, answer).
    wizard: Option<(&'static str, &'static str)>,
    /// After the command ran: (input, needle on screen).
    checks: &'static [(&'static str, &'static str)],
}

const fn profile(name: &'static str, shell: &'static str) -> Profile {
    Profile {
        name,
        shell,
        files: &[],
        requires: &[],
        user_zdotdir: None,
        wizard: None,
        checks: &[],
    }
}

const P10K_ZSHRC: &str = r#"if [[ -r "${XDG_CACHE_HOME:-$HOME/.cache}/p10k-instant-prompt-${(%):-%n}.zsh" ]]; then
  source "${XDG_CACHE_HOME:-$HOME/.cache}/p10k-instant-prompt-${(%):-%n}.zsh"
fi
source /usr/share/zsh-theme-powerlevel10k/powerlevel10k.zsh-theme
[[ ! -f ~/.p10k.zsh ]] || source ~/.p10k.zsh
"#;

const P10K_CONFIG: &str = "POWERLEVEL9K_LEFT_PROMPT_ELEMENTS=(dir prompt_char)
POWERLEVEL9K_RIGHT_PROMPT_ELEMENTS=(status)
POWERLEVEL9K_MODE=ascii
POWERLEVEL9K_INSTANT_PROMPT=verbose
POWERLEVEL9K_DISABLE_CONFIGURATION_WIZARD=true
";

const OH_MY_ZSHRC: &str = "export ZSH=/usr/share/oh-my-zsh
ZSH_THEME=robbyrussell
plugins=(git)
ZSH_CACHE_DIR=$HOME/.cache/oh-my-zsh
source $ZSH/oh-my-zsh.sh
source /usr/share/zsh/plugins/zsh-autosuggestions/zsh-autosuggestions.zsh
source /usr/share/zsh/plugins/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh
";

/// zsh without a ZDOTDIR of its own reports `zd=unset kz=0`.
const ZSH_ZDOTDIR_RESTORED: (&str, &str) = (
    "echo zd=${ZDOTDIR-unset} kz=${+KINEIN_ZDOTDIR}\n",
    "zd=unset kz=0",
);

/// Builds the isolated home and the wrapper; returns (wrapper, workspace, base).
fn wrapper(profile: &Profile) -> (PathBuf, PathBuf, PathBuf) {
    let base = std::env::temp_dir().join("kinein-core-tests").join(format!(
        "{}-shells-{}",
        std::process::id(),
        profile.name
    ));
    drop(std::fs::remove_dir_all(&base));
    let home = base.join("home");
    std::fs::create_dir_all(&home).unwrap();
    let base = crate::platform::canonicalize(&base).unwrap();
    let home = base.join("home");
    for (relative, contents) in profile.files {
        let path = home.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }
    let h = home.display();
    let common = format!(
        "PATH=\"$PATH\" LANG=C.UTF-8 TERM=\"$TERM\" COLORTERM=\"$COLORTERM\" \
         TERM_PROGRAM=\"$TERM_PROGRAM\" USER=\"$USER\" HOME='{h}' \
         XDG_CONFIG_HOME='{h}/.config' XDG_DATA_HOME='{h}/.local/share' \
         XDG_CACHE_HOME='{h}/.cache' XDG_STATE_HOME='{h}/.local/state'"
    );
    let user = |name: &str| {
        profile
            .user_zdotdir
            .map_or_else(String::new, |dir| format!("{name}='{h}/{dir}'"))
    };
    let integration = base.join("integration");
    let script = format!(
        "#!/bin/sh\n\
         printf '%s\\n' \"$@\" > '{base}/args'\n\
         printf '%s\\n' \"${{ZDOTDIR-}}\" > '{base}/zdotdir'\n\
         case \"${{ZDOTDIR-}}\" in\n\
         '{integration}'/*) exec env -i {common} ZDOTDIR=\"$ZDOTDIR\" {kinein} /usr/bin/{shell} \"$@\" ;;\n\
         *) exec env -i {common} {plain} /usr/bin/{shell} \"$@\" ;;\n\
         esac\n",
        base = base.display(),
        integration = integration.display(),
        kinein = user("KINEIN_ZDOTDIR"),
        plain = user("ZDOTDIR"),
        shell = profile.shell,
    );
    std::fs::create_dir_all(base.join("bin")).unwrap();
    let program = crate::write_executable(base.join("bin").join(profile.shell), script);
    let root = base.join("workspace");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("Cargo.toml"), "[package]\n").unwrap();
    (program, root, base)
}

fn available(profile: &Profile) -> bool {
    let shell = format!("/usr/bin/{}", profile.shell);
    let missing = std::iter::once(shell.as_str())
        .chain(profile.requires.iter().copied())
        .find(|path| !Path::new(path).exists());
    if let Some(path) = missing {
        eprintln!("SKIP {}: {path} ausente", profile.name);
    }
    missing.is_none()
}

fn screen_text(params: &Value) -> String {
    params["lines"]
        .as_array()
        .map(|lines| {
            lines
                .iter()
                .map(|line| {
                    line.as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(|span| span["text"].as_str())
                        .collect::<String>()
                })
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default()
}

/// What the core said about one session, absorbed as events arrive.
struct Watch<'a> {
    events: &'a mpsc::Receiver<JsonRpcRequest>,
    id: String,
    screen: String,
    /// `event.terminal.command` payloads, in order.
    commands: Vec<(String, String)>,
    /// How many command events had arrived when `closed` did.
    closed_after: Option<usize>,
}

impl<'a> Watch<'a> {
    fn new(events: &'a mpsc::Receiver<JsonRpcRequest>, id: &str) -> Self {
        Self {
            events,
            id: id.to_owned(),
            screen: String::new(),
            commands: Vec::new(),
            closed_after: None,
        }
    }

    fn absorb(&mut self, event: &JsonRpcRequest) {
        let Some(params) = event.params.as_ref() else {
            return;
        };
        if params["id"].as_str() != Some(self.id.as_str()) {
            return;
        }
        match event.method.as_str() {
            "event.terminal.render" => self.screen = screen_text(params),
            "event.terminal.command" => self.commands.push((
                params["state"].as_str().unwrap_or("").to_owned(),
                params["reason"].as_str().unwrap_or("").to_owned(),
            )),
            "event.terminal.closed" => self.closed_after = Some(self.commands.len()),
            _ => {}
        }
    }

    /// Absorbs events until `done` holds; `false` when `limit` passes first.
    fn until(&mut self, limit: Duration, done: impl Fn(&Self) -> bool) -> bool {
        let deadline = Instant::now() + limit;
        while !done(self) {
            let Some(left) = deadline.checked_duration_since(Instant::now()) else {
                return false;
            };
            match self.events.recv_timeout(left) {
                Ok(event) => self.absorb(&event),
                Err(_) => return done(self),
            }
        }
        true
    }

    fn shows(&mut self, needle: &str) -> bool {
        self.until(LIMIT, |watch| watch.screen.contains(needle))
    }

    fn failure(&self, what: &str) -> String {
        format!(
            "{what}\ncommand events: {:?}\n---- screen ----\n{}\n----",
            self.commands, self.screen
        )
    }
}

fn request(core: &mut crate::Core, method: &str, params: Value) -> Value {
    let outcome = core.handle_request(&JsonRpcRequest::new(7_i64, method, Some(params)));
    let response = outcome.response();
    assert!(response.error.is_none(), "{method}: {:?}", response.error);
    response.result.clone().unwrap_or_default()
}

fn error_code(core: &mut crate::Core, method: &str, params: Value) -> Option<JsonRpcErrorCode> {
    let outcome = core.handle_request(&JsonRpcRequest::new(8_i64, method, Some(params)));
    outcome.response().error.as_ref().map(|error| error.code)
}

/// A core whose `terminal.open` starts `program`, with the workspace open.
fn core_for(
    name: &str,
    program: &Path,
    root: &Path,
    base: &Path,
) -> (crate::Core, mpsc::Receiver<JsonRpcRequest>) {
    let (sender, events) = mpsc::channel();
    let mut core = core_with_empty_search_path(&format!("shells-{name}"));
    core.enable_lsp(sender);
    let terminal = core.terminal.as_mut().unwrap();
    terminal.use_shell(program.to_str().unwrap());
    terminal.use_integration_root(&base.join("integration"));
    request(&mut core, "workspace.open", json!({ "path": root }));
    (core, events)
}

fn run(profile: &Profile) {
    if !available(profile) {
        return;
    }
    let (program, root, base) = wrapper(profile);
    let (mut core, events) = core_for(profile.name, &program, &root, &base);
    let opened = request(
        &mut core,
        "terminal.open",
        json!({ "command": "echo kv''-ran-42" }),
    );
    assert_eq!(opened["command"], "waiting", "{}", profile.name);
    let id = opened["id"].as_str().unwrap().to_owned();
    let mut watch = Watch::new(&events, &id);
    if let Some((question, answer)) = profile.wizard {
        assert!(watch.shows(question), "{}", watch.failure(profile.name));
        assert!(
            watch.commands.is_empty(),
            "{}",
            watch.failure("the command left before the wizard was answered")
        );
        request(
            &mut core,
            "terminal.input",
            json!({ "id": id, "data": answer }),
        );
    }
    assert!(
        watch.until(LIMIT, |watch| !watch.commands.is_empty()),
        "{}",
        watch.failure(&format!("{}: no prompt marker", profile.name))
    );
    assert_eq!(watch.commands, [("sent".into(), "prompt".into())]);
    assert!(watch.shows("kv-ran-42"), "{}", watch.failure(profile.name));
    for (input, needle) in profile.checks {
        request(
            &mut core,
            "terminal.input",
            json!({ "id": id, "data": input }),
        );
        assert!(
            watch.shows(needle),
            "{}",
            watch.failure(&format!("{}: {needle}", profile.name))
        );
    }
    assert!(
        !watch.screen.contains("not found"),
        "{}",
        watch.failure(profile.name)
    );
    request(&mut core, "terminal.close", json!({ "id": id }));
    assert!(watch.until(LIMIT, |watch| watch.closed_after.is_some()));
    assert_eq!(watch.commands.len(), 1, "nothing settles twice");
    drop(std::fs::remove_dir_all(&base));
}

#[test]
fn bash_without_rc() {
    run(&profile("bash-no-rc", "bash"));
}

#[test]
fn bash_slow_rc_with_its_own_prompt_command() {
    run(&Profile {
        files: &[(
            ".bashrc",
            "sleep 1\nPROMPT_COMMAND='printf \"\\033]0;kv\\007\"'\nPS1='kvbash$ '\n",
        )],
        checks: &[("echo ok\n", "kvbash$ ")],
        ..profile("bash-slow-rc", "bash")
    });
}

/// A theme that rewrites PS1 at every prompt keeps it, and keeps `$?`.
#[test]
fn bash_theme_rewriting_ps1_keeps_exit_status() {
    run(&Profile {
        files: &[(".bashrc", "PROMPT_COMMAND='PS1=\"kvtheme($?)> \"'\n")],
        checks: &[("false\n", "kvtheme(1)> ")],
        ..profile("bash-theme", "bash")
    });
}

/// `$?` in PS1 itself, and in hooks added later, is the user's last command.
#[test]
fn bash_ps1_showing_exit_status() {
    run(&Profile {
        files: &[(".bashrc", "PS1='kvst[$?]\\$ '\n")],
        checks: &[
            ("false\n", "kvst[1]$ "),
            // A hook appended after the integration still sees the status.
            ("PROMPT_COMMAND+=('kvlast=$?')\n", "kvst[0]$ "),
            ("false\n", "kvst[1]$ "),
            ("echo kv''-last=$kvlast\n", "kv-last=1"),
        ],
        ..profile("bash-ps1-status", "bash")
    });
}

/// zsh-newuser-install is still offered to whoever has no zsh files, and the
/// command waits for it.
#[test]
fn zsh_new_user_keeps_its_first_run_menu() {
    run(&Profile {
        wizard: Some(("(q)  Quit and do nothing.", "q")),
        checks: &[ZSH_ZDOTDIR_RESTORED],
        ..profile("zsh-newuser", "zsh")
    });
}

#[test]
fn zsh_new_user_menu_writes_the_users_zshrc() {
    run(&Profile {
        wizard: Some(("(0)  Exit, creating the file ~/.zshrc", "0")),
        checks: &[("head -c 22 ~/.zshrc; echo :end\n", "# Created by newuser f")],
        ..profile("zsh-newuser-0", "zsh")
    });
}

#[test]
fn zsh_slow_rc() {
    run(&Profile {
        files: &[(".zshrc", "sleep 1\nPROMPT='kvzsh%# '\n")],
        checks: &[ZSH_ZDOTDIR_RESTORED, ("echo ok\n", "kvzsh% ")],
        ..profile("zsh-slow-rc", "zsh")
    });
}

#[test]
fn zsh_p10k_wizard_is_answered_before_the_command() {
    run(&Profile {
        files: &[(
            ".zshrc",
            "source /usr/share/zsh-theme-powerlevel10k/powerlevel10k.zsh-theme\n",
        )],
        requires: &[P10K],
        wizard: Some(("Choice [yn", "q")),
        ..profile("zsh-p10k-unconfigured", "zsh")
    });
}

#[test]
fn zsh_p10k_instant_prompt() {
    run(&Profile {
        files: &[(".zshrc", P10K_ZSHRC), (".p10k.zsh", P10K_CONFIG)],
        requires: &[P10K],
        checks: &[ZSH_ZDOTDIR_RESTORED],
        ..profile("zsh-p10k-instant-prompt", "zsh")
    });
}

#[test]
fn zsh_oh_my_zsh_with_plugins() {
    run(&Profile {
        files: &[(".zshrc", OH_MY_ZSHRC)],
        requires: &[OH_MY_ZSH, AUTOSUGGESTIONS, HIGHLIGHTING],
        checks: &[ZSH_ZDOTDIR_RESTORED],
        ..profile("zsh-oh-my-zsh", "zsh")
    });
}

/// The user's own ZDOTDIR is given back before their files are read.
#[test]
fn zsh_user_zdotdir() {
    run(&Profile {
        files: &[
            (".config/zsh/.zshenv", "KV_ENV=yes\n"),
            (".config/zsh/.zshrc", "KV_RC=yes\n"),
        ],
        user_zdotdir: Some(".config/zsh"),
        checks: &[(
            "echo zd=${ZDOTDIR#$HOME/} rc=$KV_RC env=$KV_ENV kz=${+KINEIN_ZDOTDIR}\n",
            "zd=.config/zsh rc=yes env=yes kz=0",
        )],
        ..profile("zsh-user-zdotdir", "zsh")
    });
}

/// The common XDG layout: ~/.zshenv points ZDOTDIR elsewhere.
#[test]
fn zsh_zshenv_that_moves_zdotdir() {
    run(&Profile {
        files: &[
            (".zshenv", "export ZDOTDIR=$HOME/.config/zsh\n"),
            (".config/zsh/.zshrc", "KV_RC=xdg\n"),
        ],
        checks: &[(
            "echo zd=${ZDOTDIR#$HOME/} rc=$KV_RC\n",
            "zd=.config/zsh rc=xdg",
        )],
        ..profile("zsh-zshenv-zdotdir", "zsh")
    });
}

#[test]
fn fish_without_config() {
    run(&profile("fish-no-config", "fish"));
}

#[test]
fn fish_with_its_own_config() {
    run(&Profile {
        files: &[(
            ".config/fish/config.fish",
            "set -g fish_greeting kv-greeting\nfunction fish_prompt\n    echo -n 'kvfish> '\nend\n",
        )],
        checks: &[("echo ok\n", "kvfish> ")],
        ..profile("fish-config", "fish")
    });
}

/// A bashrc that becomes another shell never marks a prompt: the command
/// waits until it is sent or discarded on request.
#[test]
fn command_without_a_marker_is_sent_or_discarded_on_request() {
    let profile = Profile {
        files: &[(".bashrc", "exec /bin/sh\n")],
        ..profile("bash-exec-sh", "bash")
    };
    if !available(&profile) {
        return;
    }
    let (program, root, base) = wrapper(&profile);
    let (mut core, events) = core_for(profile.name, &program, &root, &base);
    for (action, state) in [("send", "sent"), ("discard", "discarded")] {
        let opened = request(
            &mut core,
            "terminal.open",
            json!({ "command": "echo kv''-ran-7" }),
        );
        assert_eq!(opened["command"], "waiting");
        let id = opened["id"].as_str().unwrap().to_owned();
        let mut watch = Watch::new(&events, &id);
        request(
            &mut core,
            "terminal.input",
            json!({ "id": id, "data": "echo kv''-sh-up\n" }),
        );
        assert!(
            watch.shows("kv-sh-up"),
            "{}",
            watch.failure("sh did not start")
        );
        assert!(
            watch.commands.is_empty(),
            "{}",
            watch.failure("sh marked a prompt")
        );
        let result = request(
            &mut core,
            "terminal.pendingCommand",
            json!({ "id": id, "action": action }),
        );
        assert_eq!(result, json!({ "id": id, "state": state }));
        assert!(watch.until(LIMIT, |watch| !watch.commands.is_empty()));
        assert_eq!(watch.commands, [(state.into(), "request".into())]);
        let again = request(
            &mut core,
            "terminal.pendingCommand",
            json!({ "id": id, "action": "send" }),
        );
        assert_eq!(again["state"], "none", "a settled command is gone");
        request(
            &mut core,
            "terminal.input",
            json!({ "id": id, "data": "echo kv''-done\n" }),
        );
        assert!(watch.shows("kv-done"));
        assert_eq!(
            watch.screen.contains("kv-ran-7"),
            action == "send",
            "{}",
            watch.failure(action)
        );
        request(&mut core, "terminal.close", json!({ "id": id }));
        assert!(watch.until(LIMIT, |watch| watch.closed_after.is_some()));
        assert_eq!(watch.commands.len(), 1);
    }
    drop(std::fs::remove_dir_all(&base));
}

/// A shell that exits before its prompt discards the command, before `closed`.
#[test]
fn command_is_discarded_when_the_shell_exits_first() {
    let profile = Profile {
        files: &[(".bashrc", "exit 3\n")],
        ..profile("bash-exits", "bash")
    };
    if !available(&profile) {
        return;
    }
    let (program, root, base) = wrapper(&profile);
    let (mut core, events) = core_for(profile.name, &program, &root, &base);
    let opened = request(
        &mut core,
        "terminal.open",
        json!({ "command": "echo kv''-ran-3" }),
    );
    let id = opened["id"].as_str().unwrap().to_owned();
    let mut watch = Watch::new(&events, &id);
    assert!(watch.until(LIMIT, |watch| watch.closed_after.is_some()));
    assert_eq!(watch.commands, [("discarded".into(), "exit".into())]);
    assert_eq!(
        watch.closed_after,
        Some(1),
        "the discard comes before closed"
    );
    assert_eq!(
        error_code(
            &mut core,
            "terminal.pendingCommand",
            json!({ "id": id, "action": "send" })
        ),
        Some(JsonRpcErrorCode::InvalidRequest)
    );
    drop(std::fs::remove_dir_all(&base));
}

/// A shell without known integration gets the command at open (documented
/// limit), and a terminal opened without a command starts unchanged.
#[test]
fn unknown_shell_gets_the_command_at_open_and_plain_terminals_are_untouched() {
    let profile = profile("bash-plain", "bash");
    if !available(&profile) {
        return;
    }
    let (program, root, base) = wrapper(&profile);
    let (mut core, events) = core_for(profile.name, &program, &root, &base);
    let opened = request(&mut core, "terminal.open", json!({}));
    assert!(opened.get("command").is_none(), "{opened}");
    let mut watch = Watch::new(&events, opened["id"].as_str().unwrap());
    request(
        &mut core,
        "terminal.input",
        json!({ "id": watch.id, "data": "echo kv''-plain\n" }),
    );
    assert!(watch.shows("kv-plain"), "{}", watch.failure("plain bash"));
    assert_eq!(
        std::fs::read_to_string(base.join("args")).unwrap(),
        "\n",
        "no integration arguments"
    );
    assert!(
        !base.join("integration").exists(),
        "nothing written for a plain terminal"
    );

    core.terminal.as_mut().unwrap().use_shell("/bin/sh");
    let opened = request(
        &mut core,
        "terminal.open",
        json!({ "command": "echo kv''-sh-ran" }),
    );
    assert_eq!(opened["command"], "sent");
    let mut watch = Watch::new(&events, opened["id"].as_str().unwrap());
    assert!(watch.shows("kv-sh-ran"), "{}", watch.failure("sh"));
    assert!(watch.commands.is_empty(), "nothing waited");
    assert!(
        !base.join("integration").exists(),
        "sh needs no integration files"
    );
    drop(std::fs::remove_dir_all(&base));
}

#[test]
fn invalid_commands_and_pending_requests_are_refused() {
    let profile = profile("bash-refusals", "bash");
    let (program, root, base) = wrapper(&profile);
    let (mut core, _events) = core_for(profile.name, &program, &root, &base);
    let long = "x".repeat(8193);
    for command in [
        "",
        "   ",
        "echo a\necho b",
        "echo a\r",
        "echo \u{1b}[31m",
        "echo \u{85}",
        long.as_str(),
    ] {
        assert_eq!(
            error_code(&mut core, "terminal.open", json!({ "command": command })),
            Some(JsonRpcErrorCode::InvalidParams),
            "{command:?}"
        );
    }
    assert!(
        !base.join("args").exists(),
        "no shell started for a refused command"
    );
    assert_eq!(
        error_code(
            &mut core,
            "terminal.pendingCommand",
            json!({ "id": "t404", "action": "send" })
        ),
        Some(JsonRpcErrorCode::InvalidRequest)
    );
    assert_eq!(
        error_code(
            &mut core,
            "terminal.pendingCommand",
            json!({ "id": "t1", "action": "later" })
        ),
        Some(JsonRpcErrorCode::InvalidParams)
    );
    drop(std::fs::remove_dir_all(&base));
}
