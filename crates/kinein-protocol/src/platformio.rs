//! `platformio.*`: the PIO Home server the IDE shows (`0.191.0`, roadmap 52
//! §14.45, `03-dominios/platformio.md`).

use serde::{Deserialize, Serialize};

/// Result payload for `platformio.homeStart`: the session page, served only
/// on `127.0.0.1`.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformioHomeStartResult {
    /// `http://127.0.0.1:<port>/session/<session>/`; the server root answers 403.
    pub url: String,
    /// The port the server listens on.
    pub port: u16,
}

/// Result payload for `platformio.homeStop`.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformioHomeStopResult {
    /// `false` when no server was running.
    pub stopped: bool,
}

/// Payload of `event.platformio.homeExited`: the server ended without a
/// `platformio.homeStop` (idle timeout or failure).
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformioHomeExitedParams {
    /// Exit code, or `None` when a signal ended the process.
    pub code: Option<i32>,
}
