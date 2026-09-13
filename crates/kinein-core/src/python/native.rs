//! O MODULO NATIVO de um projeto Python (fatia 5 da cadeia do `roadmaps/41`
//! bloco B, 2026-09-13): C++ por pybind11/nanobind, Rust por `PyO3` — e a
//! ferramenta que o compila e instala no ambiente.
//!
//! E' a ponte entre as duas metades da IDE: um projeto Python com extensao em
//! C++/Rust e' TAMBEM um projeto CMake/Cargo (o indice e o clangd/rust-analyzer
//! ja' o leem), mas quem o instala no `.venv` e' o build do Python — `maturin
//! develop`, `pip install -e .`. Sem saber isso a tela oferece o build errado.
//!
//! Por evidencia, sem adivinhacao, e so' nos arquivos da raiz:
//!
//! ```text
//! pyproject.toml   [build-system] requires: maturin | scikit-build-core |
//!                  setuptools-rust | pybind11 | nanobind; [tool.maturin]
//! Cargo.toml       pyo3 nas dependencias
//! CMakeLists.txt   find_package(pybind11|nanobind), pybind11_add_module,
//!                  nanobind_add_module
//! setup.py         Pybind11Extension / pybind11 / nanobind
//! ```
//!
//! Linhas de comando lidas na fonte em 2026-09-13: `maturin develop` (docs do
//! maturin: compila e instala no venv ativo); `pip install -e .` para
//! scikit-build-core/setuptools (o nanobind recomenda `--no-build-isolation`
//! para iterar, mas o primeiro build e' o padrao).

use std::path::Path;

use kinein_protocol::PythonNativeModule;

/// O que se le de cada arquivo, uma vez.
struct Arquivos {
    pyproject: String,
    cargo: String,
    cmake: String,
    setup_py: String,
}

impl Arquivos {
    fn ler(root: &Path) -> Self {
        let ler = |nome: &str| std::fs::read_to_string(root.join(nome)).unwrap_or_default();
        Self {
            pyproject: ler("pyproject.toml"),
            cargo: ler("Cargo.toml"),
            cmake: ler("CMakeLists.txt"),
            setup_py: ler("setup.py"),
        }
    }
}

/// O modulo nativo do projeto, se as evidencias apontam um.
#[must_use]
pub fn detect(root: &Path) -> Option<PythonNativeModule> {
    let a = Arquivos::ler(root);
    let mut evidence = Vec::new();
    let mut kind: Option<&str> = None;
    let mut tool: Option<&str> = None;

    // A ferramenta de build, pelo pyproject.
    if let Some(requires) = build_system_requires(&a.pyproject) {
        for (marca, ferramenta) in [
            ("maturin", "maturin"),
            ("scikit-build-core", "scikit-build-core"),
            ("setuptools-rust", "setuptools-rust"),
        ] {
            if requires.contains(marca) {
                tool = Some(ferramenta);
                evidence.push(format!("pyproject.toml: [build-system] requires {marca}"));
                break;
            }
        }
        if requires.contains("pybind11") {
            kind = Some("pybind11");
            evidence.push("pyproject.toml: [build-system] requires pybind11".to_owned());
        } else if requires.contains("nanobind") {
            kind = Some("nanobind");
            evidence.push("pyproject.toml: [build-system] requires nanobind".to_owned());
        }
    }
    if tool.is_none() && a.pyproject.contains("[tool.maturin") {
        tool = Some("maturin");
        evidence.push("pyproject.toml: [tool.maturin]".to_owned());
    }

    // O tipo do modulo, pelas fontes.
    if kind.is_none() && depende_de_pyo3(&a.cargo) {
        kind = Some("PyO3");
        evidence.push("Cargo.toml: dependencia pyo3".to_owned());
    }
    if kind.is_none() {
        if a.cmake.contains("pybind11") {
            kind = Some("pybind11");
            evidence.push("CMakeLists.txt: pybind11".to_owned());
        } else if a.cmake.contains("nanobind") {
            kind = Some("nanobind");
            evidence.push("CMakeLists.txt: nanobind".to_owned());
        }
    }
    if kind.is_none() {
        if a.setup_py.contains("pybind11") {
            kind = Some("pybind11");
            evidence.push("setup.py: pybind11".to_owned());
        } else if a.setup_py.contains("nanobind") {
            kind = Some("nanobind");
            evidence.push("setup.py: nanobind".to_owned());
        }
    }

    // maturin/setuptools-rust sem pyo3 explicito: e' Rust de qualquer forma.
    let kind = match (kind, tool) {
        (Some(k), _) => k,
        (None, Some("maturin" | "setuptools-rust")) => "Rust",
        (None, _) => return None,
    };
    // Sem ferramenta declarada: PyO3 se instala com maturin; C++ com o
    // setuptools do proprio pip. ("Rust" so' existe COM ferramenta.)
    let tool = tool.unwrap_or(if kind == "PyO3" {
        "maturin"
    } else {
        "setuptools"
    });
    let build_hint = match tool {
        "maturin" => {
            "maturin develop (no ambiente do projeto: `uv run maturin develop`, ou \
                      `.venv/bin/maturin develop`)"
        }
        "scikit-build-core" => {
            "pip install -e . (o scikit-build-core chama o CMake; `uv pip \
                                install -e .` num projeto do uv)"
        }
        _ => "pip install -e . (compila a extensao com o compilador do sistema)",
    };
    Some(PythonNativeModule {
        kind: kind.to_owned(),
        tool: tool.to_owned(),
        evidence,
        build_hint: build_hint.to_owned(),
    })
}

/// O texto de `[build-system]` a partir de `requires`, achatado numa linha
/// (o `requires` pode ocupar varias linhas; o `build-backend` que vem depois
/// entra junto e nao atrapalha — os nomes procurados sao os das ferramentas).
fn build_system_requires(pyproject: &str) -> Option<String> {
    let mut dentro = false;
    let mut requires = String::new();
    let mut lendo = false;
    for linha in pyproject.lines() {
        let l = linha.trim();
        if l.starts_with('[') {
            dentro = l == "[build-system]";
            continue;
        }
        if !dentro {
            continue;
        }
        if l.starts_with("requires") {
            lendo = true;
        }
        if lendo {
            requires.push_str(l);
            requires.push(' ');
        }
    }
    lendo.then_some(requires)
}

/// `pyo3` como dependencia do Cargo.toml (`pyo3 = ...` ou `[dependencies.pyo3]`).
fn depende_de_pyo3(cargo: &str) -> bool {
    cargo.lines().any(|l| {
        let l = l.trim();
        l.starts_with("pyo3 ") || l.starts_with("pyo3=") || l == "[dependencies.pyo3]"
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::detect;

    fn raiz(nome: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("kinein-core-tests")
            .join(format!("{}-python-native-{nome}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Um projeto Python puro nao tem modulo nativo — nem com um Cargo.toml
    /// sem pyo3 ou um `CMakeLists` sem binding ao lado.
    #[test]
    fn pure_python_has_no_native_module() {
        let root = raiz("puro");
        std::fs::write(
            root.join("pyproject.toml"),
            "[project]\nname = \"x\"\n[build-system]\nrequires = [\"hatchling\"]\n",
        )
        .unwrap();
        std::fs::write(root.join("Cargo.toml"), "[dependencies]\nserde = \"1\"\n").unwrap();
        std::fs::write(root.join("CMakeLists.txt"), "project(x CXX)\n").unwrap();
        assert_eq!(detect(&root), None);
    }

    /// maturin + pyo3: o caso `PyO3` completo, com as duas evidencias.
    #[test]
    fn maturin_with_pyo3_is_a_rust_module_built_by_maturin() {
        let root = raiz("maturin");
        std::fs::write(
            root.join("pyproject.toml"),
            "[build-system]\nrequires = [\"maturin>=1.7,<2.0\"]\nbuild-backend = \"maturin\"\n\n[project]\nname = \"demo\"\n",
        )
        .unwrap();
        std::fs::write(root.join("Cargo.toml"), "[package]\nname = \"demo\"\n\n[dependencies]\npyo3 = { version = \"0.26\", features = [\"extension-module\"] }\n").unwrap();
        let m = detect(&root).unwrap();
        assert_eq!((m.kind.as_str(), m.tool.as_str()), ("PyO3", "maturin"));
        assert_eq!(
            m.evidence,
            vec![
                "pyproject.toml: [build-system] requires maturin",
                "Cargo.toml: dependencia pyo3"
            ]
        );
        assert!(
            m.build_hint.starts_with("maturin develop"),
            "{}",
            m.build_hint
        );

        // requires em varias linhas tambem conta.
        std::fs::write(
            root.join("pyproject.toml"),
            "[build-system]\nrequires = [\n  \"maturin>=1.7\",\n]\n",
        )
        .unwrap();
        assert_eq!(detect(&root).unwrap().tool, "maturin");
        // So' [tool.maturin], sem pyo3 no Cargo: Rust por maturin.
        std::fs::write(
            root.join("pyproject.toml"),
            "[project]\nname = \"d\"\n[tool.maturin]\nfeatures = []\n",
        )
        .unwrap();
        std::fs::write(root.join("Cargo.toml"), "[package]\nname = \"demo\"\n").unwrap();
        let m = detect(&root).unwrap();
        assert_eq!((m.kind.as_str(), m.tool.as_str()), ("Rust", "maturin"));
        assert_eq!(m.evidence, vec!["pyproject.toml: [tool.maturin]"]);
    }

    /// pybind11 e nanobind: pelo `CMake` (com scikit-build-core ou nao), pelo
    /// setup.py, e pelo proprio requires do pyproject.
    #[test]
    fn cpp_bindings_are_recognised_from_cmake_setup_py_and_pyproject() {
        let root = raiz("pybind");
        std::fs::write(
            root.join("CMakeLists.txt"),
            "find_package(pybind11 CONFIG REQUIRED)\npybind11_add_module(demo src/demo.cpp)\n",
        )
        .unwrap();
        let m = detect(&root).unwrap();
        assert_eq!(
            (m.kind.as_str(), m.tool.as_str()),
            ("pybind11", "setuptools")
        );
        assert!(
            m.build_hint.starts_with("pip install -e ."),
            "{}",
            m.build_hint
        );

        std::fs::write(
            root.join("pyproject.toml"),
            "[build-system]\nrequires = [\"scikit-build-core\", \"nanobind\"]\n",
        )
        .unwrap();
        std::fs::write(
            root.join("CMakeLists.txt"),
            "find_package(nanobind CONFIG REQUIRED)\nnanobind_add_module(demo src/demo.cpp)\n",
        )
        .unwrap();
        let m = detect(&root).unwrap();
        assert_eq!(
            (m.kind.as_str(), m.tool.as_str()),
            ("nanobind", "scikit-build-core")
        );
        assert!(
            m.build_hint.contains("scikit-build-core"),
            "{}",
            m.build_hint
        );
        assert_eq!(m.evidence.len(), 2, "{:?}", m.evidence);

        let root = raiz("setup-py");
        std::fs::write(
            root.join("setup.py"),
            "from pybind11.setup_helpers import Pybind11Extension\n",
        )
        .unwrap();
        let m = detect(&root).unwrap();
        assert_eq!(m.kind, "pybind11");
        assert_eq!(m.evidence, vec!["setup.py: pybind11"]);
    }

    /// setuptools-rust: Rust sem maturin, instalado por pip.
    #[test]
    fn setuptools_rust_is_rust_installed_by_pip() {
        let root = raiz("setuptools-rust");
        std::fs::write(
            root.join("pyproject.toml"),
            "[build-system]\nrequires = [\"setuptools\", \"setuptools-rust\"]\n",
        )
        .unwrap();
        let m = detect(&root).unwrap();
        assert_eq!(
            (m.kind.as_str(), m.tool.as_str()),
            ("Rust", "setuptools-rust")
        );
        assert!(
            m.build_hint.starts_with("pip install -e ."),
            "{}",
            m.build_hint
        );
    }
}
