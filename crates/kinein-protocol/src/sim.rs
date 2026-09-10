//! Types for the `sim.*` domain: simulation by concept.
//!
//! The shape comes from `docs/arquitetura/34-simulacao-por-conceito.md`, and one
//! decision governs every type here (§2.1, author 2026-09-05): **nothing is
//! guessed**. The IDE computes and shows; the user picks and fills. That is why
//! every field that decides a result is required rather than defaulted, and why
//! the binding between formula variables and physical quantities is data the
//! user supplies instead of something the core matches by name.

use serde::{Deserialize, Serialize};

/// The mathematical form a concept reduces to — the closed, small layer that
/// the engine actually solves (`arquitetura/34` §4.1).
///
/// The catalog's upper layer (many named concepts) maps onto this lower layer
/// (few forms), which is what makes "every concept" affordable: a new concept
/// is a catalog entry, not new engine code.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SimForm {
    /// `y = f(parameters)` — direct evaluation, no integrator. Carries 66 of
    /// the 100 catalogued concepts (`arquitetura/34` appendix A).
    Algebraic,
    /// `dy/dt = f(t, y)` — one state variable.
    Ode1,
    /// `d2y/dt2 = f(t, y, dy/dt)` — the shape of Newtonian mechanics.
    Ode2,
    /// `dY/dt = F(t, Y)` with vector `Y` — several bodies or degrees.
    OdeSystem,
    /// Field equations over space: wave, heat, Laplace, Schrodinger.
    Pde,
}

/// The view a concept opens in.
///
/// The concept DECLARES it and the user may change it — the single documented
/// exception to §2.1, recorded in `roadmaps/31` §18.3.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SimView {
    /// Curves over axes: 68 of the 100 catalogued concepts.
    Plot2d,
    /// Trajectory in x, y, z: 24 of 100.
    Space3d,
    /// Scalar field over a mesh: 8 of 100.
    Field,
}

/// A physical quantity a concept declares, and that the user binds a formula
/// variable to.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimQuantity {
    /// Stable id used by the binding.
    pub id: String,
    /// What it is, in the user's words: "posição", "constante elástica".
    pub label: String,
    /// Unit LABEL, e.g. `m/s^2`. Declared, shown and stored.
    ///
    /// Dimensional coherence is checked by the oracle when it is present
    /// (`arquitetura/34` §7.2); this string is what the screen shows either way.
    pub unit: String,
    /// Whether the concept cannot be expressed without it.
    pub required: bool,
}

/// One entry of the upper catalog layer: a concept the user picks by name.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimConcept {
    /// Stable id.
    pub id: String,
    /// The name the user knows it by.
    pub name: String,
    /// One sentence saying what it models.
    pub summary: String,
    /// Course it belongs to, for grouping: "Física I", "Cálculo III".
    pub course: String,
    /// The form that solves it.
    pub form: SimForm,
    /// The view it opens in; changeable by the user.
    pub view: SimView,
    /// The quantities it declares, required and optional.
    pub quantities: Vec<SimQuantity>,
    /// The ORDERED state components, for the `odeSystem` form only.
    ///
    /// Empty for every other form. The order is the concept's and it is what
    /// the run's per-component formulas and the trail samples line up with —
    /// the core never infers which formula belongs to which component
    /// (`arquitetura/34` §13.2).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub components: Vec<SimSystemComponent>,
    /// Which components are a position/velocity pair, by index into
    /// [`Self::components`].
    ///
    /// DECLARED by the concept, never deduced. The symplectic method is only
    /// definable when this pairing exists, because the method updates the
    /// velocity first and then moves the position with the NEW velocity; in a
    /// general first-order system that pair does not exist. Deducing it by name
    /// or by position is exactly the deduction that fails silently, so a
    /// concept that does not declare it simply does not offer the method
    /// (`arquitetura/34` §13.3).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pairing: Vec<SimPair>,
    /// The two components the trajectory view plots against each other.
    ///
    /// `None` when the concept has no natural plane; the screen then offers
    /// only the components-over-time mode.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plane: Option<SimPair>,
    /// Quantities the physics CONSERVES, when the concept declares any.
    ///
    /// This is the accuracy signal that the scalar forms did not need and the
    /// system form cannot do without: an orbit has no usable closed form in
    /// general and a double pendulum has none at all, but energy drift is
    /// measurable with no oracle whatsoever. Measured on the circular orbit
    /// (`../roadmaps/31` §19.1.1): 2.032e-1 with explicit Euler against
    /// 2.800e-10 with the symplectic one — the difference between an orbit that
    /// spirals out and one that closes.
    ///
    /// A conserved invariant does NOT mean a right result: an error that
    /// respects the symmetry passes through it. The screen says so.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub invariants: Vec<SimInvariantInfo>,
    /// Whether a closed-form solution is known for it.
    ///
    /// True for 88 of the 100 catalogued concepts, and it is this number that
    /// lets the IDE put the true value beside the computed one instead of
    /// promising exactness (`arquitetura/34` §7).
    pub closed_form: bool,
    /// Where the formulation came from, with a date — same rule as `setup.list`:
    /// without a source the IDE does not assert.
    pub source: String,
}

/// One state component of an `odeSystem` concept.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimSystemComponent {
    /// Stable id, used by the per-component formula and by the bindings.
    pub id: String,
    /// What it is, in the user's words: "posição x", "velocidade angular 1".
    pub label: String,
    /// Unit LABEL, e.g. `m/s`.
    pub unit: String,
}

/// Two component indices that belong together.
///
/// Used for the position/velocity pairing and for the trajectory plane. Indices
/// are into [`SimConcept::components`].
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimPair {
    /// First component: the position, or the horizontal axis.
    pub first: usize,
    /// Second component: the velocity, or the vertical axis.
    pub second: usize,
}

/// An invariant the concept declares, as the screen names it.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimInvariantInfo {
    /// Stable id.
    pub id: String,
    /// What it is: "energia mecânica", "momento angular".
    pub label: String,
    /// Unit LABEL.
    pub unit: String,
}

/// What happened to one invariant over a run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimInvariantDrift {
    /// The [`SimInvariantInfo::id`] this is about.
    pub id: String,
    /// Its value at t = 0.
    pub initial: f64,
    /// Its value at the end of the run.
    pub final_value: f64,
    /// `|final - initial|`. The screen shows it as DRIFT, never as "error".
    pub drift: f64,
}

/// The user binding one formula variable to one declared quantity.
///
/// This is the type that keeps the core from matching by name. It exists
/// because matching `x` in the formula to the quantity called `x` is a
/// deduction, and a deduction that fails silently: name something `x` that is
/// not position and the physics is wrong while the table looks right
/// (`arquitetura/34` §5.0).
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimBinding {
    /// The variable as it appears in the formula.
    pub variable: String,
    /// The `SimQuantity::id` the user says it is.
    pub quantity: String,
}

/// Params of `sim.catalog`: the concepts the IDE offers.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimCatalogParams {
    /// Optional course filter; `None` lists everything.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub course: Option<String>,
}

/// Params of `sim.inspectFormula`: what variables does this formula use?
///
/// Deliberately does NOT take the bindings: this is the step that tells the
/// user what there is to bind.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimInspectParams {
    /// The formula as typed.
    pub formula: String,
}

/// Result of `sim.inspectFormula`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimInspectResult {
    /// The variables found, in the order the evaluator reports them.
    ///
    /// The order is the crate's, and it is ALPHABETICAL rather than the order
    /// they appear in the formula. Nothing downstream may depend on it — the
    /// evaluation vector is assembled from [`SimBinding`], never from position
    /// (ADR-0006, trap 1).
    pub variables: Vec<String>,
}

/// Params of `sim.checkFormula`: does this formula fit the chosen concept?
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimCheckParams {
    /// The concept the user picked.
    pub concept: String,
    /// The formula as typed.
    pub formula: String,
    /// What the user says each variable is. Required: the core never guesses.
    pub bindings: Vec<SimBinding>,
}

/// What the check found — one variant per thing that can be wrong, so the UI
/// never has to match on message text.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum SimCheckIssue {
    /// The formula does not parse. Carries the IDE's OWN message: the crate's
    /// error text contains a raw pointer address and is never forwarded
    /// (ADR-0006, trap 4).
    ParseFailed {
        /// The IDE's own wording of what is wrong with the expression.
        message: String,
    },
    /// A quantity the concept requires has no variable bound to it.
    MissingQuantity {
        /// The `SimQuantity::id` that nothing was bound to.
        quantity: String,
        /// Its human label, so the message can name it in the user's words.
        label: String,
    },
    /// A variable in the formula was left without a role.
    ///
    /// This is also how lowercase `pi` surfaces: the evaluator treats it as a
    /// free variable, so it shows up here as a question instead of a silent
    /// unknown (ADR-0006, trap 2).
    UnboundVariable {
        /// The variable in the formula that was left without a role.
        variable: String,
    },
    /// A binding points at a quantity the concept does not declare.
    UnknownQuantity {
        /// The variable whose binding points nowhere.
        variable: String,
        /// The quantity id that the concept does not declare.
        quantity: String,
    },
    /// Two variables were bound to the same quantity.
    DuplicateQuantity {
        /// The quantity that more than one variable claims to be.
        quantity: String,
    },
    /// A binding names a variable the formula does not contain.
    VariableNotInFormula {
        /// The variable named by a binding but absent from the formula.
        variable: String,
    },
}

/// Result of `sim.checkFormula`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimCheckResult {
    /// True only when `issues` is empty.
    pub ok: bool,
    /// Everything wrong, at once — not the first thing found. Typing is a live
    /// loop and a list that changes as you type teaches more than one error.
    pub issues: Vec<SimCheckIssue>,
    /// The variables the formula uses, echoed so the UI can render the binding
    /// table from a single response.
    pub variables: Vec<String>,
    /// The warning the author asked to be on screen, not in a footnote
    /// (`arquitetura/34` §5.1): a right concept and a valid formula do not mean
    /// a right result. `None` while there are issues.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caveat: Option<String>,
}

/// A formula the user wrote for ONE state component, with its bindings.
///
/// The component is named, never positional: the core does not assume that the
/// n-th formula belongs to the n-th component (`arquitetura/34` §13.2).
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimComponentFormula {
    /// The [`SimSystemComponent::id`] this formula is the derivative of.
    pub component: String,
    /// The formula as typed.
    pub formula: String,
    /// What each variable in THIS formula is. A variable may be bound to a
    /// state component, to time, or to a parameter quantity.
    pub bindings: Vec<SimBinding>,
}

/// Params de `sim.checkSystem`: as `n` formulas fecham com o conceito?
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimCheckSystemParams {
    /// O conceito escolhido.
    pub concept: String,
    /// Uma formula por componente.
    pub equations: Vec<SimComponentFormula>,
}

/// O que a checagem achou em UM componente.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimComponentCheck {
    /// De que componente e' esta linha.
    pub component: String,
    /// O resultado da checagem daquela formula.
    pub result: SimCheckResult,
}

/// Resultado de `sim.checkSystem`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimCheckSystemResult {
    /// True so' quando toda formula passou e todo componente tem a sua.
    pub ok: bool,
    /// Uma linha por formula recebida.
    pub components: Vec<SimComponentCheck>,
    /// Componentes que o conceito declara e para os quais nao veio formula.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub missing: Vec<String>,
    /// Formulas que apontam para componente que o conceito nao declara.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unknown: Vec<String>,
    /// O aviso da §5.1, quando nao ha' problema nenhum.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caveat: Option<String>,
}
