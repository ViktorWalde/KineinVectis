//! USB DFU discovery; descriptors are observations, never a flash destination.
use serde::{Deserialize, Serialize};

/// One alternative USB interface announced by dfu-util 0.11.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DfuInterface {
    /// Whether the device is already in DFU mode or running its application.
    pub mode: DfuMode,
    /// USB vendor id, four lowercase hexadecimal digits.
    pub vid: String,
    /// USB product id, four lowercase hexadecimal digits.
    pub pid: String,
    /// USB device release, four lowercase hexadecimal digits.
    pub release: String,
    /// Transient USB device address, not a stable identity.
    pub device_number: u8,
    /// USB configuration descriptor value.
    pub configuration: u8,
    /// USB interface number.
    pub interface: u8,
    /// Observed bus/port path, not a stable device identity.
    pub path: String,
    /// Alternative setting of the interface, not a separate board.
    pub alternate: u8,
    /// Alternative setting description, possibly a `DfuSe` memory map.
    pub name: String,
    /// Serial descriptor; absent when empty or UNKNOWN.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub serial: Option<String>,
}

/// Device mode reported by the tool; discovery does not change it.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DfuMode {
    /// The bootloader's DFU interface.
    Dfu,
    /// The application's Runtime interface.
    Runtime,
}

/// Parameters for `dfu.list`; discovery accepts no selection or flash options.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DfuListParams {}

/// Complete discovery or an explained failure, with diagnostic output either way.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DfuListResult {
    /// Interfaces from a complete listing; empty on any query/format failure.
    pub interfaces: Vec<DfuInterface>,
    /// Whether the executable could start (not a proof of USB access).
    pub tool_available: bool,
    /// Successful, supported listing without ignored diagnostics or partial output.
    pub complete: bool,
    /// Executable selected by the existing detector, when found.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
    /// Combined output, retained up to 64 KiB by the shared query runner.
    pub raw_output: String,
    /// Failure cause or guidance for an empty listing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{DfuInterface, DfuListParams};
    use serde_json::json;

    #[test]
    fn discovery_has_strict_parameters_and_typed_modes() {
        assert!(serde_json::from_value::<DfuListParams>(json!({})).is_ok());
        assert!(serde_json::from_value::<DfuListParams>(json!({"device":"auto"})).is_err());
        let value = json!({"mode":"runtime", "vid":"0483", "pid":"df11", "release":"2200",
            "deviceNumber":2, "configuration":1, "interface":0, "path":"1-2", "alternate":0,
            "name":"DFU"});
        let interface: DfuInterface = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(interface).unwrap(), value);
        let mut invalid = value;
        invalid["mode"] = json!("unknown");
        assert!(serde_json::from_value::<DfuInterface>(invalid).is_err());
    }
}
