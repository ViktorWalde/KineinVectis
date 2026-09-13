//! Creating folders and scaffolding new projects from templates.

use std::{
    fs, io,
    path::{Path, PathBuf},
    process::Command,
};

use kinein_protocol::{WorkspaceInfo, WorkspaceProjectTemplate};

use super::WorkspaceError;
use super::open_workspace;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum NameKind {
    Folder,
    Project,
}

/// Creates a child directory under an existing parent for the workspace picker.
pub fn create_directory(parent: &Path, name: &str) -> Result<PathBuf, WorkspaceError> {
    let parent = canonical_directory(parent)?;
    let name = validate_child_name(name, NameKind::Folder)?;
    let target = parent.join(name);
    if target.exists() {
        return Err(WorkspaceError::AlreadyExists {
            path: target.display().to_string(),
        });
    }
    fs::create_dir(&target).map_err(|source| WorkspaceError::CreateDirectory {
        path: target.display().to_string(),
        source,
    })?;
    fs::canonicalize(&target).map_err(|source| WorkspaceError::InvalidRoot {
        path: target.display().to_string(),
        source,
    })
}

/// Creates a new project directory and opens it as a workspace.
pub fn create_project(
    parent: &Path,
    name: &str,
    template: WorkspaceProjectTemplate,
) -> Result<WorkspaceInfo, WorkspaceError> {
    match template {
        WorkspaceProjectTemplate::Empty => {
            let root = create_directory(parent, name)?;
            open_workspace(&root)
        }
        WorkspaceProjectTemplate::CppCmake => create_cpp_cmake_project(parent, name),
        WorkspaceProjectTemplate::RustCargo => create_rust_cargo_project(parent, name),
        WorkspaceProjectTemplate::Python => create_python_project(parent, name),
    }
}

/// Um projeto Python como a PEP 621 escreve, sem rodar ferramenta nenhuma:
/// `pyproject.toml` (nome, versao, `requires-python`, `[project.optional-
/// dependencies] dev = ["pytest"]`, `[tool.ruff]`, `[tool.pytest.ini_options]`
/// com `pythonpath = ["."]`), `main.py` como ponto de entrada (o que o botao
/// Executar procura primeiro), o pacote `<pacote>/__init__.py` NA RAIZ (layout
/// plano: `python main.py` e `python -m pytest` acham o pacote sem instalar
/// nada — um `src/` exigiria `pip install -e .` antes do primeiro Executar),
/// `tests/test_main.py`, `.gitignore` com o `.venv`. Criar o ambiente e' o
/// clique da faixa de saude; instalar o pytest e' `uv add --dev pytest` ou
/// `pip install -e .[dev]` — a IDE nao roda nada aqui.
fn create_python_project(parent: &Path, name: &str) -> Result<WorkspaceInfo, WorkspaceError> {
    let parent = canonical_directory(parent)?;
    let name = validate_child_name(name, NameKind::Project)?;
    let root = parent.join(name);
    if root.exists() {
        return Err(WorkspaceError::AlreadyExists {
            path: root.display().to_string(),
        });
    }
    // O nome do pacote e' o do projeto em forma de modulo: `-` vira `_`.
    let pacote = name.replace('-', "_");
    let src = root.join(&pacote);
    let tests = root.join("tests");
    for directory in [&root, &src, &tests] {
        fs::create_dir(directory).map_err(|source| WorkspaceError::CreateDirectory {
            path: directory.display().to_string(),
            source,
        })?;
    }
    let pyproject = [
        "[project]".to_owned(),
        format!("name = \"{name}\""),
        "version = \"0.1.0\"".to_owned(),
        "description = \"\"".to_owned(),
        "requires-python = \">=3.12\"".to_owned(),
        "dependencies = []".to_owned(),
        String::new(),
        "[project.optional-dependencies]".to_owned(),
        "dev = [\"pytest\"]".to_owned(),
        String::new(),
        "[tool.ruff]".to_owned(),
        "line-length = 100".to_owned(),
        String::new(),
        "[tool.pytest.ini_options]".to_owned(),
        "testpaths = [\"tests\"]".to_owned(),
        "pythonpath = [\".\"]".to_owned(),
    ]
    .join("\n")
        + "\n";
    write_template(&root.join("pyproject.toml"), &pyproject)?;
    let main = [
        format!("\"\"\"Ponto de entrada de {name}: e' o que o botao Executar roda.\"\"\""),
        String::new(),
        format!("from {pacote} import saudacao"),
        String::new(),
        String::new(),
        "def main() -> None:".to_owned(),
        "    print(saudacao(\"mundo\"))".to_owned(),
        String::new(),
        String::new(),
        "if __name__ == \"__main__\":".to_owned(),
        "    main()".to_owned(),
    ]
    .join("\n")
        + "\n";
    write_template(&root.join("main.py"), &main)?;
    let init = [
        format!("\"\"\"{pacote}: o pacote de {name}.\"\"\""),
        String::new(),
        String::new(),
        "def saudacao(nome: str) -> str:".to_owned(),
        "    return f\"Ola, {nome}!\"".to_owned(),
    ]
    .join("\n")
        + "\n";
    write_template(&src.join("__init__.py"), &init)?;
    let test_main = [
        format!("from {pacote} import saudacao"),
        String::new(),
        String::new(),
        "def test_saudacao() -> None:".to_owned(),
        "    assert saudacao(\"mundo\") == \"Ola, mundo!\"".to_owned(),
    ]
    .join("\n")
        + "\n";
    write_template(&tests.join("test_main.py"), &test_main)?;
    write_template(
        &root.join(".gitignore"),
        ".venv/\n__pycache__/\n*.pyc\n.pytest_cache/\n.ruff_cache/\ndist/\n",
    )?;
    let readme = [
        format!("# {name}"),
        String::new(),
        "Projeto Python criado pelo Kinein Vectis.".to_owned(),
        String::new(),
        "- Ambiente: o botao \"Criar .venv\" da IDE (`uv venv .venv` ou `python3 -m venv .venv`)."
            .to_owned(),
        "- Testes: `uv add --dev pytest` (ou `.venv/bin/pip install -e .[dev]`) e o botao Testes."
            .to_owned(),
        "- Executar: o botao Executar roda `main.py` com o interpretador do projeto.".to_owned(),
    ]
    .join("\n")
        + "\n";
    write_template(&root.join("README.md"), &readme)?;
    open_workspace(&root)
}

fn canonical_directory(path: &Path) -> Result<PathBuf, WorkspaceError> {
    let directory = fs::canonicalize(path).map_err(|source| WorkspaceError::InvalidRoot {
        path: path.display().to_string(),
        source,
    })?;

    if !directory.is_dir() {
        return Err(WorkspaceError::NotADirectory {
            path: directory.display().to_string(),
        });
    }

    Ok(directory)
}

fn validate_child_name(name: &str, kind: NameKind) -> Result<&str, WorkspaceError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(WorkspaceError::InvalidName {
            name: name.to_owned(),
            reason: "nome vazio".to_owned(),
        });
    }
    if trimmed == "." || trimmed == ".." || trimmed.contains('/') || trimmed.contains('\\') {
        return Err(WorkspaceError::InvalidName {
            name: name.to_owned(),
            reason: "use apenas o nome, nao um caminho".to_owned(),
        });
    }
    if kind == NameKind::Project {
        let starts_ok = trimmed
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphabetic);
        let chars_ok = trimmed
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'));
        if !starts_ok || !chars_ok {
            return Err(WorkspaceError::InvalidName {
                name: name.to_owned(),
                reason: "projetos usam letras, numeros, '-' ou '_' e comecam por letra".to_owned(),
            });
        }
    }
    Ok(trimmed)
}

fn create_cpp_cmake_project(parent: &Path, name: &str) -> Result<WorkspaceInfo, WorkspaceError> {
    let parent = canonical_directory(parent)?;
    let name = validate_child_name(name, NameKind::Project)?;
    let root = parent.join(name);
    if root.exists() {
        return Err(WorkspaceError::AlreadyExists {
            path: root.display().to_string(),
        });
    }
    fs::create_dir(&root).map_err(|source| WorkspaceError::CreateDirectory {
        path: root.display().to_string(),
        source,
    })?;
    let src = root.join("src");
    let include = root.join("include");
    let tests = root.join("tests");
    for directory in [&src, &include, &tests] {
        fs::create_dir(directory).map_err(|source| WorkspaceError::CreateDirectory {
            path: directory.display().to_string(),
            source,
        })?;
    }

    write_template(
        &root.join("CMakeLists.txt"),
        &format!(
            "cmake_minimum_required(VERSION 3.24)\n\n\
             project({name}\n\
                 VERSION 0.1.0\n\
                 LANGUAGES CXX\n\
             )\n\n\
             add_executable({name}\n\
                 src/main.cpp\n\
             )\n\n\
             target_compile_features({name}\n\
                 PRIVATE\n\
                     cxx_std_23\n\
             )\n\n\
             target_include_directories({name}\n\
                 PRIVATE\n\
                     ${{CMAKE_CURRENT_SOURCE_DIR}}/include\n\
             )\n\n\
             target_compile_options({name} PRIVATE\n\
                 -Wall -Wextra -Wpedantic -Werror\n\
                 -Wconversion -Wsign-conversion -Wshadow\n\
             )\n"
        ),
    )?;
    write_template(
        &root.join("CMakePresets.json"),
        "{\n  \"version\": 6,\n  \"configurePresets\": [\n    {\n      \"name\": \"debug\",\n      \"displayName\": \"Debug\",\n      \"generator\": \"Ninja\",\n      \"binaryDir\": \"${sourceDir}/build/debug\",\n      \"cacheVariables\": {\n        \"CMAKE_BUILD_TYPE\": \"Debug\",\n        \"CMAKE_EXPORT_COMPILE_COMMANDS\": \"ON\"\n      }\n    },\n    {\n      \"name\": \"release\",\n      \"displayName\": \"Release\",\n      \"generator\": \"Ninja\",\n      \"binaryDir\": \"${sourceDir}/build/release\",\n      \"cacheVariables\": {\n        \"CMAKE_BUILD_TYPE\": \"Release\",\n        \"CMAKE_EXPORT_COMPILE_COMMANDS\": \"ON\"\n      }\n    }\n  ],\n  \"buildPresets\": [\n    {\n      \"name\": \"debug\",\n      \"configurePreset\": \"debug\"\n    },\n    {\n      \"name\": \"release\",\n      \"configurePreset\": \"release\"\n    }\n  ]\n}\n",
    )?;
    write_template(
        &src.join("main.cpp"),
        "#include <iostream>\n\nint main()\n{\n    std::cout << \"Kinein Vectis\" << '\\n';\n    return 0;\n}\n",
    )?;
    write_template(
        &root.join("README.md"),
        &format!("# {name}\n\nProjeto C++/CMake strict criado pelo Kinein Vectis.\n"),
    )?;
    write_template(
        &root.join(".gitignore"),
        "/build/\n/.cache/\n/compile_commands.json\n",
    )?;

    open_workspace(&root)
}

fn create_rust_cargo_project(parent: &Path, name: &str) -> Result<WorkspaceInfo, WorkspaceError> {
    let parent = canonical_directory(parent)?;
    let name = validate_child_name(name, NameKind::Project)?;
    let root = parent.join(name);
    if root.exists() {
        return Err(WorkspaceError::AlreadyExists {
            path: root.display().to_string(),
        });
    }

    let output = Command::new("cargo")
        .args(["new", "--bin", "--vcs", "none", name])
        .current_dir(&parent)
        .output()
        .map_err(|source| {
            if source.kind() == io::ErrorKind::NotFound {
                WorkspaceError::MissingTool { tool: "cargo" }
            } else {
                WorkspaceError::CreateDirectory {
                    path: root.display().to_string(),
                    source,
                }
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(WorkspaceError::ToolFailed {
            command: "cargo new --bin --vcs none".to_owned(),
            exit_code: output.status.code(),
            message: format!("{stderr}{stdout}"),
        });
    }

    open_workspace(&root)
}

fn write_template(path: &Path, content: &str) -> Result<(), WorkspaceError> {
    fs::write(path, content).map_err(|source| WorkspaceError::WriteTemplate {
        path: path.display().to_string(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use kinein_protocol::{ProjectKind, WorkspaceProjectTemplate};

    use super::{create_directory, create_project};
    use crate::workspace::browse_directories;

    fn temp_workspace(test_name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("kinein-workspace-create-tests")
            .join(format!("{}-{test_name}", std::process::id()));
        if dir.exists() {
            fs::remove_dir_all(&dir).unwrap();
        }
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn create_directory_creates_child_and_browse_can_see_it() {
        let dir = temp_workspace("create-dir");

        let created = create_directory(&dir, "novo modulo").unwrap();
        let result = browse_directories(&dir).unwrap();

        assert!(created.is_dir());
        assert_eq!(result.entries[0].name, "novo modulo");
        assert_eq!(result.entries[0].path, created.display().to_string());
    }

    #[test]
    fn create_directory_rejects_path_like_name() {
        let dir = temp_workspace("create-dir-invalid");

        let error = create_directory(&dir, "../escape").unwrap_err();

        assert!(error.is_invalid_path());
        assert!(error.to_string().contains("nao um caminho"));
    }

    #[test]
    fn create_cpp_cmake_project_writes_strict_starter_files() {
        let dir = temp_workspace("create-cpp");

        let workspace =
            create_project(&dir, "demo_cpp", WorkspaceProjectTemplate::CppCmake).unwrap();

        assert_eq!(workspace.kind, ProjectKind::Cmake);
        assert_eq!(workspace.markers, ["CMakeLists.txt"]);
        let root = PathBuf::from(&workspace.root);
        assert!(root.join("CMakePresets.json").is_file());
        assert!(root.join("src/main.cpp").is_file());
        assert!(root.join("include").is_dir());
        assert!(root.join("tests").is_dir());
        assert!(root.join(".gitignore").is_file());
        let cmake = fs::read_to_string(root.join("CMakeLists.txt")).unwrap();
        assert!(cmake.contains("-Werror"));
        assert!(cmake.contains("target_compile_features"));
        assert!(cmake.contains("cxx_std_23"));
        assert!(!cmake.contains("CMAKE_CXX_STANDARD"));
        let presets = fs::read_to_string(root.join("CMakePresets.json")).unwrap();
        assert!(presets.contains("\"name\": \"debug\""));
        assert!(presets.contains("\"name\": \"release\""));
    }

    #[test]
    fn create_project_rejects_spaces_in_project_name() {
        let dir = temp_workspace("create-project-invalid");

        let error =
            create_project(&dir, "demo cpp", WorkspaceProjectTemplate::CppCmake).unwrap_err();

        assert!(error.is_invalid_path());
        assert!(error.to_string().contains("letras"));
    }
}

#[cfg(test)]
mod python_template_tests {
    use kinein_protocol::{ProjectKind, WorkspaceProjectTemplate};

    use super::create_project;

    /// O template Python (2026-09-13): PEP 621, layout plano, pytest nos
    /// extras — e o workspace ja' e' reconhecido como Python. Nada e'
    /// executado: um projeto nasce mesmo sem python3 na maquina.
    #[test]
    fn create_python_project_writes_a_pep_621_project_that_runs_without_installing() {
        let dir = std::env::temp_dir()
            .join("kinein-core-tests")
            .join(format!("{}-create-python", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let workspace = create_project(&dir, "demo-py", WorkspaceProjectTemplate::Python).unwrap();
        assert_eq!(workspace.kind, ProjectKind::Python);
        let root = dir.join("demo-py");
        let pyproject = std::fs::read_to_string(root.join("pyproject.toml")).unwrap();
        assert!(pyproject.contains("name = \"demo-py\""), "{pyproject}");
        assert!(pyproject.contains("dev = [\"pytest\"]"));
        assert!(
            pyproject.contains("pythonpath = [\".\"]"),
            "o pytest acha o pacote sem instalar"
        );
        // O nome vira modulo: `-` -> `_`, e o pacote fica na RAIZ.
        assert!(root.join("demo_py/__init__.py").is_file());
        assert!(!root.join("src").exists(), "layout plano");
        let main = std::fs::read_to_string(root.join("main.py")).unwrap();
        assert!(main.contains("from demo_py import saudacao"));
        assert!(
            std::fs::read_to_string(root.join("tests/test_main.py"))
                .unwrap()
                .contains("from demo_py import")
        );
        assert!(
            std::fs::read_to_string(root.join(".gitignore"))
                .unwrap()
                .contains(".venv/")
        );
        // O botao Executar acha o main.py como ponto de entrada.
        assert_eq!(
            crate::python::run::entry_point(&root),
            Some(crate::python::run::EntryPoint::File("main.py".to_owned()))
        );
        // Existe: recusa.
        assert!(create_project(&dir, "demo-py", WorkspaceProjectTemplate::Python).is_err());
    }
}
