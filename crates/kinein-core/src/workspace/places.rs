//! Quick places of the IDE-owned folder picker (`0.147.0`; `0.152.0`).
//!
//! Only the home. Until `0.152.0` the picker also offered the XDG user
//! directories (Desktop, Documents, Downloads) and the filesystem root; the
//! author removed them on 2026-10-03: projects live under the home, and the
//! home already contains those folders. The root stays reachable from the
//! path bar (`/`), and is the one place offered when there is no home.

use std::{
    fs,
    path::{Path, PathBuf},
};

use kinein_protocol::WorkspaceBrowsePlace;

/// Quick places for the current user, read from `$HOME`.
#[must_use]
pub fn browse_places() -> Vec<WorkspaceBrowsePlace> {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    places_from(home.as_deref())
}

fn places_from(home: Option<&Path>) -> Vec<WorkspaceBrowsePlace> {
    let home = home
        .and_then(|home| fs::canonicalize(home).ok())
        .filter(|home| home.is_dir());
    let (id, path) = home
        .as_ref()
        .map_or_else(|| ("root", Path::new("/")), |home| ("home", home.as_path()));
    vec![WorkspaceBrowsePlace {
        id: id.to_owned(),
        path: path.display().to_string(),
    }]
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::places_from;

    #[test]
    fn the_only_place_is_the_home() {
        let home = std::env::temp_dir()
            .join("kinein-browse-places")
            .join(std::process::id().to_string());
        let _ = fs::remove_dir_all(&home);
        fs::create_dir_all(home.join("Documentos")).unwrap();

        let places = places_from(Some(&home));

        assert_eq!(places.len(), 1);
        assert_eq!(places[0].id, "home");
        assert_eq!(
            places[0].path,
            fs::canonicalize(&home).unwrap().display().to_string()
        );
        fs::remove_dir_all(&home).unwrap();
    }

    #[test]
    fn without_home_the_root_is_offered() {
        let places = places_from(None);

        assert_eq!(places.len(), 1);
        assert_eq!(places[0].id, "root");
        assert_eq!(places[0].path, "/");
    }
}
