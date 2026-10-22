//! Catalogue checksum is authoritative input from its owner, never from RPC argv.
use super::fixture;
use crate::flash::{
    FirmwareImage,
    native::{self, NativeRequest},
};
use kinein_protocol::FlashProposalParams;
use serde_json::json;
use std::{
    fs,
    sync::{Arc, atomic::AtomicBool},
};

#[test]
fn native_catalogue_images_require_the_original_checksum() {
    let fixture = fixture("native-catalog-checksum");
    let image = fixture.root.join("catalog.bin");
    fs::write(&image, "abc").unwrap();
    let request = || NativeRequest {
        model: fixture.core.compute_project_model(&fixture.root),
        context: json!(["catalog fixture"]),
        selection: FlashProposalParams {
            native: true,
            firmware: Some("fixture-catalog-entry".to_owned()),
            device: Some(fixture.port.display().to_string()),
            ..FlashProposalParams::default()
        },
        firmware: Some(FirmwareImage {
            label: "Fixture firmware".to_owned(),
            engine: "esptool".to_owned(),
            offset: Some("0x0".to_owned()),
            chip: Some("esp32c3".to_owned()),
            file: image.clone(),
            size_bytes: 3,
            expected_sha256: Some(
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".to_owned(),
            ),
        }),
        tool: fixture.root.join("bin/esptool"),
    };
    let cancel = Arc::new(AtomicBool::new(false));
    let prepared = native::prepare(&fixture.root, request(), &cancel).unwrap();
    assert!(prepared.unchanged());
    assert_eq!(
        &prepared.intent.args[prepared.intent.args.len() - 3..],
        ["write-flash", "0x0", image.to_str().unwrap()]
    );
    fs::write(&image, "abX").unwrap();
    assert!(!prepared.unchanged());
    let error = native::prepare(&fixture.root, request(), &cancel).unwrap_err();
    assert!(error.to_string().contains("checksum"), "{error}");
    assert!(!fixture.root.join("flash.args").exists());
}

#[test]
fn native_images_larger_than_the_snapshot_limit_are_refused_without_reading_them() {
    let fixture = fixture("native-catalog-size-limit");
    let image = fixture.root.join("large.bin");
    fs::File::create(&image)
        .unwrap()
        .set_len(1024 * 1024 * 1024 + 1)
        .unwrap();
    let request = NativeRequest {
        model: fixture.core.compute_project_model(&fixture.root),
        context: json!([]),
        selection: FlashProposalParams {
            native: true,
            firmware: Some("fixture-catalog-entry".to_owned()),
            device: Some(fixture.port.display().to_string()),
            ..FlashProposalParams::default()
        },
        firmware: Some(FirmwareImage {
            label: "Fixture".to_owned(),
            engine: "esptool".to_owned(),
            offset: Some("0x0".to_owned()),
            chip: Some("esp32c3".to_owned()),
            file: image,
            size_bytes: 1024 * 1024 * 1024 + 1,
            expected_sha256: Some("a".repeat(64)),
        }),
        tool: fixture.root.join("bin/esptool"),
    };
    let error =
        native::prepare(&fixture.root, request, &Arc::new(AtomicBool::new(false))).unwrap_err();
    assert!(error.to_string().contains("1 GiB"), "{error}");
    assert!(!fixture.root.join("flash.args").exists());
}
