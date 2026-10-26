//! Explicit `DfuSe` image and USB destination for a native flash intent.
use serde::{Deserialize, Serialize};

/// An explicitly chosen `DfuSe` file; never inferred from build recency.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DfuFlashSelection {
    /// Regular file inside the workspace, canonicalized in the returned intent.
    pub image: String,
    /// Explicit alternative of a device already in DFU mode.
    pub destination: DfuDestination,
}

/// Observed USB attachment and alternative; never a serial port or row index.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DfuDestination {
    /// Canonical four-digit lowercase hexadecimal vendor id.
    pub vid: String,
    /// Canonical four-digit lowercase hexadecimal product id.
    pub pid: String,
    /// Non-empty serial descriptor, without selector separators or controls.
    pub serial: String,
    /// Canonical bus/port path from the current tool's listing.
    pub path: String,
    /// USB configuration (not changed by dfu-util).
    pub configuration: u8,
    /// DFU interface number.
    pub interface: u8,
    /// Alternative setting, also declared in the `DfuSe` file.
    pub alternate: u8,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn dfu_selection_round_trips_strictly_in_a_native_intent() {
        let selection = json!({"image":"/project/firmware.dfu","destination":{
            "vid":"0483","pid":"df11","serial":"S1","path":"1-2",
            "configuration":1,"interface":0,"alternate":0}});
        let value = json!({"version":1,"workspace":"/project","selection":{"native":true,"engine":"dfu-util","dfu":selection},
            "program":"/tools/dfu-util","args":["--download","/project/firmware.dfu"],"deviceKey":"usb","fingerprint":"a".repeat(64)});
        let intent: crate::FlashIntent = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(intent).unwrap(), value);
        let mut unknown = selection.clone();
        unknown["destination"]["deviceIndex"] = json!(0);
        assert!(serde_json::from_value::<super::DfuFlashSelection>(unknown).is_err());
        let mut unknown = selection.clone();
        unknown["force"] = json!(true);
        assert!(serde_json::from_value::<super::DfuFlashSelection>(unknown).is_err());
        let mut invalid = selection;
        invalid["destination"]["alternate"] = json!(256);
        assert!(serde_json::from_value::<super::DfuFlashSelection>(invalid).is_err());
    }
}
