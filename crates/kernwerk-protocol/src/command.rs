//! Command descriptors surfaced by `command.list`.

use serde::{Deserialize, Serialize};

/// Descriptor exposed to command palettes, menus, buttons, and shortcuts.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandDescriptor {
    /// Stable command identifier.
    pub id: String,
    /// Short title displayed in the UI.
    pub title: String,
    /// Group shown in command palettes and menus.
    pub category: String,
    /// Human-readable command description.
    pub description: String,
    /// Optional default shortcut.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_shortcut: Option<String>,
    /// Whether the command requires an open workspace.
    pub requires_workspace: bool,
}
