import QtQuick

// O EXPLORER SEGUE O ARQUIVO ATIVO (Etapa 2, F3 do roadmaps/43, 2026-09-18):
// o "autoscroll from source" da referencia. Abre as pastas ate' o arquivo,
// uma listagem por vez (o core responde `fs.list` por pasta), e o seleciona
// quando ele aparece na arvore. Um arquivo fora do workspace nao revela
// nada; o pedido mais novo substitui o anterior. Filho do
// ProjectTreeController (que bateu em 400 ao ganhar isto): le a arvore por
// ele e pede as listagens por ele.
Item {
    id: root

    property var tree: null
    property string target: ""
    // Pastas ja' pedidas e ainda sem resposta: pedir duas vezes faria a
    // segunda listagem RECOLHER a pasta (setDirectoryListing recolhe antes de
    // inserir) e derrubar o que a primeira ja' tinha aberto abaixo dela.
    property var requested: ({})

    visible: false

    function revealPath(path) {
        if (tree === null || path === "" || tree.workspaceRoot === ""
                || path.indexOf(tree.workspaceRoot + "/") !== 0) {
            target = "";
            return;
        }
        target = path;
        advance();
    }

    // Chamado a cada listagem que chega: seleciona se o alvo ja' esta' na
    // arvore, senao abre o ancestral mais fundo que esta'.
    function listed(path) {
        delete requested[path];
    }

    function advance() {
        if (target === "" || tree === null) {
            return;
        }
        const alvo = tree.rowIndexForPath(target);
        if (alvo >= 0) {
            tree.selectEntry(target, tree.entriesModel.get(alvo).kind);
            target = "";
            return;
        }
        let dir = tree.parentDir(target);
        while (dir !== "" && dir !== tree.workspaceRoot) {
            const i = tree.rowIndexForPath(dir);
            if (i >= 0) {
                if (tree.entriesModel.get(i).expanded) {
                    delete requested[dir];
                } else if (requested[dir] !== true) {
                    requested[dir] = true;
                    tree.listDirRequested(dir);
                }
                return;
            }
            dir = tree.parentDir(dir);
        }
        // Nenhum ancestral na arvore: a raiz ainda nao foi listada — a
        // proxima listagem chama de novo.
    }
}
