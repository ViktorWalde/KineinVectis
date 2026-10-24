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

/// Params of `platformio.homeRelease` (`0.192.0`): the page left the screen.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformioHomeReleaseParams {
    /// Seconds until the core stops the server, 0 to 3600; 300 when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after: Option<u32>,
}

/// Result payload for `platformio.homeRelease`.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformioHomeReleaseResult {
    /// `false` when no server was running.
    pub releasing: bool,
}

/// Why `event.platformio.homeExited` was sent (`0.192.0`).
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PlatformioHomeExitReason {
    /// The server or its agent ended on its own.
    #[default]
    Exited,
    /// The deadline of a `platformio.homeRelease` passed and the core
    /// stopped the server.
    Idle,
}

/// Payload of `event.platformio.homeExited`: the server is gone without a
/// `platformio.homeStop`.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformioHomeExitedParams {
    /// Exit code, or `None` when a signal ended the process or the core
    /// stopped it.
    pub code: Option<i32>,
    /// Why it is gone.
    #[serde(default)]
    pub reason: PlatformioHomeExitReason,
}
