//! Reading the presets files with `include` (C1-18), as `CMake` 4.4.5 does.
//!
//! Measured on 2026-11-09 (consolidacao C §17): a relative include is relative
//! to the including file; an included file may include others, and a file can
//! be reached twice. A cycle, a missing or invalid file, `include` before
//! version 4, a macro the version does not accept, a repeated preset name and
//! a parent unreachable from the child's file make `CMake` refuse every
//! preset. The IDE refuses with the same reason, never with part of the list.

use std::{
    collections::HashMap,
    fs,
    path::{Component, Path, PathBuf},
};

use serde_json::Value;

/// The two files `CMake` reads at the root.
pub(super) const PROJECT: &str = "CMakePresets.json";
pub(super) const USER: &str = "CMakeUserPresets.json";

/// One presets file that was read.
pub(super) struct File {
    /// The folder `${fileDir}` names and relative includes start from.
    pub dir: PathBuf,
    /// The `version` of the file: it decides which macros its presets use.
    pub version: u64,
    /// The files this one reaches, itself included.
    reaches: Vec<usize>,
    /// Reached from `CMakeUserPresets.json` and not from `CMakePresets.json`.
    pub user_side: bool,
}

/// The kinds of presets whose macros `CMake` expands when it reads the
/// files (C1-24): an invalid macro in any of them refuses every preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    Configure,
    Build,
    Test,
    Package,
}

impl Kind {
    pub(super) const ALL: [Self; 4] = [Self::Configure, Self::Build, Self::Test, Self::Package];

    /// The list of this kind in a presets file.
    const fn field(self) -> &'static str {
        match self {
            Self::Configure => "configurePresets",
            Self::Build => "buildPresets",
            Self::Test => "testPresets",
            Self::Package => "packagePresets",
        }
    }
}

/// One preset and the file that defines it.
pub(super) struct Entry {
    pub name: String,
    pub value: Value,
    pub file: usize,
}

/// Every preset of the project, in list order: the configure presets apart,
/// because the IDE configures with them.
pub(super) struct Presets {
    pub files: Vec<File>,
    pub entries: Vec<Entry>,
    /// Build, test and package presets, by kind.
    pub others: Vec<(Kind, Entry)>,
}

impl Presets {
    /// No presets file at the root.
    const fn empty() -> Self {
        Self {
            files: Vec::new(),
            entries: Vec::new(),
            others: Vec::new(),
        }
    }

    pub(super) fn find(&self, name: &str) -> Option<&Entry> {
        self.entries.iter().find(|entry| entry.name == name)
    }

    /// The presets of one kind, in list order.
    pub(super) fn of(&self, kind: Kind) -> Vec<&Entry> {
        if kind == Kind::Configure {
            return self.entries.iter().collect();
        }
        self.others
            .iter()
            .filter(|(other, _)| *other == kind)
            .map(|(_, entry)| entry)
            .collect()
    }

    pub(super) fn file_of(&self, entry: &Entry) -> &File {
        &self.files[entry.file]
    }
}

/// Reads `CMakePresets.json`, then `CMakeUserPresets.json`, with includes.
pub(super) fn load(root: &Path) -> Result<Presets, String> {
    let mut loader = Loader {
        root,
        presets: Presets::empty(),
        read: HashMap::new(),
        stack: Vec::new(),
    };
    let project = root.join(PROJECT);
    let project_index = if project.is_file() {
        Some(loader.read(&project, PROJECT)?)
    } else {
        None
    };
    let user = root.join(USER);
    if user.is_file() {
        // `CMakeUserPresets.json` includes `CMakePresets.json` implicitly:
        // the project side reaching the user file is a cycle.
        if loader.read.contains_key(&identity(&user)?) {
            return Err(format!(
                "ciclo de include entre arquivos de presets: {USER}"
            ));
        }
        let first_user_file = loader.presets.files.len();
        let user_index = loader.read(&user, USER)?;
        if let Some(project_index) = project_index {
            let reached = loader.presets.files[project_index].reaches.clone();
            loader.presets.files[user_index].reaches.extend(reached);
        }
        for file in &mut loader.presets.files[first_user_file..] {
            file.user_side = true;
        }
    }
    validate(&loader.presets)?;
    Ok(loader.presets)
}

struct Loader<'a> {
    root: &'a Path,
    presets: Presets,
    /// Files already read, by canonical path.
    read: HashMap<PathBuf, usize>,
    /// Files being read: one of them again is a cycle.
    stack: Vec<PathBuf>,
}

impl Loader<'_> {
    fn read(&mut self, path: &Path, label: &str) -> Result<usize, String> {
        if !path.is_file() {
            return Err(format!("arquivo de presets não encontrado: {label}"));
        }
        let identity = identity(path)?;
        if self.stack.contains(&identity) {
            return Err(format!(
                "ciclo de include entre arquivos de presets: {label}"
            ));
        }
        if let Some(&index) = self.read.get(&identity) {
            return Ok(index);
        }
        let body =
            fs::read_to_string(path).map_err(|error| format!("falha lendo {label}: {error}"))?;
        let value: Value =
            serde_json::from_str(&body).map_err(|error| format!("{label} inválido: {error}"))?;
        let version = value
            .get("version")
            .and_then(Value::as_u64)
            .ok_or_else(|| format!("{label} sem o campo version"))?;
        let includes = includes(&value, label, version)?;
        let dir = path.parent().unwrap_or(self.root).to_path_buf();
        let index = self.presets.files.len();
        self.presets.files.push(File {
            dir: dir.clone(),
            version,
            reaches: vec![index],
            user_side: false,
        });
        self.read.insert(identity.clone(), index);
        for kind in Kind::ALL {
            for item in value
                .get(kind.field())
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                let Some(name) = item.get("name").and_then(Value::as_str) else {
                    continue;
                };
                let entry = Entry {
                    name: name.to_owned(),
                    value: item.clone(),
                    file: index,
                };
                if kind == Kind::Configure {
                    self.presets.entries.push(entry);
                } else {
                    self.presets.others.push((kind, entry));
                }
            }
        }
        self.stack.push(identity);
        for include in includes {
            let expanded = super::macros::include_path(include, version, self.root, &dir)
                .map_err(|reason| format!("{label}: include inválido: {reason}"))?;
            let target = collapse(&dir.join(expanded));
            let child = self.read(&target, &super::label(self.root, &target))?;
            let reached = self.presets.files[child].reaches.clone();
            self.presets.files[index].reaches.extend(reached);
        }
        self.stack.pop();
        Ok(index)
    }
}

/// The `include` paths; the field exists from version 4 on.
fn includes<'a>(value: &'a Value, label: &str, version: u64) -> Result<Vec<&'a str>, String> {
    let Some(field) = value.get("include") else {
        return Ok(Vec::new());
    };
    if version < 4 {
        return Err(format!("{label}: include exige version 4 ou maior"));
    }
    field
        .as_array()
        .ok_or_else(|| format!("{label}: include não é uma lista"))?
        .iter()
        .map(|item| {
            item.as_str()
                .ok_or_else(|| format!("{label}: item de include não é texto"))
        })
        .collect()
}

/// A repeated name, a parent that exists nowhere or in a file the child's
/// file does not reach, and cyclic inheritance, as `CMake` 4.4.5 refuses.
fn validate(presets: &Presets) -> Result<(), String> {
    for (position, entry) in presets.entries.iter().enumerate() {
        if presets.entries[..position]
            .iter()
            .any(|earlier| earlier.name == entry.name)
        {
            return Err(format!("preset de configure repetido: \"{}\"", entry.name));
        }
    }
    for entry in &presets.entries {
        let reaches = &presets.file_of(entry).reaches;
        for parent in parents(entry) {
            let Some(found) = presets.find(parent) else {
                return Err(format!(
                    "o preset de configure \"{}\" herda de \"{parent}\", que não existe",
                    entry.name
                ));
            };
            if !reaches.contains(&found.file) {
                return Err(format!(
                    "o preset de configure \"{}\" herda de um preset que o arquivo dele não inclui",
                    entry.name
                ));
            }
        }
        if inherits_itself(presets, entry, &mut Vec::new()) {
            return Err(format!(
                "herança cíclica no preset de configure \"{}\"",
                entry.name
            ));
        }
    }
    Ok(())
}

fn parents(entry: &Entry) -> Vec<&str> {
    match entry.value.get("inherits") {
        Some(Value::String(parent)) => vec![parent.as_str()],
        Some(Value::Array(parents)) => parents.iter().filter_map(Value::as_str).collect(),
        _ => Vec::new(),
    }
}

/// Whether following `inherits` from `entry` comes back to a preset on the
/// current path.
fn inherits_itself<'a>(presets: &'a Presets, entry: &'a Entry, path: &mut Vec<&'a str>) -> bool {
    if path.contains(&entry.name.as_str()) {
        return true;
    }
    path.push(&entry.name);
    let cyclic = parents(entry)
        .into_iter()
        .filter_map(|parent| presets.find(parent))
        .any(|parent| inherits_itself(presets, parent, path));
    path.pop();
    cyclic
}

/// The same file reached by two paths is one file.
fn identity(path: &Path) -> Result<PathBuf, String> {
    crate::platform::canonicalize(path)
        .map_err(|error| format!("falha lendo {}: {error}", path.display()))
}

/// `a/./b/../c` is `a/c`, as `CMake` collapses an include path.
fn collapse(path: &Path) -> PathBuf {
    let mut collapsed = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !collapsed.pop() {
                    collapsed.push(component);
                }
            }
            other => collapsed.push(other),
        }
    }
    collapsed
}
