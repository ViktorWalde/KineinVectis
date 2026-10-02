//! Quick places of the IDE-owned folder picker (`0.147.0`).
//!
//! The home, the XDG user directories the person really has (Desktop,
//! Documents, Downloads — read from `user-dirs.dirs`, so a localized
//! "Documentos" is found) and the filesystem root. Only existing directories
//! are returned; a user dir that points at the home itself (how xdg-user-dirs
//! disables one) is skipped, the home is already there.

use std::{
    fs,
    path::{Path, PathBuf},
};

use kinein_protocol::WorkspaceBrowsePlace;

/// XDG user directory keys the picker offers, in display order.
const USER_DIRS: &[(&str, &str)] = &[
    ("XDG_DESKTOP_DIR", "desktop"),
    ("XDG_DOCUMENTS_DIR", "documents"),
    ("XDG_DOWNLOAD_DIR", "downloads"),
];

/// Quick places for the current user, read from `$HOME` and
/// `$XDG_CONFIG_HOME/user-dirs.dirs`.
#[must_use]
pub fn browse_places() -> Vec<WorkspaceBrowsePlace> {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let config = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| home.as_ref().map(|home| home.join(".config")));
    let user_dirs = config
        .and_then(|config| fs::read_to_string(config.join("user-dirs.dirs")).ok())
        .unwrap_or_default();
    places_from(home.as_deref(), &user_dirs)
}

fn places_from(home: Option<&Path>, user_dirs: &str) -> Vec<WorkspaceBrowsePlace> {
    let mut places = Vec::new();
    let home = home.and_then(|home| fs::canonicalize(home).ok());
    if let Some(home) = &home {
        push_place(&mut places, "home", home);
        for (key, id) in USER_DIRS {
            let Some(dir) = user_dir(user_dirs, key, home) else {
                continue;
            };
            if let Ok(dir) = fs::canonicalize(dir) {
                if &dir != home {
                    push_place(&mut places, id, &dir);
                }
            }
        }
    }
    push_place(&mut places, "root", Path::new("/"));
    places
}

fn push_place(places: &mut Vec<WorkspaceBrowsePlace>, id: &str, path: &Path) {
    if path.is_dir() {
        places.push(WorkspaceBrowsePlace {
            id: id.to_owned(),
            path: path.display().to_string(),
        });
    }
}

/// `XDG_DOCUMENTS_DIR="$HOME/Documentos"` → `<home>/Documentos`. Only the
/// two forms xdg-user-dirs writes: `$HOME/...` and an absolute path.
fn user_dir(user_dirs: &str, key: &str, home: &Path) -> Option<PathBuf> {
    let value = user_dirs.lines().find_map(|line| {
        let (name, value) = line.trim().split_once('=')?;
        (name.trim() == key).then(|| value.trim().trim_matches('"').to_owned())
    })?;
    if let Some(rest) = value.strip_prefix("$HOME") {
        return Some(home.join(rest.trim_start_matches('/')));
    }
    value.starts_with('/').then(|| PathBuf::from(value))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::places_from;

    #[test]
    fn places_follow_the_user_dirs_that_exist() {
        let home = std::env::temp_dir()
            .join("kinein-browse-places")
            .join(std::process::id().to_string());
        let _ = fs::remove_dir_all(&home);
        fs::create_dir_all(home.join("Documentos")).unwrap();
        fs::create_dir_all(home.join("Baixados")).unwrap();
        let user_dirs = "# gerado pelo xdg-user-dirs-update\n\
            XDG_DESKTOP_DIR=\"$HOME/\"\n\
            XDG_DOCUMENTS_DIR=\"$HOME/Documentos\"\n\
            XDG_DOWNLOAD_DIR=\"$HOME/Baixados\"\n\
            XDG_MUSIC_DIR=\"$HOME/Musica\"\n";

        let places = places_from(Some(&home), user_dirs);

        let ids = places
            .iter()
            .map(|place| place.id.as_str())
            .collect::<Vec<_>>();
        // A area de trabalho desligada (aponta para a home) nao entra.
        assert_eq!(ids, ["home", "documents", "downloads", "root"]);
        assert!(places[1].path.ends_with("/Documentos"));
        fs::remove_dir_all(&home).unwrap();
    }

    #[test]
    fn without_home_only_the_root_is_offered() {
        let places = places_from(None, "");

        assert_eq!(places.len(), 1);
        assert_eq!(places[0].id, "root");
        assert_eq!(places[0].path, "/");
    }
}
