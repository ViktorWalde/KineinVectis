import QtQuick

// A lista de arquivos ABERTOS RECENTEMENTE (MRU).
//
// POR QUE ESTE ARQUIVO EXISTE (2026-09-03). E' uma lista de caminhos com
// politica propria — mais recente na frente, sem repeticao, teto de 50 — que
// nao tem nada a ver com abas, buffers ou disco, e mesmo assim vivia dentro do
// EditorDocumentController (548 linhas, quatro vocabularios). Quem consome e' o
// "Buscar em todo lugar" (SearchEverywhereController), nao o editor.
//
// O TETO DE 50 e' deliberado: a lista alimenta um seletor onde ninguem rola
// alem das primeiras dezenas, e um MRU sem teto cresce para sempre na sessao.
Item {
    id: root

    visible: false

    property var rules: null
    property var paths: []

    function reset() {
        paths = [];
    }

    // Poe `path` na frente, tirando a ocorrencia anterior.
    function touch(path) {
        if (path === "") {
            return;
        }
        const updated = [path];
        for (let i = 0; i < paths.length && updated.length < 50; i++) {
            if (paths[i] !== path) {
                updated.push(paths[i]);
            }
        }
        paths = updated;
    }

    // Um diretorio renomeado reescreve todos os recentes debaixo dele; sem
    // isso a lista aponta para caminhos que nao existem mais.
    function applyRename(from, to) {
        const updated = [];
        for (let i = 0; i < paths.length; i++) {
            updated.push(rules.renamed(paths[i], from, to));
        }
        paths = updated;
    }
}
