//! Types for the `python.*` domain: the Python ENVIRONMENT of the workspace
//! (block B of `roadmaps/41`, chain started on 2026-09-12).
//!
//! The interpreter is the `compile_commands.json` of Python: every other
//! piece (language server, run, tests, debugger) reads it from here.

use serde::{Deserialize, Serialize};

use crate::PythonEnv;

/// Which tool creates the project environment.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PythonEnvironmentTool {
    /// `uv venv .venv` — preferred when `uv` is installed.
    Uv,
    /// `python3 -m venv .venv` — the standard library, always available with a system Python.
    Venv,
}

/// Result payload for `python.status`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PythonStatus {
    /// The interpreter the project resolves to (`roadmaps/29` §4.1 order), when any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interpreter: Option<PythonEnv>,
    /// `true` when the interpreter is a project environment (not the system one).
    pub has_environment: bool,
    /// The tool the IDE would use to create `.venv`, when one exists on this machine.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment_tool: Option<PythonEnvironmentTool>,
    /// Path of `uv`, when detected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uv: Option<String>,
    /// Project files found at the root (`pyproject.toml`, `requirements.txt`, ...).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub project_files: Vec<String>,
    /// What is missing, in words, with the remedy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

/// Parameters for `python.status`.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PythonStatusParams {}

/// Parameters for `python.createEnvironment`.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PythonCreateEnvironmentParams {
    /// Tool to use; omitted = `uv` when detected, else `venv`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool: Option<PythonEnvironmentTool>,
}

/// Payload of `event.python.finished`: the environment job ended.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PythonFinishedEvent {
    /// Job id.
    pub job_id: String,
    /// `true` when the tool exited 0.
    pub success: bool,
    /// Tool that ran.
    pub tool: PythonEnvironmentTool,
    /// The command line, for the log.
    pub command: String,
    /// The environment directory, absolute.
    pub path: String,
}
