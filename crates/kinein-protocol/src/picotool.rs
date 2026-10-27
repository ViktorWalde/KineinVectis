//! Observations from a bounded BOOTSEL query, not a flash authorization.
use serde::{Deserialize, Serialize};

/// One RP2040 whose picotool serial selector was confirmed separately.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PicoDevice {
    /// Reported chip, currently only `rp2040`.
    pub chip: String,
    /// Confirmed `--ser` value; not necessarily a raw USB descriptor.
    pub serial_selector: String,
    /// Capacity reported by picotool, not an independently measured guarantee.
    pub flash_size_bytes: u32,
}

/// The query accepts no destination, force or write flags.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PicoListParams {}

/// Verified observations, or an explained incomplete query with no devices.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PicoListResult {
    /// Empty on every query, format or confirmation failure.
    pub devices: Vec<PicoDevice>,
    /// False when absent or a launch failed; does not prove USB access.
    pub tool_available: bool,
    /// Whether the entire query and every selector confirmation succeeded.
    pub complete: bool,
    /// Executable selected by the existing detector.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
    /// Audited version reported by the tool.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Combined diagnostic output, at most 64 KiB.
    pub raw_output: String,
    /// Cause or guidance, including unsupported versions and access failures.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{PicoDevice, PicoListParams};
    use serde_json::json;
    #[test]
    fn pico_query_is_strict_and_observations_round_trip() {
        assert!(serde_json::from_value::<PicoListParams>(json!({})).is_ok());
        assert!(serde_json::from_value::<PicoListParams>(json!({"force":true})).is_err());
        let value =
            json!({"chip":"rp2040","serialSelector":"E660583883355B21","flashSizeBytes":2_097_152});
        let device: PicoDevice = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(device).unwrap(), value);
        assert!(
            serde_json::from_value::<PicoDevice>(
                json!({"chip":"rp2040","serialSelector":"S","flashSizeBytes":-1})
            )
            .is_err()
        );
    }
}
