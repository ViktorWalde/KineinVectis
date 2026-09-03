//! Types for the `library.*` domain: the curated C/C++ library catalog.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Whether the package was found on this machine.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LibraryStatus {
    /// A config package exists in a known prefix; `find_package` should find it.
    Detected,
    /// Not found in the known prefixes. **Not** the same as "does not exist":
    /// the user may have a custom `CMAKE_PREFIX_PATH`.
    NotDetected,
}

/// One audited catalog entry, already crossed with this machine.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryInfo {
    /// Stable id used by `library.plan`.
    pub id: String,
    /// Project name.
    pub name: String,
    /// One sentence saying what it does.
    pub summary: String,
    /// List category.
    pub category: String,
    /// SPDX identifier, verified against the project's own licence file.
    pub license: String,
    /// Pinned tag — never "latest".
    pub pinned_version: String,
    /// Release date of the pinned version (ISO), so age is visible.
    pub released_at: String,
    /// Official documentation.
    pub documentation: String,
    /// Repository.
    pub repository: String,
    /// Whether this machine already has it.
    pub status: LibraryStatus,
    /// The STRONG signal, when there is one: the library became part of the ISO
    /// standard, or passed Boost's formal review.
    ///
    /// **There is no official certification body for C++ libraries.** WG21
    /// standardises the language and the standard library; the Standard C++
    /// Foundation supports the community and states its goal as reducing the
    /// barriers to *adopting* libraries into the Standard itself — not
    /// certifying third-party ones. This field records the signal that really
    /// exists instead of inventing a seal nobody issues.
    pub standard_lineage: Option<String>,
}

/// One step of a plan, naming the Configuration Action that would run it.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryStep {
    /// Configuration Action id that performs this step.
    pub action_id: String,
    /// What it does, in the user's words.
    pub summary: String,
    /// Parameters to hand to that action.
    ///
    /// Without these the UI could name the action but not run it, and the
    /// button would have to rebuild them — putting the same knowledge in two
    /// places. The plan carries what it decided.
    pub params: BTreeMap<String, String>,
}

/// What it would take for a target to use a library.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryPlan {
    /// Library id.
    pub id: String,
    /// `CMake` target that would link it.
    pub target: String,
    /// `true` = `find_package`; `false` = `FetchContent` with the pinned tag.
    pub uses_find_package: bool,
    /// Whether the package was found on this machine.
    pub detected: bool,
    /// Pinned tag used when fetching.
    pub pinned_version: String,
    /// Targets to link.
    pub targets: Vec<String>,
    /// The steps, in order.
    pub steps: Vec<LibraryStep>,
    /// Where the detection looked — only filled when it found nothing, so the
    /// UI can say WHERE instead of just "not found".
    pub searched_paths: Vec<String>,
}

/// Parameters for `library.list`.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LibraryListParams {}

/// Result payload for `library.list`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryListResult {
    /// The catalog, already crossed with this machine.
    pub libraries: Vec<LibraryInfo>,
}

/// Parameters for `library.plan`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LibraryPlanParams {
    /// Library id from the catalog.
    pub id: String,
    /// `CMake` target that would link it.
    pub target: String,
}
