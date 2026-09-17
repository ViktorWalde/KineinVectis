//! Types for the `serial.*` domain: USB serial ports on this machine.
//!
//! The serial port is the one channel every bare-metal family shares
//! (`DocsPublic/integracoes/38` §2): ROM bootloader, console and the DTR/RTS reset
//! lines all ride on it. This domain only ENUMERATES and DESCRIBES — it never
//! opens a port, because opening one asserts DTR/RTS on most bridges and that
//! resets the board.

use serde::{Deserialize, Serialize};

/// What kind of tty the kernel exposed.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SerialPortKind {
    /// `/dev/ttyUSB*`: a USB-to-UART bridge (`CP210x`, FTDI, CH340...). The chip
    /// behind it is unknown to the USB descriptor.
    UsbUartBridge,
    /// `/dev/ttyACM*`: a CDC-ACM device — the chip's own USB (Espressif
    /// USB Serial/JTAG, Pico stdio) or a probe's virtual COM port.
    UsbCdc,
}

/// Whether the current user can read and write the device node, and why not.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SerialAccess {
    /// `access(2)` with read+write succeeded — honours ACLs, which is how
    /// `TAG+="uaccess"` grants the seat user access without any group.
    pub readable_writable: bool,
    /// Symbolic mode of the node, as `ls -l` prints it (`crw-rw----`).
    pub mode: String,
    /// Owning group name, when it could be resolved (`dialout`, `uucp`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// What to do when access is denied. Never a `sudo` invocation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

/// What udev told `ModemManager` about this port.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModemManagerState {
    /// `ID_MM_CANDIDATE=1`: `ModemManager` may probe this tty when it appears.
    pub candidate: bool,
    /// `ID_MM_DEVICE_IGNORE=1`: a rule already told `ModemManager` to stay away.
    pub ignored: bool,
    /// A `ModemManager` process exists right now.
    pub running: bool,
}

/// One USB serial port.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SerialPortInfo {
    /// Device node, `/dev/ttyUSB0`.
    pub device: String,
    /// Stable path under `/dev/serial/by-id`, when udev created one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub by_id: Option<String>,
    /// Bridge (`ttyUSB*`) or CDC (`ttyACM*`).
    pub kind: SerialPortKind,
    /// USB vendor id (hex, no prefix), from sysfs.
    pub vid: String,
    /// USB product id.
    pub pid: String,
    /// USB `manufacturer` string descriptor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manufacturer: Option<String>,
    /// USB `product` string descriptor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub product: Option<String>,
    /// USB `serial` string descriptor — the CP2102 ships `0001` on every unit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub serial: Option<String>,
    /// USB interface number (`00`, `01`): tells the two ports of an FT2232 or
    /// the VCP of an ST-Link apart.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interface: Option<String>,
    /// Kernel driver bound to the port (`cp210x`, `ch341`, `ftdi_sio`,
    /// `cdc_acm`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub driver: Option<String>,
    /// What the VID:PID says about the LINK — never about the chip behind a
    /// bridge. `None` when the pair is not in the table.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
    /// Can this user read and write the node, measured — never guessed.
    pub access: SerialAccess,
    /// `None` when `udevadm` is not available to ask.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modem_manager: Option<ModemManagerState>,
}

/// Result payload for `serial.list`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SerialListResult {
    /// Ports found, sorted by device path.
    pub ports: Vec<SerialPortInfo>,
    /// What to do when the list is empty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

/// Parameters for `serial.list`.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SerialListParams {}

/// Parameters for `serial.monitor`: open a monitor process on a port, in a
/// terminal tab.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SerialMonitorParams {
    /// Device node, `/dev/ttyUSB0`.
    pub device: String,
    /// Baud rate; absent, 115200 — what ESP32, Pico stdio and most firmware
    /// default to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baud: Option<u32>,
}

/// Result payload for `serial.monitor`: a terminal session, like
/// `terminal.open`, plus what runs in it.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SerialMonitorResult {
    /// Terminal session id.
    pub id: String,
    /// The command line running in the tab.
    pub command: String,
    /// Catalogue id of the tool chosen (`tio`, `picocom`, `minicom`, `espflash`).
    pub tool: String,
}

/// Parameters for `serial.identify` (`0.112.0`, E5 of `integracoes/38` §6):
/// ask `esptool` what chip and flash sit behind a port.
///
/// This is the one `serial.*` request that OPENS the port — and therefore
/// resets the board (DTR/RTS into the ROM bootloader). It runs only on an
/// explicit gesture, never when the panel opens.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SerialIdentifyParams {
    /// Device node, `/dev/ttyUSB0`. Must be a port `serial.list` returned and
    /// the user can read/write.
    pub device: String,
    /// `esptool` executable to use; absent, the one found on `PATH`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
}

/// Result payload for `serial.identify`: the job that is now talking to the
/// board. The outcome arrives as `event.serial.identified`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SerialIdentifyResult {
    /// Job id (`event.job.*`).
    pub job_id: String,
    /// The command line the job runs.
    pub command: String,
}

/// What `esptool flash-id` said about the chip and its flash. Every field is
/// what the tool printed, parsed line by line; a line the parser does not
/// know is ignored and stays in `raw`.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SerialIdentity {
    /// Chip key as the kit/IDF want it (`esp32c3`), derived from `Chip type:`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chip: Option<String>,
    /// `Chip type:` verbatim (`ESP32-C3 (QFN32) (revision v0.4)`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chip_description: Option<String>,
    /// `Features:` split on commas.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub features: Vec<String>,
    /// `Crystal frequency:` (`40MHz`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crystal: Option<String>,
    /// `USB mode:` when the chip has a native USB.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usb_mode: Option<String>,
    /// `MAC:` (`aa:bb:cc:dd:ee:ff`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mac: Option<String>,
    /// `Manufacturer:` of the SPI flash (JEDEC id, hex).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flash_manufacturer: Option<String>,
    /// `Device:` of the SPI flash (hex).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flash_device: Option<String>,
    /// `Detected flash size:` verbatim (`4MB`); absent when `Unknown`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flash_size: Option<String>,
    /// `flash_size` in bytes, when it parses (`4MB` → 4194304).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flash_size_bytes: Option<u64>,
}

/// `event.serial.identified`: the outcome of a `serial.identify` job.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SerialIdentifiedEvent {
    /// The job that ran.
    pub job_id: String,
    /// The port that was asked.
    pub device: String,
    /// The command line that ran (v5 `flash-id`, or the v4 `flash_id` retry).
    pub command: String,
    /// The tool answered and at least the chip was read.
    pub success: bool,
    /// Why not, when `success` is false: the tool's last lines, the timeout.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// What was read, when `success`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity: Option<SerialIdentity>,
    /// The kit the chip SUGGESTS (`project.model`'s target tables: family,
    /// flash/monitor/debug engines). A suggestion: applying it to the kit is
    /// the user's click (`toolchain.setKit`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<crate::TargetModel>,
    /// Everything the tool printed (stdout+stderr), for the panel to show
    /// when the parser understood nothing.
    pub raw: String,
}

/// Parameters for `serial.access` (`0.114.0`, E2 of `integracoes/38` §6):
/// permission diagnosis per channel, with the official fix.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SerialAccessParams {
    /// One port to diagnose; absent = every port `serial.list` sees (plus the
    /// machine-wide probe channel either way).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device: Option<String>,
}

/// Which channel a diagnosis is about.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AccessChannelKind {
    /// Read/write on a tty node: group membership or a udev `uaccess` ACL.
    Serial,
    /// A udev rule for debug probes (probe-rs, `OpenOCD`, ST-Link).
    Probe,
    /// `ModemManager` probing the tty right after plug.
    ModemManager,
}

/// One step of an official fix, as `setup.*` presents it: the IDE writes the
/// command into its own terminal; the user presses Enter. Never `sudo` run
/// by the IDE.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessFix {
    /// Steps in order.
    pub steps: Vec<crate::SetupStep>,
    /// The official page (or the shipped file) the steps were taken from.
    pub source_url: String,
    /// When that source was checked (ISO date).
    pub checked_on: String,
}

/// The diagnosis of one channel.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessChannel {
    /// What this is about.
    pub kind: AccessChannelKind,
    /// The port, for per-port channels; absent for the machine-wide probe rule.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device: Option<String>,
    /// Nothing to do.
    pub ok: bool,
    /// What was measured, in one sentence — present whether `ok` or not.
    pub detail: String,
    /// What is wrong, when not `ok`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub problem: Option<String>,
    /// The official fix, when not `ok` and one is known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fix: Option<AccessFix>,
    /// What the distro/udev already did that makes this `ok` (a shipped rule,
    /// an ACL), when that is the reason.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub distro_did_it: Option<String>,
}

/// Result of `serial.access`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SerialAccessResult {
    /// Per-port serial and `ModemManager` channels, then the probe channel.
    pub channels: Vec<AccessChannel>,
}

/// What `serial.files` does on the board (`0.116.0`, C2 of `roadmaps/41`
/// bloco C): the five gestures of a "files on device" panel, each one
/// `mpremote fs <cmd>`.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SerialFilesAction {
    /// `fs ls :<path>` — the entries of a directory (root when `path` is
    /// absent). Interrupts the running program (raw REPL); writes nothing.
    List,
    /// `fs cp :<path> <local>` — download one file into `local`.
    Get,
    /// `fs cp <local> :<path>` — upload `local` as `path`. WRITES the board.
    Put,
    /// `fs rm :<path>` — delete one file. WRITES the board.
    Rm,
    /// `fs mkdir :<path>` — create a directory. WRITES the board.
    Mkdir,
}

impl SerialFilesAction {
    /// The `mpremote fs` sub-command.
    #[must_use]
    pub const fn fs_command(self) -> &'static str {
        match self {
            Self::List => "ls",
            Self::Get | Self::Put => "cp",
            Self::Rm => "rm",
            Self::Mkdir => "mkdir",
        }
    }

    /// Whether this writes the board's flash file system.
    #[must_use]
    pub const fn writes_board(self) -> bool {
        matches!(self, Self::Put | Self::Rm | Self::Mkdir)
    }
}

/// Parameters for `serial.files`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SerialFilesParams {
    /// Device node, `/dev/ttyUSB0`: a port `serial.list` returned and the
    /// user can read/write.
    pub device: String,
    /// What to do.
    pub action: SerialFilesAction,
    /// Path ON THE BOARD (`main.py`, `lib/wifi.py`); no leading `:`. Absent
    /// only for `list` (the root).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Path ON THIS MACHINE for `get` (where the file lands) and `put` (what
    /// is sent). Relative paths are under the workspace root.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local: Option<String>,
    /// `mpremote` executable to use; absent, the one found on `PATH`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
}

/// Result payload for `serial.files`: the job now talking to the board. The
/// outcome arrives as `event.serial.files`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SerialFilesResult {
    /// Job id (`event.job.*`).
    pub job_id: String,
    /// The command line the job runs.
    pub command: String,
}

/// One entry of `fs ls`, as `mpremote` prints it (`{size:12} {name}[/]`).
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SerialFileEntry {
    /// Name without the trailing `/`.
    pub name: String,
    /// Size in bytes (0 for directories on most ports).
    pub size: u64,
    /// The entry is a directory (`mpremote` appends `/`).
    pub directory: bool,
}

/// `event.serial.files`: the outcome of a `serial.files` job.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SerialFilesEvent {
    /// The job that ran.
    pub job_id: String,
    /// The port that was asked.
    pub device: String,
    /// What ran.
    pub action: SerialFilesAction,
    /// The board path that was asked (empty = root).
    pub path: String,
    /// The command line that ran.
    pub command: String,
    /// `mpremote` exited 0.
    pub success: bool,
    /// Why not: the `mpremote: …` line, the timeout, the cancellation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// The listing, for `list` when `success`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entries: Option<Vec<SerialFileEntry>>,
    /// The resolved local path, for `get`/`put`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local: Option<String>,
    /// Everything the tool printed (stdout+stderr).
    pub raw: String,
}

#[cfg(test)]
mod tests {
    use super::{SerialFilesAction, SerialFilesParams};

    /// The contract is strict: unknown fields are refused, `action` is one
    /// of the five gestures, `path`/`local` are optional.
    #[test]
    fn files_params_parse_strictly() {
        let p: SerialFilesParams =
            serde_json::from_str(r#"{"device":"/dev/ttyUSB0","action":"list"}"#).unwrap();
        assert_eq!(p.action, SerialFilesAction::List);
        assert_eq!(p.path, None);
        let p: SerialFilesParams = serde_json::from_str(
            r#"{"device":"/dev/ttyUSB0","action":"put","path":"main.py","local":"main.py"}"#,
        )
        .unwrap();
        assert_eq!(p.action, SerialFilesAction::Put);
        assert!(p.action.writes_board());
        assert_eq!(p.action.fs_command(), "cp");
        assert!(!SerialFilesAction::Get.writes_board());
        assert!(
            serde_json::from_str::<SerialFilesParams>(
                r#"{"device":"/dev/ttyUSB0","action":"list","baud":9600}"#
            )
            .is_err()
        );
        assert!(
            serde_json::from_str::<SerialFilesParams>(
                r#"{"device":"/dev/ttyUSB0","action":"tree"}"#
            )
            .is_err()
        );
    }
}
