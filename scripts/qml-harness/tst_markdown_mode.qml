import QtQuick
// Carrega o controller REAL do modo "imagem" dos .md.
import "../../ui/qml/editor"

Item {
    id: root
    width: 100
    height: 100

    EditorMarkdownModeController {
        id: mode
    }

    Component.onCompleted: {
        let failures = 0;

        // Deteccao por extensao, caso-insensitiva.
        if (!mode.isMarkdownPath("docs/README.md")) failures += 1;
        if (!mode.isMarkdownPath("PONTO_ATUAL.MD")) failures += 2;
        if (mode.isMarkdownPath("src/main.rs")) failures += 4;
        if (mode.isMarkdownPath("md")) failures += 8;

        // Estado inicial: nada em preview.
        if (mode.isPreview("docs/README.md", mode.revision)) failures += 16;

        // Alternar liga; alternar de novo desliga.
        mode.toggle("docs/README.md");
        if (!mode.isPreview("docs/README.md", mode.revision)) failures += 32;
        mode.toggle("docs/README.md");
        if (mode.isPreview("docs/README.md", mode.revision)) failures += 64;

        // O estado e POR ARQUIVO: ligar um nao vaza para o outro.
        mode.toggle("a.md");
        if (mode.isPreview("b.md", mode.revision)) failures += 128;
        if (!mode.isPreview("a.md", mode.revision)) failures += 256;

        // Arquivo que nao e markdown: toggle e um no-op COMPLETO — nem o
        // revision anda, senao o rebind dispararia sem mudanca real.
        const before = mode.revision;
        mode.toggle("src/main.rs");
        if (mode.revision !== before) failures += 512;
        if (mode.isPreview("src/main.rs", mode.revision)) failures += 1024;

        // clear() descarta tudo (fechar workspace nao pode vazar estado
        // para o proximo projeto — caminhos relativos colidem).
        mode.toggle("c.md");
        mode.clear();
        if (mode.isPreview("a.md", mode.revision)
                || mode.isPreview("c.md", mode.revision)) failures += 2048;

        // O contrato do rebind: toggle valido INCREMENTA revision.
        const rev = mode.revision;
        mode.toggle("d.md");
        if (mode.revision <= rev) failures += 4096;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
