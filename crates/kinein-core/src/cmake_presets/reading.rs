//! Why a configure preset of the files is not one `CMake` uses: what reading
//! them left out of the list (C1-24), and what the `CMake` of the project does
//! not list (C1-25).

/// What reading the configure presets left out of the list.
#[derive(Debug, Default)]
pub(super) struct Reading {
    /// Hidden: `CMake` uses them only through `inherits` (C1-27).
    pub hidden: Vec<String>,
    /// `CMake` ignores them: a `$vendor{}` macro.
    pub ignored: Vec<String>,
    /// Their `condition` is false.
    pub disabled: Vec<String>,
    /// Their `condition` uses `matches` or `notMatches`, which the IDE does
    /// not decide (C1-26); empty once the `CMake` of the project listed them.
    pub unknown: Vec<String>,
    /// Not hidden, and the `CMake` of the project does not list them: each
    /// with why.
    pub dropped: Vec<(String, String)>,
    /// The names in the order the `CMake` of the project lists them; empty
    /// without its answer.
    pub order: Vec<String>,
}

impl Reading {
    pub(super) fn listed(&self, name: &str) -> bool {
        self.unusable(name).is_none()
    }

    /// Why `CMake` will not configure with a preset it read (measured:
    /// "Cannot use hidden configure preset", "Could not evaluate configure
    /// preset", "Cannot use disabled configure preset" and "Could not create
    /// named generator").
    pub(super) fn unusable(&self, name: &str) -> Option<String> {
        if self.hidden.iter().any(|hidden| hidden == name) {
            return Some(format!(
                "o preset {name} é oculto: o CMake não o usa pelo nome"
            ));
        }
        if self.ignored.iter().any(|ignored| ignored == name) {
            return Some(format!(
                "o CMake ignora o preset {name}: ele usa $vendor{{}}"
            ));
        }
        if self.disabled.iter().any(|disabled| disabled == name) {
            return Some(format!(
                "a condition do preset {name} é falsa: o CMake não o usa"
            ));
        }
        self.dropped
            .iter()
            .find(|(dropped, _)| dropped == name)
            .map(|(_, reason)| reason.clone())
    }

    /// What the IDE says of a `condition` it took for true without the
    /// `CMake` (C1-26, `conditionNote`).
    pub(super) fn condition_note(&self, name: &str) -> Option<String> {
        self.unknown.iter().any(|unknown| unknown == name).then(|| {
            "a condition usa matches ou notMatches, que só o CMake decide; sem ele, a IDE a dá \
             por verdadeira"
                .to_owned()
        })
    }
}
