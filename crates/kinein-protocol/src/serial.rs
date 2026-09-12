//! Types for the `serial.*` domain: USB serial ports on this machine.
//!
//! The serial port is the one channel every bare-metal family shares
//! (`docs/integracoes/38` §2): ROM bootloader, console and the DTR/RTS reset
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
