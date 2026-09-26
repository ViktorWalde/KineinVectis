import QtQuick

// O TOKEN DE COMENTARIO DE CADA LINGUAGEM.
//
// Saiu do `EditorController` em 2026-09-25, e nao por tamanho: e' uma TABELA
// que cresce a cada linguagem nova, e uma tabela que cresce dentro de um
// arquivo de 795 linhas e' uma tabela que ninguem acha.
//
// Linguagem sem token conhecido devolve vazio, e quem chama nao comenta nada —
// inventar `//` para um `.toml` estragaria o arquivo.
QtObject {
    id: root

    readonly property var tokenByLanguage: ({
        "c": "//",
        "cpp": "//",
        "rust": "//",
        "js": "//",
        "python": "#",
        "shell": "#",
        "cmake": "#",
        "toml": "#"
    })

    function tokenFor(language) {
        const token = root.tokenByLanguage[language];
        return token === undefined ? "" : token;
    }
}
