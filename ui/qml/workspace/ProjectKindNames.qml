pragma Singleton
import QtQuick

// Nome e arquivo-marca de cada `ProjectKind` do core: o dono do mapa. O
// rodape do shell diz "Rust/Cargo" com ele, e o seletor de pastas marca a
// pasta de projeto com o mesmo nome e o icone do arquivo que a fez projeto
// (0.147.0). O "desconhecido" fica com quem chama (traduz o proprio texto).
QtObject {
    readonly property var kinds: ({
        rustCargo: { label: "Rust/Cargo", marker: "Cargo.toml" },
        cmake: { label: "CMake", marker: "CMakeLists.txt" },
        maven: { label: "Maven", marker: "pom.xml" },
        gradle: { label: "Gradle", marker: "build.gradle" },
        python: { label: "Python", marker: "pyproject.toml" },
        make: { label: "Make", marker: "Makefile" },
        platformIo: { label: "PlatformIO", marker: "platformio.ini" }
    })

    // Cada sistema de build e a linguagem que a IDE suporta por ele (0.152.0).
    // Maven e Gradle sao reconhecidos, mas Java nao e' linguagem da IDE: nao
    // contam para "hibrido".
    readonly property var systems: ({
        cargo: { label: "Cargo", language: "Rust" },
        cmake: { label: "CMake", language: "C/C++" },
        make: { label: "Make", language: "C/C++" },
        platformIo: { label: "PlatformIO", language: "C/C++" },
        python: { label: "Python", language: "Python" },
        maven: { label: "Maven", language: "" },
        gradle: { label: "Gradle", language: "" }
    })

    // As linguagens, sem repetir, na ordem dos marcadores: [{ language,
    // systems: ["Cargo"] }].
    function languages(buildSystems) {
        const out = [];
        for (const key of buildSystems.split(",")) {
            const system = systems[key];
            if (system === undefined || system.language === "") continue;
            const found = out.filter(function(entry) { return entry.language === system.language; })[0];
            if (found !== undefined) found.systems.push(system.label);
            else out.push({ language: system.language, systems: [system.label] });
        }
        return out;
    }

    // Hibrido: duas ou mais linguagens que a IDE suporta (Rust e C/C++, como
    // o proprio Kinein).
    function isHybrid(buildSystems) {
        return languages(buildSystems).length >= 2;
    }

    // "Rust (Cargo) · C/C++ (CMake)"
    function hybridDescription(buildSystems) {
        return languages(buildSystems).map(function(entry) {
            return entry.language + " (" + entry.systems.join(", ") + ")";
        }).join(" · ");
    }

    function label(kind) {
        return kinds[kind] !== undefined ? kinds[kind].label : "";
    }

    function marker(kind) {
        return kinds[kind] !== undefined ? kinds[kind].marker : "";
    }
}
