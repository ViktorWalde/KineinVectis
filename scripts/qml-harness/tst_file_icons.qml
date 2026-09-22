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
                !== "tree-file-c") failures += 64;
        if (icon.resolvedIconName("query.sql", false, false)
                !== "tree-file-sql") failures += 128;
        if (icon.resolvedIconName("README.md", false, false)
                !== "tree-file-markdown") failures += 256;
        if (icon.resolvedIconName("src", true, false)
                !== "tree-folder-closed") failures += 512;
        if (icon.resolvedIconName("src", true, true)
                !== "tree-folder-open") failures += 1024;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
