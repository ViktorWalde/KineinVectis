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

    function label(kind) {
        return kinds[kind] !== undefined ? kinds[kind].label : "";
    }

    function marker(kind) {
        return kinds[kind] !== undefined ? kinds[kind].marker : "";
    }
}
