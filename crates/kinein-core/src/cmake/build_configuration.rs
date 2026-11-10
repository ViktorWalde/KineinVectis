//! C1-22: the configuration a multi-config build folder builds, tests and
//! runs (consolidação C §20).
//!
//! Measured with `CMake` 4.4.5 and `Ninja Multi-Config`: the compilation
//! database has one entry per configuration for each file, in the order of
//! `CMAKE_CONFIGURATION_TYPES`, and clangd reads the first; the codemodel has
//! the configurations in the same order. `cmake --build` without `--config`
//! builds `CMAKE_DEFAULT_BUILD_TYPE`, and `ctest` without `-C` runs nothing
//! ("Not Run"). The IDE names the first one to both.

use std::fs;
use std::path::Path;

/// The configuration of a folder a multi-config generator configured.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildConfiguration {
    /// The first item of `CMAKE_CONFIGURATION_TYPES`: the one clangd reads.
    pub name: String,
    /// `CMAKE_DEFAULT_BUILD_TYPE`, when it is another one: what
    /// `cmake --build` without `--config` builds.
    pub default: Option<String>,
}

/// Reads the cache of `directory`. `None` without cache, with a
/// single-config generator, which ignores `CMAKE_CONFIGURATION_TYPES`
/// (measured with `Ninja`), or with an empty list.
#[must_use]
pub fn of(directory: &Path) -> Option<BuildConfiguration> {
    let cache = fs::read_to_string(directory.join("CMakeCache.txt")).ok()?;
    let value = |key: &str| {
        cache.lines().find_map(|line| {
            let (declaration, value) = line.split_once('=')?;
            // `CMake` drops the blanks at the end of a cache value and keeps
            // those at the start: ` Release` names the folder ` Release`.
            (declaration.split(':').next() == Some(key)).then(|| value.trim_end())
        })
    };
    let generator = value("CMAKE_GENERATOR")?;
    if !(generator == "Ninja Multi-Config"
        || generator == "Xcode"
        || generator.starts_with("Visual Studio "))
    {
        return None;
    }
    // `CMake` drops the empty items: `;Release;Debug` builds Release first.
    let name = value("CMAKE_CONFIGURATION_TYPES")?
        .split(';')
        .find(|item| !item.is_empty())?
        .to_owned();
    let default = value("CMAKE_DEFAULT_BUILD_TYPE")
        .filter(|default| !default.is_empty() && *default != name)
        .map(str::to_owned);
    Some(BuildConfiguration { name, default })
}

#[cfg(test)]
mod tests {
    use super::{BuildConfiguration, of};
    use std::fs;
    use std::path::PathBuf;

    fn folder(name: &str, cache: Option<&str>) -> PathBuf {
        let directory = std::env::temp_dir()
            .join("kinein-build-configuration-tests")
            .join(format!("{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).unwrap();
        if let Some(cache) = cache {
            fs::write(directory.join("CMakeCache.txt"), cache).unwrap();
        }
        directory
    }

    fn configuration(name: &str, default: Option<&str>) -> BuildConfiguration {
        BuildConfiguration {
            name: name.to_owned(),
            default: default.map(str::to_owned),
        }
    }

    /// Each multi-config generator, the list as `CMake` reads it and the
    /// default only when it is another one; a single-config generator with
    /// the list in the cache (measured: `Ninja` ignores it), a folder without
    /// cache and an empty list name none.
    #[test]
    fn the_first_configuration_of_a_cache_with_several() {
        let cases = [
            (
                "ninja-multi",
                "# This is the CMakeCache file.\n//Configurations\n\
                 CMAKE_CONFIGURATION_TYPES:STRING=Release;Debug\n\
                 CMAKE_GENERATOR:INTERNAL=Ninja Multi-Config\n",
                Some(configuration("Release", None)),
            ),
            (
                "default-other",
                "CMAKE_GENERATOR:INTERNAL=Ninja Multi-Config\n\
                 CMAKE_CONFIGURATION_TYPES:STRING=Release;Debug\n\
                 CMAKE_DEFAULT_BUILD_TYPE:STRING=Debug\n",
                Some(configuration("Release", Some("Debug"))),
            ),
            (
                "default-same",
                "CMAKE_GENERATOR:INTERNAL=Ninja Multi-Config\n\
                 CMAKE_CONFIGURATION_TYPES:STRING=Release;Debug\n\
                 CMAKE_DEFAULT_BUILD_TYPE:STRING=Release\n",
                Some(configuration("Release", None)),
            ),
            (
                "default-empty",
                "CMAKE_GENERATOR:INTERNAL=Ninja Multi-Config\n\
                 CMAKE_CONFIGURATION_TYPES:STRING=Debug\n\
                 CMAKE_DEFAULT_BUILD_TYPE:STRING=\n",
                Some(configuration("Debug", None)),
            ),
            (
                "empty-item",
                "CMAKE_GENERATOR:INTERNAL=Ninja Multi-Config\n\
                 CMAKE_CONFIGURATION_TYPES:UNINITIALIZED=;Fast;Debug\n",
                Some(configuration("Fast", None)),
            ),
            (
                "visual-studio",
                "CMAKE_GENERATOR:INTERNAL=Visual Studio 17 2022\n\
                 CMAKE_GENERATOR_PLATFORM:INTERNAL=x64\n\
                 CMAKE_CONFIGURATION_TYPES:STRING=Debug;Release;MinSizeRel;RelWithDebInfo\n",
                Some(configuration("Debug", None)),
            ),
            (
                "xcode",
                "CMAKE_GENERATOR:INTERNAL=Xcode\nCMAKE_CONFIGURATION_TYPES:STRING=Debug;Release\n",
                Some(configuration("Debug", None)),
            ),
            (
                "single-ninja",
                "CMAKE_GENERATOR:INTERNAL=Ninja\n\
                 CMAKE_CONFIGURATION_TYPES:STRING=Release;Debug\n\
                 CMAKE_BUILD_TYPE:STRING=Debug\n",
                None,
            ),
            (
                "instance-only",
                "CMAKE_GENERATOR_INSTANCE:INTERNAL=Ninja Multi-Config\n\
                 CMAKE_CONFIGURATION_TYPES:STRING=Debug\n",
                None,
            ),
            (
                "empty-list",
                "CMAKE_GENERATOR:INTERNAL=Ninja Multi-Config\nCMAKE_CONFIGURATION_TYPES:STRING=\n",
                None,
            ),
            (
                "no-list",
                "CMAKE_GENERATOR:INTERNAL=Ninja Multi-Config\n",
                None,
            ),
            // Measured with `CMake` 4.4.5: `-DCMAKE_CONFIGURATION_TYPES=" Release;Debug "`
            // writes ` Release;Debug`, and the codemodel names ` Release`.
            (
                "blanks",
                "CMAKE_GENERATOR:INTERNAL=Ninja Multi-Config \t\n\
                 CMAKE_CONFIGURATION_TYPES:UNINITIALIZED= Release;Debug\n\
                 CMAKE_DEFAULT_BUILD_TYPE:UNINITIALIZED=Debug \r\n",
                Some(configuration(" Release", Some("Debug"))),
            ),
        ];
        for (name, cache, expected) in cases {
            assert_eq!(of(&folder(name, Some(cache))), expected, "{name}");
        }
        assert_eq!(of(&folder("no-cache", None)), None);
    }
}
