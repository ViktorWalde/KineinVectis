//! Every preset as `CMake` reads it (C1-24).
//!
//! Measured with `CMake` 4.4.5 on 2026-11-09 (consolidacao C §17.4): when it
//! reads the files, `CMake` expands every configure, build, test and package
//! preset, hidden or not, after inheritance and in the context of its file.
//! The order is `environment` (variables by name), `condition` and then the
//! fields of the kind. An invalid macro anywhere refuses every preset;
//! `$vendor{}` stops the preset, and `CMake` ignores it; a false `condition`
//! leaves it out of the list. `matches` and `notMatches` depend on the
//! regular expressions of `CMake` and stay unknown here (C1-26).

use std::{collections::BTreeMap, path::Path};

use serde_json::Value;

use super::{
    files::{Entry, Kind, Presets},
    macros::{Context, Expanded},
    reading::Reading,
};

/// `inherits` deeper than this is a cycle `CMake` refuses elsewhere.
const MAX_DEPTH: usize = 16;

/// How a field holds text.
#[derive(Clone, Copy)]
enum Shape {
    Text,
    /// A text or a list of texts.
    Texts,
    /// Names to texts; an object uses its `value`.
    Map,
}

type Field = (&'static [&'static str], Shape);

/// The fields each kind expands after `environment` and `condition`, in the
/// order `CMake` expands them (measured).
const CONFIGURE: &[Field] = &[
    (&["binaryDir"], Shape::Text),
    (&["installDir"], Shape::Text),
    (&["toolchainFile"], Shape::Text),
    (&["graphviz"], Shape::Text),
    (&["cacheVariables"], Shape::Map),
];
const BUILD: &[Field] = &[
    (&["targets"], Shape::Texts),
    (&["nativeToolOptions"], Shape::Texts),
];
const TEST: &[Field] = &[
    (&["overwriteConfigurationFile"], Shape::Texts),
    (&["output", "outputLogFile"], Shape::Text),
    (&["output", "outputJUnitFile"], Shape::Text),
    (&["filter", "include", "name"], Shape::Text),
    (&["filter", "include", "label"], Shape::Text),
    (&["filter", "include", "index"], Shape::Text),
    (&["filter", "exclude", "name"], Shape::Text),
    (&["filter", "exclude", "label"], Shape::Text),
    (&["filter", "exclude", "fixtures", "any"], Shape::Text),
    (&["filter", "exclude", "fixtures", "setup"], Shape::Text),
    (&["filter", "exclude", "fixtures", "cleanup"], Shape::Text),
    (&["execution", "resourceSpecFile"], Shape::Text),
];
const PACKAGE: &[Field] = &[
    (&["variables"], Shape::Map),
    (&["configFile"], Shape::Text),
    (&["packageName"], Shape::Text),
    (&["packageVersion"], Shape::Text),
    (&["packageDirectory"], Shape::Text),
    (&["vendorName"], Shape::Text),
];

const fn fields(kind: Kind) -> &'static [Field] {
    match kind {
        Kind::Configure => CONFIGURE,
        Kind::Build => BUILD,
        Kind::Test => TEST,
        Kind::Package => PACKAGE,
    }
}

const fn label(kind: Kind) -> &'static str {
    match kind {
        Kind::Configure => "configure",
        Kind::Build => "build",
        Kind::Test => "teste",
        Kind::Package => "pacote",
    }
}

/// Reads every preset of every kind, as `CMake` does.
pub(super) fn read(root: &Path, presets: &Presets) -> Result<Reading, String> {
    let mut reading = Reading::default();
    for kind in Kind::ALL {
        for entry in presets.of(kind) {
            let preset = Preset::new(root, presets, kind, entry);
            let outcome = preset.read().map_err(|reason| {
                format!(
                    "macro inválida no preset de {} \"{}\": {reason}",
                    label(kind),
                    entry.name
                )
            })?;
            if kind != Kind::Configure {
                continue;
            }
            if entry.value.get("hidden").and_then(Value::as_bool) == Some(true) {
                reading.hidden.push(entry.name.clone());
            }
            match outcome {
                None => reading.ignored.push(entry.name.clone()),
                Some(Truth::False) => reading.disabled.push(entry.name.clone()),
                Some(Truth::Unknown) => reading.unknown.push(entry.name.clone()),
                Some(Truth::True) => {}
            }
        }
    }
    Ok(reading)
}

/// The value of a field after `inherits`, and the preset that gives it: the
/// preset first, then its parents in order, depth first. `null` undoes, and
/// wins over the parents after it.
pub(super) fn inherited<'a>(
    presets: &'a Presets,
    kind: Kind,
    entry: &'a Entry,
    path: &[&str],
) -> Option<(&'a Value, &'a Entry)> {
    chain(presets, kind, entry).into_iter().find_map(|link| {
        path.iter()
            .try_fold(&link.value, |value, key| value.get(*key))
            .map(|value| (value, link))
    })
}

/// A map field after `inherits`: each name from the first preset that has it.
fn merged<'a>(
    presets: &'a Presets,
    kind: Kind,
    entry: &'a Entry,
    field: &str,
) -> BTreeMap<String, (&'a Value, &'a Entry)> {
    let mut map = BTreeMap::new();
    for link in chain(presets, kind, entry) {
        for (name, value) in link
            .value
            .get(field)
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
        {
            map.entry(name.clone()).or_insert((value, link));
        }
    }
    map
}

/// The preset and its ancestors, in the order fields are looked up.
fn chain<'a>(presets: &'a Presets, kind: Kind, entry: &'a Entry) -> Vec<&'a Entry> {
    let mut order = Vec::new();
    let mut pending = vec![(entry, 0)];
    while let Some((link, depth)) = pending.pop() {
        order.push(link);
        if depth >= MAX_DEPTH {
            continue;
        }
        let parents: Vec<&str> = match link.value.get("inherits") {
            Some(Value::String(parent)) => vec![parent.as_str()],
            Some(Value::Array(parents)) => parents.iter().filter_map(Value::as_str).collect(),
            _ => Vec::new(),
        };
        let all = presets.of(kind);
        for parent in parents.into_iter().rev() {
            if let Some(found) = all.iter().find(|candidate| candidate.name == parent) {
                pending.push((found, depth + 1));
            }
        }
    }
    order
}

/// A three-valued `condition`: `matches` stays unknown (C1-26).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Truth {
    True,
    False,
    Unknown,
}

impl Truth {
    const fn from(value: bool) -> Self {
        if value { Self::True } else { Self::False }
    }

    const fn not(self) -> Self {
        match self {
            Self::True => Self::False,
            Self::False => Self::True,
            Self::Unknown => Self::Unknown,
        }
    }
}

/// One preset being read.
pub(super) struct Preset<'a> {
    root: &'a Path,
    presets: &'a Presets,
    kind: Kind,
    entry: &'a Entry,
    environment: BTreeMap<String, Option<String>>,
    generator: String,
}

impl<'a> Preset<'a> {
    pub(super) fn new(root: &'a Path, presets: &'a Presets, kind: Kind, entry: &'a Entry) -> Self {
        let text = |found: Option<(&Value, &Entry)>| {
            found
                .and_then(|(value, _)| value.as_str())
                .map(str::to_owned)
        };
        let mut environment = BTreeMap::new();
        let mut generator = None;
        if kind == Kind::Configure {
            generator = text(inherited(presets, kind, entry, &["generator"]));
        } else if let Some(configure) = text(inherited(presets, kind, entry, &["configurePreset"]))
            .and_then(|name| presets.find(&name))
        {
            // A package preset sees no `${generator}`, even with a configure
            // preset that has one (measured); its `environment` still
            // inherits.
            if kind != Kind::Package {
                generator = text(inherited(
                    presets,
                    Kind::Configure,
                    configure,
                    &["generator"],
                ));
            }
            let inherit = inherited(presets, kind, entry, &["inheritConfigureEnvironment"])
                .and_then(|(value, _)| value.as_bool())
                != Some(false);
            if inherit {
                environment = variables(presets, Kind::Configure, configure);
            }
        }
        environment.extend(variables(presets, kind, entry));
        Self {
            root,
            presets,
            kind,
            entry,
            environment,
            generator: generator.unwrap_or_default(),
        }
    }

    fn context(&self) -> Context<'_> {
        let file = self.presets.file_of(self.entry);
        Context {
            root: self.root,
            preset: &self.entry.name,
            generator: &self.generator,
            environment: &self.environment,
            version: file.version,
            file_dir: &file.dir,
        }
    }

    /// Expands the preset: `None` when `CMake` ignores it (`$vendor{}`),
    /// else the truth of its `condition`.
    fn read(&self) -> Result<Option<Truth>, String> {
        let context = self.context();
        for name in self.environment.keys() {
            if context.expand_variable(name)? == Expanded::Vendor {
                return Ok(None);
            }
        }
        let truth = match inherited(self.presets, self.kind, self.entry, &["condition"]) {
            None | Some((Value::Null, _)) => Truth::True,
            Some((condition, _)) => match evaluate(&context, condition)? {
                Some(truth) => truth,
                None => return Ok(None),
            },
        };
        // Whether a text is refused or stops at `$vendor{}` does not depend
        // on the folder `${fileDir}` names, so the version 12 rule of
        // `expand_written` only matters in `field`.
        for (path, shape) in fields(self.kind) {
            for text in self.texts(path, *shape) {
                if context.expand(text)? == Expanded::Vendor {
                    return Ok(None);
                }
            }
        }
        Ok(Some(truth))
    }

    /// The texts of one field after `inherits`.
    fn texts(&self, path: &[&str], shape: Shape) -> Vec<&'a str> {
        if matches!(shape, Shape::Map) {
            return merged(self.presets, self.kind, self.entry, path[0])
                .into_values()
                .filter_map(|(value, _)| {
                    value
                        .as_str()
                        .or_else(|| value.get("value").and_then(Value::as_str))
                })
                .collect();
        }
        match inherited(self.presets, self.kind, self.entry, path) {
            Some((Value::String(text), _)) => vec![text.as_str()],
            Some((Value::Array(items), _)) if matches!(shape, Shape::Texts) => {
                items.iter().filter_map(Value::as_str).collect()
            }
            _ => Vec::new(),
        }
    }

    /// One field expanded in the context of the preset: `Ok(None)` when it is
    /// absent or undone, an error when `CMake` would refuse it or ignore the
    /// preset.
    pub(super) fn field(&self, path: &[&str]) -> Result<Option<String>, String> {
        let shape = if path.len() == 2 && path[0] == "cacheVariables" {
            Shape::Map
        } else {
            Shape::Text
        };
        let found = if matches!(shape, Shape::Map) {
            merged(self.presets, self.kind, self.entry, path[0])
                .remove(path[1])
                .and_then(|(value, link)| {
                    value
                        .as_str()
                        .or_else(|| value.get("value").and_then(Value::as_str))
                        .map(|text| (text, link))
                })
        } else {
            inherited(self.presets, self.kind, self.entry, path)
                .and_then(|(value, link)| value.as_str().map(|text| (text, link)))
        };
        let Some((text, written_in)) = found else {
            return Ok(None);
        };
        let file = self.presets.file_of(written_in);
        match self
            .context()
            .expand_written(text, &file.dir, file.version)?
        {
            Expanded::Text(value) => Ok(Some(value)),
            Expanded::Vendor => Err(format!(
                "o CMake ignora o preset {}: ele usa $vendor{{}}",
                self.entry.name
            )),
        }
    }
}

/// The `environment` of one preset after `inherits`.
fn variables(presets: &Presets, kind: Kind, entry: &Entry) -> BTreeMap<String, Option<String>> {
    merged(presets, kind, entry, "environment")
        .into_iter()
        .filter_map(|(name, (value, _))| match value {
            Value::Null => Some((name, None)),
            Value::String(text) => Some((name, Some(text.clone()))),
            _ => None,
        })
        .collect()
}

/// A `condition`, expanding its texts as `CMake` evaluates it: `anyOf` stops
/// at the first true and `allOf` at the first false (measured), so what comes
/// after is never expanded. `None` when a `$vendor{}` stops the preset.
fn evaluate(context: &Context<'_>, condition: &Value) -> Result<Option<Truth>, String> {
    if let Value::Bool(value) = condition {
        return Ok(Some(Truth::from(*value)));
    }
    let text = |key: &str| -> Result<Option<String>, String> {
        let raw = condition.get(key).and_then(Value::as_str).unwrap_or("");
        Ok(match context.expand(raw)? {
            Expanded::Text(value) => Some(value),
            Expanded::Vendor => None,
        })
    };
    let kind = condition.get("type").and_then(Value::as_str).unwrap_or("");
    Ok(Some(match kind {
        "const" => condition
            .get("value")
            .and_then(Value::as_bool)
            .map_or(Truth::Unknown, Truth::from),
        "equals" | "notEquals" => {
            let Some(lhs) = text("lhs")? else {
                return Ok(None);
            };
            let Some(rhs) = text("rhs")? else {
                return Ok(None);
            };
            let equal = Truth::from(lhs == rhs);
            if kind == "equals" { equal } else { equal.not() }
        }
        "inList" | "notInList" => {
            let Some(item) = text("string")? else {
                return Ok(None);
            };
            let mut found = false;
            for listed in condition
                .get("list")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                match context.expand(listed.as_str().unwrap_or(""))? {
                    Expanded::Text(value) => found |= value == item,
                    Expanded::Vendor => return Ok(None),
                }
            }
            let inside = Truth::from(found);
            if kind == "inList" {
                inside
            } else {
                inside.not()
            }
        }
        "matches" | "notMatches" => {
            if text("string")?.is_none() {
                return Ok(None);
            }
            if text("regex")?.is_none() {
                return Ok(None);
            }
            Truth::Unknown
        }
        "anyOf" | "allOf" => {
            let stop = Truth::from(kind == "anyOf");
            let mut result = stop.not();
            for inner in condition
                .get("conditions")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                match evaluate(context, inner)? {
                    None => return Ok(None),
                    Some(truth) if truth == stop => return Ok(Some(stop)),
                    Some(Truth::Unknown) => result = Truth::Unknown,
                    Some(_) => {}
                }
            }
            result
        }
        "not" => match condition.get("condition") {
            Some(inner) => match evaluate(context, inner)? {
                Some(truth) => truth.not(),
                None => return Ok(None),
            },
            None => Truth::Unknown,
        },
        _ => Truth::Unknown,
    }))
}

#[cfg(test)]
mod tests;
