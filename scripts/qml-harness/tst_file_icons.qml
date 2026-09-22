import QtQuick
import KineinVectis

// O mesmo resolver alimenta a arvore do projeto e as abas do editor. Nomes
// especiais precisam vencer extensoes genericas, e headers C/C++ nao podem
// voltar a compartilhar um icone por acidente.
Item {
    KvFileIcon { id: icon }

    Component.onCompleted: {
        let failures = 0;

        if (icon.resolvedIconName("CMakeLists.txt", false, false)
                !== "tree-file-cmakelists") failures += 1;
        if (icon.resolvedIconName("Dockerfile.arm64", false, false)
                !== "tree-file-docker") failures += 2;
        if (icon.resolvedIconName("compose.dev.yaml", false, false)
                !== "tree-file-docker") failures += 4;
        if (icon.resolvedIconName("config.yaml", false, false)
                !== "tree-file-yaml") failures += 8;
        if (icon.resolvedIconName("api.h", false, false)
                !== "tree-file-h") failures += 16;
        if (icon.resolvedIconName("api.hpp", false, false)
                !== "tree-file-hpp") failures += 32;
        if (icon.resolvedIconName("main.cpp", false, false)
                !== "tree-file-cpp") failures += 64;
        if (icon.resolvedIconName("main.c", false, false)
                !== "tree-file-c") failures += 131072;
        if (icon.resolvedIconName("query.sql", false, false)
                !== "tree-file-sql") failures += 128;
        if (icon.resolvedIconName("README.md", false, false)
                !== "tree-file-markdown") failures += 256;
        if (icon.resolvedIconName("src", true, false)
                !== "tree-folder-closed") failures += 512;
        if (icon.resolvedIconName("src", true, true)
                !== "tree-folder-open") failures += 1024;
        if (icon.resolvedIconName("Cargo.toml", false, false)
                !== "tree-file-cargo") failures += 2048;
        if (icon.resolvedIconName("Makefile", false, false)
                !== "tree-file-makefile") failures += 4096;
        if (icon.resolvedIconName("rules.mk", false, false)
                !== "tree-file-makefile") failures += 8192;
        if (icon.resolvedIconName("pyproject.toml", false, false)
                !== "tree-file-pyproject") failures += 16384;
        if (icon.resolvedIconName("package.xml", false, false)
                !== "tree-file-ros") failures += 32768;
        if (icon.resolvedIconName("robot.launch.xml", false, false)
                !== "tree-file-ros") failures += 65536;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
