//! Configuration model for Kernwerk Studio.
//!
//! Defaults are intentionally strict. Relaxed settings must be explicit so
//! generated projects and the core do not silently drift away from CI-grade
//! behavior.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Project strictness level.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StrictnessLevel {
    /// Maximum safety and diagnostics. This is the default.
    #[default]
    Strict,
    /// Useful for migration work, still biased toward correctness.
    Balanced,
    /// Legacy-friendly mode. Requires an explicit justification in project metadata.
    Relaxed,
}

/// Diagnostic handling policy.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DiagnosticPolicy {
    /// Diagnostics fail the build.
    #[default]
    Deny,
    /// Diagnostics are reported but do not fail the build.
    Warn,
    /// Diagnostics are disabled.
    Allow,
}

/// Rust `unsafe` policy used by the core and Rust templates.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RustUnsafePolicy {
    /// `unsafe` code is rejected.
    #[default]
    Forbid,
    /// `unsafe` may be used only with a recorded design review.
    AllowWithJustification,
}

/// Sanitizer policy for native C++ targets.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SanitizerPolicy {
    /// `AddressSanitizer` and `UndefinedBehaviorSanitizer` are required in debug builds.
    #[default]
    AddressAndUndefinedBehavior,
    /// Sanitizers are opt-in.
    Manual,
}

/// Release hardening policy for native C++ targets.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HardeningPolicy {
    /// Hardened linker/compiler flags and LTO are required.
    #[default]
    Required,
    /// Hardening is selected per target.
    Manual,
}

/// Rust compiler and linting safety settings.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RustCompilerSafety {
    /// Policy for `unsafe` code.
    pub unsafe_policy: RustUnsafePolicy,
    /// Policy for compiler warnings.
    pub warnings: DiagnosticPolicy,
    /// Policy for Clippy diagnostics.
    pub clippy: DiagnosticPolicy,
}

impl Default for RustCompilerSafety {
    fn default() -> Self {
        Self {
            unsafe_policy: RustUnsafePolicy::Forbid,
            warnings: DiagnosticPolicy::Deny,
            clippy: DiagnosticPolicy::Deny,
        }
    }
}

/// C++ compiler, sanitizer, and hardening settings.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CxxCompilerSafety {
    /// Policy for compiler warnings.
    pub warnings: DiagnosticPolicy,
    /// Policy for clang-tidy diagnostics.
    pub clang_tidy: DiagnosticPolicy,
    /// Debug sanitizer policy.
    pub sanitizers: SanitizerPolicy,
    /// Release hardening policy.
    pub hardening: HardeningPolicy,
}

impl Default for CxxCompilerSafety {
    fn default() -> Self {
        Self {
            warnings: DiagnosticPolicy::Deny,
            clang_tidy: DiagnosticPolicy::Deny,
            sanitizers: SanitizerPolicy::AddressAndUndefinedBehavior,
            hardening: HardeningPolicy::Required,
        }
    }
}

/// QML validation settings.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QmlSafety {
    /// Policy for static QML checks.
    pub static_checks: DiagnosticPolicy,
    /// Policy for QML compiler diagnostics.
    pub compiler_diagnostics: DiagnosticPolicy,
}

impl Default for QmlSafety {
    fn default() -> Self {
        Self {
            static_checks: DiagnosticPolicy::Deny,
            compiler_diagnostics: DiagnosticPolicy::Deny,
        }
    }
}

/// Compiler safety profile for all primary Kernwerk languages.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompilerSafety {
    /// Rust compiler and linter policy.
    pub rust: RustCompilerSafety,
    /// C++ compiler and linter policy.
    pub cxx: CxxCompilerSafety,
    /// QML compiler and static validation policy.
    pub qml: QmlSafety,
}

/// Top-level Kernwerk project settings.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSettings {
    /// Project strictness level.
    pub strictness: StrictnessLevel,
    /// Compiler safety settings.
    pub compiler_safety: CompilerSafety,
    /// Whether telemetry is enabled.
    pub telemetry: TelemetrySetting,
    /// Whether external AI calls require explicit user confirmation.
    pub external_ai: ExternalAiSetting,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            strictness: StrictnessLevel::Strict,
            compiler_safety: CompilerSafety::default(),
            telemetry: TelemetrySetting::default(),
            external_ai: ExternalAiSetting::default(),
        }
    }
}

/// Telemetry setting.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TelemetrySetting {
    /// Telemetry is disabled.
    #[default]
    Disabled,
}

/// External AI provider policy.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ExternalAiSetting {
    /// Each external AI request requires an explicit user confirmation.
    #[default]
    RequireConfirmation,
    /// External AI providers are disabled.
    Disabled,
}

#[cfg(test)]
mod tests {
    use super::{
        DiagnosticPolicy, ExternalAiSetting, HardeningPolicy, ProjectSettings, RustUnsafePolicy,
        SanitizerPolicy, StrictnessLevel, TelemetrySetting,
    };

    #[test]
    fn defaults_are_strict_and_private() {
        let settings = ProjectSettings::default();

        assert_eq!(settings.strictness, StrictnessLevel::Strict);
        assert_eq!(
            settings.compiler_safety.rust.unsafe_policy,
            RustUnsafePolicy::Forbid
        );
        assert_eq!(
            settings.compiler_safety.rust.warnings,
            DiagnosticPolicy::Deny
        );
        assert_eq!(
            settings.compiler_safety.cxx.sanitizers,
            SanitizerPolicy::AddressAndUndefinedBehavior
        );
        assert_eq!(
            settings.compiler_safety.cxx.hardening,
            HardeningPolicy::Required
        );
        assert_eq!(settings.telemetry, TelemetrySetting::Disabled);
        assert_eq!(settings.external_ai, ExternalAiSetting::RequireConfirmation);
    }
}
