//! O `compiledb` do `PlatformIO` como Configuration Action (0.4.0e,
//! `roadmaps/52` §14.5), pelo despacho real e com um `pio` FALSO que grava
//! o argv e escreve a CDB onde o real escreve (a raiz, medido em 2026-10-16
//! com o Core 6.2.0). Prova a acao na lista (recomendada sem CDB), a recusa
//! sem confianca e sem `pio`, o argv `run -t compiledb` na raiz, a saida no
//! job e o contexto do indice relido com a unidade nova.

use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use kinein_protocol::{JsonRpcErrorCode, JsonRpcRequest, JsonRpcResponse};
use serde_json::{Value, json};

struct Project {
    core: crate::Core,
    base: PathBuf,
    root: PathBuf,
    events: Receiver<JsonRpcRequest>,
}

impl Project {
    fn new(name: &str) -> Self {
        let base = std::env::temp_dir()
            .join("kinein-core-tests")
            .join(format!("{}-platformio-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        for dir in ["projeto/src", "bin", "home"] {
            std::fs::create_dir_all(base.join(dir)).unwrap();
        }
        let base = crate::platform::canonicalize(&base).unwrap();
        let root = base.join("projeto");
        std::fs::write(
            root.join("platformio.ini"),
            "[env:native]\nplatform = native\n",
        )
        .unwrap();
        std::fs::write(root.join("src/main.c"), "int main(void) { return 0; }\n").unwrap();
        let (sender, events) = std::sync::mpsc::channel();
        let mut core = crate::Core::with_detector(crate::tools::ToolDetector::with_search_path(
            base.join("bin"),
        ))
        .with_home(base.join("home"));
        core.enable_lsp(sender);
        let mut project = Self {
            core,
            base,
            root,
            events,
        };
        let path = project.root.display().to_string();
        assert!(
            project
                .rpc("workspace.open", json!({ "path": path }))
                .error
                .is_none()
        );
        project.wait_for(|event| event.method == "event.index.finished");
        project
    }

    fn rpc(&mut self, method: &str, params: Value) -> JsonRpcResponse {
        self.core
            .handle_request(&JsonRpcRequest::new(3_i64, method, Some(params)))
            .response()
            .clone()
    }

    fn apply(&mut self) -> JsonRpcResponse {
        self.rpc(
            "configAction.apply",
            json!({ "id": "platformio.compiledb" }),
        )
    }

    fn state(&mut self) -> Value {
        let list = self.rpc("configAction.list", json!({})).result.unwrap();
        list["actions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|action| action["id"] == "platformio.compiledb")
            .unwrap_or_else(|| panic!("a acao falta na lista: {list}"))["state"]
            .clone()
    }

    /// O loop real: cada evento passa pelo `observe_notification`.
    fn wait_for(&mut self, until: impl Fn(&JsonRpcRequest) -> bool) -> Vec<JsonRpcRequest> {
        let deadline = Instant::now() + Duration::from_secs(20);
        let mut seen = Vec::new();
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            let event = self.events.recv_timeout(left).expect("evento no prazo");
            self.core.observe_notification(&event);
            let done = until(&event);
            seen.push(event);
            if done {
                return seen;
            }
        }
    }
}

fn pio_fake(base: &Path) {
    crate::write_executable(
        base.join("bin/pio"),
        format!(
            "#!/bin/sh\necho \"$*\" >> '{}'\necho 'Building compilation database'\n\
             printf '[{{\"directory\":\"%s\",\"arguments\":[\"cc\",\"-DPIO_FALSO\",\"-c\",\
             \"src/main.c\"],\"file\":\"src/main.c\"}}]' \"$PWD\" > compile_commands.json\n",
            base.join("argv.log").display()
        ),
    );
}

#[test]
#[cfg_attr(
    windows,
    ignore = "W10: embarcados no Windows (COMx, gravadores, frameworks) sao da W10 (D10, 60 §3.3)"
)]
fn the_compiledb_action_runs_pio_and_hands_the_new_cdb_to_the_index() {
    let _serial = crate::serialize_executables();
    let mut project = Project::new("compiledb");
    assert_eq!(project.state(), "recommended", "sem CDB, ela e' a sugerida");
    let before = project
        .rpc("index.context", json!({ "path": "src/main.c" }))
        .result
        .unwrap();
    assert!(
        before["hint"]
            .as_str()
            .unwrap()
            .contains("\"Gerar compile_commands.json\" (PlatformIO)"),
        "a dica aponta a acao, nao o cmake: {before}"
    );

    // Sem confianca, nada roda: o `pio` executa os scripts do projeto.
    let refused = project.apply().error.expect("projeto nao confiavel");
    assert_eq!(refused.code, JsonRpcErrorCode::InvalidRequest);
    assert!(refused.message.contains("confiável"), "{}", refused.message);
    assert!(project.rpc("workspace.trust", json!({})).error.is_none());

    // Sem `pio`, a recusa do `build.run`: a ferramenta, com o framework.
    let missing = project.apply().error.expect("sem pio");
    assert_eq!(missing.code, JsonRpcErrorCode::ToolNotFound);
    assert_eq!(missing.details.as_ref().unwrap()["framework"], "PlatformIO");

    pio_fake(&project.base);
    let started = project.apply();
    let job_id = started.result.expect("o job sobe")["jobId"]
        .as_str()
        .unwrap()
        .to_owned();
    let events = project.wait_for(|event| {
        event.method == "event.job.finished"
            && event.params.as_ref().unwrap()["jobId"] == job_id.as_str()
    });
    let finished = events.last().unwrap().params.clone().unwrap();
    assert_eq!(finished["status"], "success", "{finished}");
    assert!(
        events.iter().any(|event| event.method == "event.job.output"
            && event.params.as_ref().unwrap()["line"] == "Building compilation database"),
        "a saida do pio vai para o job"
    );
    let argv = std::fs::read_to_string(project.base.join("argv.log")).unwrap();
    assert_eq!(argv, "run -t compiledb\n");
    assert!(
        project.root.join("compile_commands.json").is_file(),
        "o pio roda NA RAIZ"
    );

    // O contexto relido traz a unidade da CDB nova, e a acao deixa de ser
    // a sugerida.
    project.wait_for(|event| event.method == "event.index.finished");
    let context = project
        .rpc("index.context", json!({ "path": "src/main.c" }))
        .result
        .unwrap();
    assert_eq!(
        context["unit"]["defines"],
        json!(["PIO_FALSO"]),
        "{context}"
    );
    assert_eq!(context["effective"]["flagsOrigin"], "cdb", "{context}");
    assert!(context.get("hint").is_none(), "{context}");
    assert_ne!(project.state(), "recommended");
}
