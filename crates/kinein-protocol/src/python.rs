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
    /// The native extension this project builds (pybind11/nanobind/PyO3), when
    /// the root files say so (`0.102.0`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_module: Option<PythonNativeModule>,
}

/// A native extension module recognised at the workspace root.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PythonNativeModule {
    /// `pybind11`, `nanobind`, `PyO3`, or `Rust` (maturin/setuptools-rust
    /// without an explicit pyo3 dependency).
    pub kind: String,
    /// What builds and installs it: `maturin`, `scikit-build-core`,
    /// `setuptools-rust`, `setuptools`.
    pub tool: String,
    /// One line per file that proved it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<String>,
    /// The official command to build it into the project environment.
    pub build_hint: String,
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
