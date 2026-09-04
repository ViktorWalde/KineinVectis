//! Types for the `setup.*` domain: how to install what is missing.
//!
//! Every command in this domain comes from the tool's OWN official
//! documentation, and every guide carries the URL and the date it was checked.
//! The IDE shows text; it never runs these commands itself.

use serde::{Deserialize, Serialize};

/// One step of an install guide.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupStep {
    /// What this step does, in one sentence.
    ///
    /// Mandatory: a bare command teaches nothing, and this domain exists for
    /// the author who has not done this before.
    pub explanation: String,
    /// The command, exactly as the official source writes it.
    pub command: String,
}

/// The install guide for one tool on one distro family.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupGuide {
    /// Distro family this guide applies to (`debian`, `redhat`, ...).
    pub family: String,
    /// The official page these commands were copied from.
    ///
    /// Shown next to the steps so the author can verify rather than trust.
    pub source_url: String,
    /// When this entry was checked against the source (ISO date).
    pub checked_at: String,
    /// Steps, in the order the source presents them.
    pub steps: Vec<SetupStep>,
}

/// A tool the IDE can help install.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupToolInfo {
    /// Stable id.
    pub id: String,
    /// Project name.
    pub name: String,
    /// What it does, for someone who has not heard of it.
    pub summary: String,
    /// Official site — always shown, including when there is no guide.
    pub website: String,
    /// Whether this machine already has it.
    pub installed: bool,
    /// Step by step for the DETECTED distro family.
    ///
    /// Absent when the official source does not document this family. The UI
    /// then shows the website and says so, instead of translating a command
    /// from another distro — which would be the guess this project forbids.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guide: Option<SetupGuide>,
}

/// Result payload for `setup.list`.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupListResult {
    /// `ID` from `os-release`.
    pub distro_id: String,
    /// `PRETTY_NAME`, as the distro calls itself.
    pub distro_name: String,
    /// Package-manager family the guides were selected by.
    pub family: String,
    /// The tools, with their guides.
    pub tools: Vec<SetupToolInfo>,
}

/// Parameters for `setup.list`.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SetupListParams {}
