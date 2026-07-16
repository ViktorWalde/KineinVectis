import QtQuick
// Carrega o EditorController.qml REAL (arquivo do projeto, sem copia).
import "../../ui/qml/editor"

// Protocolo 0.61.0: quem decide o que e formatavel e o CORE. A UI consome o
// catalogo de `format.capabilities` e nao mantem lista propria.
//
// Regressao que isto protege: ate 0.60 o EditorController tinha DUAS listas
// escritas a mao — `formattableLanguage()` (rust|cpp) e `formattablePath()`
// (9 extensoes) — que nem concordavam entre si, enquanto `formatter_for_path`
// no core ja era a autoridade. Duas fontes para a mesma verdade divergem por
// construcao, e adicionar linguagem exigia editar QML.
//
// Se alguem reintroduzir uma lista literal aqui, o teste do catalogo vazio
// quebra: uma lista hardcoded responderia "sim" sem o core ter falado.
Item {
    id: root
    width: 100
    height: 100

    EditorController {
        id: editor
    }

    Component.onCompleted: {
        let failures = 0;

        // Antes de o catalogo chegar, NADA e formatavel. Nao formatar por nao
        // saber ainda e o comportamento seguro — salvar nunca trava por isso.
        // Uma lista hardcoded falharia aqui.
        if (editor.formattablePath("/w/src/main.rs")) failures += 1;
        if (editor.formattablePath("/w/src/main.cpp")) failures += 2;

        // O catalogo do core define a verdade — inclusive uma extensao que a
        // lista antiga nao conhecia.
        editor.applyFormatCapabilities([
            { "id": "rustfmt", "extensions": ["rs"] },
            { "id": "clang-format",
              "extensions": ["c", "cc", "cpp", "cxx", "h", "hh", "hpp", "hxx"] }
        ]);

        if (!editor.formattablePath("/w/src/main.rs")) failures += 4;
        if (!editor.formattablePath("/w/src/main.cpp")) failures += 8;
        if (!editor.formattablePath("/w/inc/api.hpp")) failures += 16;
        if (editor.formattablePath("/w/README.md")) failures += 32;
        if (editor.formattablePath("/w/Cargo.toml")) failures += 64;

        // Sem extensao nao e formatavel — e um ponto no diretorio nao conta.
        if (editor.formattablePath("/w/Makefile")) failures += 128;
        if (editor.formattablePath("/w/.config/arquivo")) failures += 256;
        if (editor.formattablePath("/w/dir.v2/Makefile")) failures += 512;

        // Extensao e case-insensitive, como no core (`to_lowercase`).
        if (!editor.formattablePath("/w/src/MAIN.RS")) failures += 1024;
        if (!editor.formattablePath("/w/src/Main.Cpp")) failures += 2048;

        // A UI acompanha o catalogo: se o core registrar um formatter novo, a
        // UI passa a aceitar SEM mudanca de codigo. Era o ponto da fatia.
        editor.applyFormatCapabilities([
            { "id": "black", "extensions": ["py"] }
        ]);
        if (!editor.formattablePath("/w/script.py")) failures += 4096;
        if (editor.formattablePath("/w/src/main.rs")) failures += 8192;

        // Catalogo vazio (core sem formatter) nao formata nada.
        editor.applyFormatCapabilities([]);
        if (editor.formattablePath("/w/src/main.rs")) failures += 16384;

        Qt.exit(failures);
    }
}
