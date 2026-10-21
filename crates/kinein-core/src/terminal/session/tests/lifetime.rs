//! Real PTY exits and deterministic refusal, without serial devices.
use std::{
    io,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};

use kinein_protocol::JsonRpcRequest;
use portable_pty::{Child, ChildKiller, ExitStatus};
use serde_json::{Value, json};

use super::super::{TerminalManager, lifecycle};
use super::temp_root;
use crate::terminal::render::tests::render_contains;

pub(super) fn wait_closed(receiver: &mpsc::Receiver<JsonRpcRequest>, ids: &[String]) {
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut closed = std::collections::HashSet::new();
    while closed.len() < ids.len() && Instant::now() < deadline {
        if let Ok(event) = receiver.recv_timeout(Duration::from_millis(50))
            && event.method == "event.terminal.closed"
            && let Some(id) = event
                .params
                .as_ref()
                .and_then(|params| params["id"].as_str())
            && ids.iter().any(|expected| expected == id)
        {
            closed.insert(id.to_owned());
        }
    }
    assert_eq!(
        closed.len(),
        ids.len(),
        "process exits were not observed: {closed:?}"
    );
}

struct Cleanup(lifecycle::SharedChild);
impl Drop for Cleanup {
    fn drop(&mut self) {
        // Independent cleanup also works when the stop implementation is
        // deliberately broken during the mutation proof. The lock prevents
        // reaping/reusing this owned PID while cleanup signals it.
        if let Ok(mut child) = self.0.lock()
            && child.try_wait().is_ok_and(|status| status.is_none())
            && let Some(pid) = child.process_id()
        {
            crate::kill_process(&pid.to_string());
        }
    }
}

fn stubborn(manager: &mut TerminalManager, root: &Path) -> (String, Cleanup) {
    let id = manager
        .open_command(
            root,
            "sh",
            &[
                "-c".to_owned(),
                "trap '' HUP; printf 'stubborn-ready\\n'; sleep 30".to_owned(),
            ],
        )
        .unwrap();
    let child = Arc::clone(&manager.sessions[&id].child);
    (id, Cleanup(child))
}

#[test]
fn closing_a_process_that_ignores_hup_observes_its_exit() {
    let (sender, receiver) = mpsc::channel();
    let mut manager = TerminalManager::new(sender);
    let root = temp_root("stubborn-close");
    let (id, _cleanup) = stubborn(&mut manager, &root);
    assert!(render_contains(&receiver, "stubborn-ready"));
    let other = manager.open_with_shell(&root, "sh").unwrap();
    let start = Instant::now();
    manager.close(&id).unwrap();
    manager.close(&id).unwrap();
    wait_closed(&receiver, &[id]);
    assert!(start.elapsed() < Duration::from_secs(2), "close blocked");
    assert!(manager.is_open(&other), "another session was terminated");
    manager.close(&other).unwrap();
    wait_closed(&receiver, &[other]);
}

#[test]
fn dropping_the_manager_terminates_a_stubborn_process() {
    let (sender, receiver) = mpsc::channel();
    let mut manager = TerminalManager::new(sender);
    let root = temp_root("stubborn-drop");
    let (id, _cleanup) = stubborn(&mut manager, &root);
    assert!(render_contains(&receiver, "stubborn-ready"));
    drop(manager);
    wait_closed(&receiver, &[id]);
}

struct ChildCleanup {
    pid: String,
    active: bool,
}
impl Drop for ChildCleanup {
    fn drop(&mut self) {
        if self.active && process_running(&self.pid) {
            crate::kill_process(&self.pid);
        }
    }
}

fn process_running(pid: &str) -> bool {
    std::fs::read_to_string(format!("/proc/{pid}/status")).map_or_else(
        |_| crate::process_alive(pid),
        |status| {
            !status
                .lines()
                .any(|line| line.starts_with("State:") && line.contains("zombie"))
        },
    )
}

#[test]
fn closing_a_wrapper_also_terminates_its_background_child() {
    let (sender, receiver) = mpsc::channel();
    let mut manager = TerminalManager::new(sender);
    let root = temp_root("wrapper-child");
    let pid_file = root.join("child.pid");
    let script = format!(
        "trap '' HUP; sleep 30 & {}; printf 'wrapper-ready\\n'; wait",
        crate::sh_save_background_pid(&pid_file)
    );
    let id = manager
        .open_command(&root, "sh", &["-c".to_owned(), script])
        .unwrap();
    let _cleanup = Cleanup(Arc::clone(&manager.sessions[&id].child));
    assert!(render_contains(&receiver, "wrapper-ready"));
    let mut child = ChildCleanup {
        pid: std::fs::read_to_string(pid_file).unwrap().trim().to_owned(),
        active: true,
    };
    assert!(process_running(&child.pid));
    manager.close(&id).unwrap();
    wait_closed(&receiver, &[id]);
    let deadline = Instant::now() + Duration::from_secs(3);
    while process_running(&child.pid) && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(!process_running(&child.pid), "the wrapper's child survived");
    child.active = false;
}

#[derive(Debug, Clone)]
struct RefusingChild {
    exited: Arc<AtomicBool>,
    unavailable: Arc<AtomicBool>,
}
impl ChildKiller for RefusingChild {
    fn kill(&mut self) -> io::Result<()> {
        Err(io::Error::from(io::ErrorKind::PermissionDenied))
    }
    fn clone_killer(&self) -> Box<dyn ChildKiller + Send + Sync> {
        Box::new(self.clone())
    }
}
impl Child for RefusingChild {
    fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        if self.unavailable.load(Ordering::SeqCst) {
            return Err(io::Error::other("status indisponível"));
        }
        Ok(self
            .exited
            .load(Ordering::SeqCst)
            .then(|| ExitStatus::with_exit_code(0)))
    }
    fn wait(&mut self) -> io::Result<ExitStatus> {
        Err(io::Error::other("blocking wait is forbidden"))
    }
    fn process_id(&self) -> Option<u32> {
        None
    }
}

fn refuse(manager: &mut TerminalManager, id: &str) -> (Arc<AtomicBool>, Cleanup) {
    let exited = Arc::new(AtomicBool::new(false));
    let child: lifecycle::SharedChild = Arc::new(Mutex::new(Box::new(RefusingChild {
        exited: Arc::clone(&exited),
        unavailable: Arc::new(AtomicBool::new(false)),
    })));
    let original = std::mem::replace(&mut manager.sessions.get_mut(id).unwrap().child, child);
    (exited, Cleanup(original))
}

struct NoticeCleanup {
    unavailable: Arc<AtomicBool>,
    exited: Arc<AtomicBool>,
}
impl Drop for NoticeCleanup {
    fn drop(&mut self) {
        self.unavailable.store(false, Ordering::SeqCst);
        self.exited.store(true, Ordering::SeqCst);
    }
}

fn wait_frame(receiver: &mpsc::Receiver<JsonRpcRequest>, has_notice: bool) -> Value {
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        if let Ok(event) = receiver.recv_timeout(Duration::from_millis(50))
            && event.method == "event.terminal.render"
            && let Some(frame) = event.params
            && frame.get("notice").is_some() == has_notice
        {
            return frame;
        }
    }
    panic!("terminal frame not received");
}

#[test]
fn observation_failure_has_a_notice_preserves_vt_and_recovers_without_false_exit() {
    let (sender, receiver) = mpsc::channel();
    let state = Arc::new(Mutex::new(super::super::super::state::TerminalState::new(
        4, 80, 0,
    )));
    state.lock().unwrap().process(b"program-output");
    super::super::super::render::emit_render(&sender, "notice-test", &state);
    let before = wait_frame(&receiver, false);
    let unavailable = Arc::new(AtomicBool::new(true));
    let exited = Arc::new(AtomicBool::new(false));
    let _cleanup = NoticeCleanup {
        unavailable: Arc::clone(&unavailable),
        exited: Arc::clone(&exited),
    };
    let child: lifecycle::SharedChild = Arc::new(Mutex::new(Box::new(RefusingChild {
        unavailable: Arc::clone(&unavailable),
        exited: Arc::clone(&exited),
    })));
    let running = Arc::new(AtomicBool::new(true));
    let manager = TerminalManager::new(sender);
    manager.spawn_waiter("notice-test", &child, &running, &state);
    let failed = wait_frame(&receiver, true);
    assert_eq!(failed["notice"]["code"], "INTERNAL_ERROR");
    assert!(
        failed["notice"]["message"]
            .as_str()
            .unwrap()
            .contains("status indisponível")
    );
    assert_eq!(failed["lines"], before["lines"]);
    assert_eq!(failed["cursor"], before["cursor"]);
    assert!(
        running.load(Ordering::SeqCst),
        "observation error announced an exit"
    );
    unavailable.store(false, Ordering::SeqCst);
    let recovered = wait_frame(&receiver, false);
    assert_eq!(recovered["lines"], before["lines"]);
    assert!(running.load(Ordering::SeqCst));
    exited.store(true, Ordering::SeqCst);
    wait_closed(&receiver, &["notice-test".to_owned()]);
    assert!(!running.load(Ordering::SeqCst));
}

#[test]
fn a_failed_close_keeps_its_session_and_does_not_skip_other_sessions() {
    let (sender, receiver) = mpsc::channel();
    let mut manager = TerminalManager::new(sender);
    let root = temp_root("close-refusal");
    let failed = manager.open_with_shell(&root, "sh").unwrap();
    let other = manager.open_with_shell(&root, "sh").unwrap();
    let (exited, cleanup) = refuse(&mut manager, &failed);
    assert!(manager.close(&failed).is_err());
    assert!(manager.is_open(&failed), "refused close forgot its session");
    assert!(manager.close_all().is_err());
    wait_closed(&receiver, std::slice::from_ref(&other));
    assert!(manager.is_open(&failed));
    assert!(
        !manager.is_open(&other),
        "one refusal skipped another session"
    );
    exited.store(true, Ordering::SeqCst);
    manager.close(&failed).unwrap();
    drop(cleanup);
    wait_closed(&receiver, &[failed]);
}

fn rpc(core: &mut crate::Core, method: &str, params: Value) -> kinein_protocol::JsonRpcResponse {
    core.handle_request(&JsonRpcRequest::new(1_i64, method, Some(params)))
        .response()
        .clone()
}

fn core_fixture(name: &str) -> (crate::Core, mpsc::Receiver<JsonRpcRequest>, PathBuf) {
    let root = temp_root(name);
    let mut core = crate::Core::with_detector(crate::tools::ToolDetector::with_search_path(
        root.join("empty-bin"),
    ));
    let (sender, receiver) = mpsc::channel();
    core.enable_lsp(sender);
    assert!(
        rpc(&mut core, "workspace.open", json!({"path":root}))
            .error
            .is_none()
    );
    (core, receiver, root)
}

#[test]
fn changing_workspace_terminates_old_sessions_and_allows_a_new_run() {
    let (mut core, receiver, root) = core_fixture("workspace-processes");
    let run = rpc(
        &mut core,
        "run.start",
        json!({"command":"trap '' HUP; printf 'run-ready\\n'; read -r line"}),
    );
    assert!(run.error.is_none(), "{run:?}");
    let run_id = run.result.unwrap()["terminalId"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(render_contains(&receiver, "run-ready"));
    let (monitor, _cleanup) = stubborn(core.terminal.as_mut().unwrap(), &root);
    let next = root.join("another");
    std::fs::create_dir(&next).unwrap();
    assert!(
        rpc(&mut core, "workspace.open", json!({"path":next}))
            .error
            .is_none()
    );
    wait_closed(&receiver, &[run_id, monitor]);
    assert!(core.run_terminal.is_none());
    let run = rpc(
        &mut core,
        "run.start",
        json!({"command":"printf new-workspace"}),
    );
    assert!(run.error.is_none(), "{run:?}");
    let id = run.result.unwrap()["terminalId"]
        .as_str()
        .unwrap()
        .to_owned();
    wait_closed(&receiver, &[id]);
}

#[test]
fn reopening_the_same_root_preserves_sessions() {
    let (mut core, receiver, root) = core_fixture("same-workspace");
    let (id, _cleanup) = stubborn(core.terminal.as_mut().unwrap(), &root);
    assert!(
        rpc(&mut core, "workspace.open", json!({"path":root}))
            .error
            .is_none()
    );
    assert!(core.terminal.as_ref().unwrap().is_open(&id));
    core.terminal.as_mut().unwrap().close(&id).unwrap();
    wait_closed(&receiver, &[id]);
}

#[test]
fn failed_stop_and_transition_preserve_the_current_owner() {
    let (mut core, receiver, root) = core_fixture("workspace-refusal");
    let manager = core.terminal.as_mut().unwrap();
    let id = manager.open_with_shell(&root, "sh").unwrap();
    let (exited, cleanup) = refuse(manager, &id);
    core.run_terminal = Some(id.clone());
    assert!(rpc(&mut core, "run.stop", json!({})).error.is_some());
    assert_eq!(core.run_terminal.as_deref(), Some(id.as_str()));
    let next = root.join("next");
    std::fs::create_dir(&next).unwrap();
    assert!(
        rpc(&mut core, "workspace.open", json!({"path":next}))
            .error
            .is_some()
    );
    assert_eq!(core.workspace_root().as_deref(), Some(root.as_path()));
    assert_eq!(core.run_terminal.as_deref(), Some(id.as_str()));
    assert!(rpc(&mut core, "workspace.close", json!({})).error.is_some());
    assert_eq!(core.workspace_root().as_deref(), Some(root.as_path()));
    let created = root.join("created-before-refusal");
    let refusal = rpc(
        &mut core,
        "workspace.createProject",
        json!({"parent":root,"name":"created-before-refusal","template":"cppCmake"}),
    );
    assert!(created.join("CMakeLists.txt").is_file());
    assert!(
        refusal
            .error
            .unwrap()
            .message
            .contains(&created.display().to_string())
    );
    assert_eq!(core.workspace_root().as_deref(), Some(root.as_path()));
    exited.store(true, Ordering::SeqCst);
    assert!(
        rpc(&mut core, "workspace.open", json!({"path":next}))
            .error
            .is_none()
    );
    assert_eq!(core.workspace_root().as_deref(), Some(next.as_path()));
    assert!(core.run_terminal.is_none());
    drop(cleanup);
    wait_closed(&receiver, &[id]);
}

#[test]
fn creating_a_project_also_closes_previous_sessions() {
    let (mut core, receiver, root) = core_fixture("create-workspace");
    let (id, _cleanup) = stubborn(core.terminal.as_mut().unwrap(), &root);
    let created = rpc(
        &mut core,
        "workspace.createProject",
        json!({"parent":root,"name":"created","template":"cppCmake"}),
    );
    assert!(created.error.is_none(), "{created:?}");
    wait_closed(&receiver, &[id]);
    assert_eq!(core.workspace_root(), Some(root.join("created")));
}
