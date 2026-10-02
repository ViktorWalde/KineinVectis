import QtQuick

// O que "Criar Projeto" oferece, como DADO (0.3.6, decisao do autor de
// 2026-10-01, roadmap 53 §13.0 item 6): primeiro a LINGUAGEM, depois o
// ECOSSISTEMA dela. Ate' aqui a tela inicial tinha dois botoes fixos — "Novo
// C++ / CMake" e "Novo Rust / Cargo" — e o Python, que o core ja' criava
// (`WorkspaceProjectTemplate::Python`, 0.105.0), so' aparecia escondido no
// seletor de pasta.
//
// Cada ecossistema e' um `template` que o core aceita no
// `workspace.createProject`; nada aqui promete o que o core nao cria. Um
// ecossistema novo (PlatformIO na 0.4, roadmap 52) e' uma entrada nesta lista,
// sem tocar no painel. A previa mora junto do template: um fato, um dono.
QtObject {
    id: root

    readonly property var languages: [
        {
            "key": "cpp", "label": "C/C++", "iconFile": "main.cpp",
            "ecosystems": [
                {
                    "template": "cppCmake", "label": "CMake",
                    "detail": qsTr("C++23, presets Debug e Release com Ninja"),
                    "files": ["CMakeLists.txt  · C++23 target-based",
                              "CMakePresets.json  · Debug + Release / Ninja",
                              "src/main.cpp", "include/  tests/", ".gitignore  README.md"],
                    "command": ""
                }
            ]
        },
        {
            "key": "rust", "label": "Rust", "iconFile": "main.rs",
            "ecosystems": [
                {
                    "template": "rustCargo", "label": "Cargo",
                    "detail": qsTr("binário criado pelo cargo new"),
                    "files": ["Cargo.toml", "src/main.rs"],
                    "command": "cargo new --bin --vcs none %1"
                }
            ]
        },
        {
            "key": "python", "label": "Python", "iconFile": "main.py",
            "ecosystems": [
                {
                    "template": "python", "label": "pyproject.toml",
                    "detail": qsTr("PEP 621, pytest e ruff; o .venv é um clique depois"),
                    "files": ["pyproject.toml  · PEP 621, pytest em [dev], ruff",
                              "main.py  · o ponto de entrada do Executar",
                              "%2/__init__.py  tests/test_main.py", ".gitignore  README.md"],
                    "command": ""
                }
            ]
        },
        {
            "key": "empty", "label": qsTr("Vazio"), "iconFile": "",
            "ecosystems": [
                {
                    "template": "empty", "label": qsTr("Pasta vazia"),
                    "detail": qsTr("a IDE detecta o projeto depois"),
                    "files": [],
                    "command": ""
                }
            ]
        }
    ]

    function language(key) {
        for (let i = 0; i < languages.length; i++) {
            if (languages[i].key === key) {
                return languages[i];
            }
        }
        return null;
    }

    function ecosystems(languageKey) {
        const found = language(languageKey);
        return found !== null ? found.ecosystems : [];
    }

    // A linguagem dona de um template; "" para template desconhecido ou vazio.
    function languageOf(templateId) {
        for (let i = 0; i < languages.length; i++) {
            const list = languages[i].ecosystems;
            for (let j = 0; j < list.length; j++) {
                if (list[j].template === templateId) {
                    return languages[i].key;
                }
            }
        }
        return "";
    }

    function ecosystem(templateId) {
        const list = ecosystems(languageOf(templateId));
        for (let i = 0; i < list.length; i++) {
            if (list[i].template === templateId) {
                return list[i];
            }
        }
        return null;
    }

    // O primeiro ecossistema da linguagem: o que escolher a linguagem seleciona.
    function defaultTemplate(languageKey) {
        const list = ecosystems(languageKey);
        return list.length > 0 ? list[0].template : "";
    }

    // A previa: arvore de arquivos e o comando externo, se houver.
    function preview(templateId, projectName) {
        const name = projectName.trim() !== "" ? projectName.trim() : qsTr("meu-projeto");
        const chosen = ecosystem(templateId);
        if (chosen === null) {
            return qsTr("Escolha a linguagem e o ecossistema.");
        }
        if (chosen.files.length === 0) {
            return name + "/  " + qsTr("(diretório vazio)");
        }
        const moduleName = name.replace(/-/g, "_");
        const lines = [name + "/"];
        for (let i = 0; i < chosen.files.length; i++) {
            lines.push("  " + chosen.files[i].replace("%2", moduleName));
        }
        lines.push(chosen.command !== ""
                   ? qsTr("Comando: %1").arg(chosen.command.replace("%1", name))
                   : qsTr("Geração interna: nenhum comando externo"));
        return lines.join("\n");
    }
}
