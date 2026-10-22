//! Schema and edit invariants run on every platform; no serial/device execution.
use crate::{flash::FlashCommand, runconfig};
use kinein_protocol::{FlashIntent, FlashProposalParams};
use serde_json::json;
use std::{fs, path::PathBuf};

fn fixture(name: &str) -> (PathBuf, FlashIntent, String) {
    let root = std::env::temp_dir()
        .join("kinein-core-tests")
        .join(format!("{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join(".kinein")).unwrap();
    let intent = FlashIntent {
        version: 1,
        workspace: root.display().to_string(),
        selection: FlashProposalParams {
            native: true,
            engine: Some("esptool".to_owned()),
            device: Some("fixture".to_owned()),
            ..FlashProposalParams::default()
        },
        program: std::env::current_exe().unwrap().display().to_string(),
        args: vec!["version".to_owned()],
        device_key: "opaque fixture".to_owned(),
        fingerprint: "a".repeat(64),
    };
    let command = FlashCommand {
        program: intent.program.clone().into(),
        args: intent.args.clone(),
    }
    .preview();
    (root, intent, command)
}

#[test]
fn only_saving_a_native_intent_promotes_schema_and_old_editors_cannot_drop_it() {
    let (root, intent, command) = fixture("native-config-schema");
    runconfig::save(&root, None, "Manual", "echo manual").unwrap();
    let path = runconfig::configs_path(&root);
    let original = fs::read(&path).unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&original).unwrap()["schemaVersion"],
        1
    );
    runconfig::load(&root).unwrap();
    assert_eq!(original, fs::read(&path).unwrap());
    let saved = runconfig::save_with_flash(&root, None, "Native", &command, Some(&intent)).unwrap();
    let id = saved.active_id.unwrap();
    let native_bytes = fs::read(&path).unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&native_bytes).unwrap()["schemaVersion"],
        2
    );
    assert!(runconfig::active_command(&root).is_err());
    assert!(runconfig::save(&root, Some(&id), "Lost intent", &command).is_err());
    assert!(
        runconfig::save_with_flash(
            &root,
            Some(&id),
            "Wrong preview",
            "echo changed",
            Some(&intent)
        )
        .is_err()
    );
    assert_eq!(native_bytes, fs::read(&path).unwrap());
    let renamed =
        runconfig::save_with_flash(&root, Some(&id), "Renamed", &command, Some(&intent)).unwrap();
    assert_eq!(renamed.configs[1].flash.as_ref(), Some(&intent));
    runconfig::set_active(&root, Some("cfg-1")).unwrap();
    assert_eq!(
        runconfig::active_command(&root).unwrap().as_deref(),
        Some("echo manual")
    );
}

#[test]
fn schema_one_never_accepts_a_flash_field_even_null_and_invalid_intent_preserves_storage() {
    let (root, mut intent, command) = fixture("native-config-preserve");
    let path = runconfig::configs_path(&root);
    for value in [json!(null), json!(intent)] {
        let bytes = serde_json::to_vec(&json!({"schemaVersion":1,"configs":[{"id":"cfg-1","name":"x","command":command,"flash":value}]})).unwrap();
        fs::write(&path, &bytes).unwrap();
        assert!(runconfig::load(&root).is_err());
        assert!(runconfig::save(&root, None, "New", "echo new").is_err());
        assert_eq!(bytes, fs::read(&path).unwrap());
    }
    intent.version = 99;
    let bytes = serde_json::to_vec(&json!({"schemaVersion":2,"configs":[{"id":"cfg-1","name":"x","command":command,"flash":intent}]})).unwrap();
    fs::write(&path, &bytes).unwrap();
    assert!(runconfig::load(&root).is_err());
    assert_eq!(bytes, fs::read(&path).unwrap());
}

#[test]
fn an_unavailable_native_provider_does_not_fall_back_to_manual_or_an_unrelated_tool() {
    let (root, _, _) = fixture("native-provider-unavailable");
    let mut core = crate::tests::core_with_empty_search_path("native-provider-unavailable");
    assert!(
        core.handle_request(&kinein_protocol::JsonRpcRequest::new(
            1_i64,
            "workspace.open",
            Some(json!({"path":root}))
        ))
        .response()
        .error
        .is_none()
    );
    let response = core.handle_request(&kinein_protocol::JsonRpcRequest::new(
        2_i64,
        "runConfig.flashProposal",
        Some(json!({"native":true,"engine":"probe-rs"})),
    ));
    let error = response.response().error.as_ref().unwrap();
    assert_eq!(
        error.code,
        kinein_protocol::JsonRpcErrorCode::InvalidRequest
    );
    assert!(error.message.contains("motor ainda"), "{error:?}");
    assert!(!runconfig::configs_path(&root).exists());
}
