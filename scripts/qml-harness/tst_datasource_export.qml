import QtQuick
import "../../ui/qml/datasource"
// O singleton so' e' singleton pelo modulo; o diretorio o veria como tipo.
import KineinVectis as Kv

// EXPORTAR EM CSV (passo 14b, roadmaps/59 §5.4.1): o DataSourceExportController
// REAL com a ponte falsa.
//
// O que se prova: o nome saneado e datado; a pasta antes do arquivo; a falha
// da pasta (ja' existe) nao para nada e o arquivo diz a verdade; um pedido por
// vez; resposta de outro caminho nao fecha o pedido; trocar de projeto o
// esquece.
//
// MUTACOES QUE PROVAM O GATE: a falha da pasta encerrar a exportacao; o
// `fileDone` sem conferir o caminho; o `begin` sem a guarda de `busy`.
Item {
    id: root

    property var directories: []
    property var files: []
    property var outcomes: []

    DataSourceExportController {
        id: exports

        onDirectoryRequested: path => root.directories.push(path)
        onFileRequested: (path, content) => root.files.push({ path: path, content: content })
        onFinished: (message, ok) => root.outcomes.push({ message: message, ok: ok })
    }

    Component.onCompleted: {
        let failures = 0;
        const when = new Date(2026, 9, 8, 7, 5, 3);

        // o nome: saneado, sem ponto no comeco, datado na hora local
        if (Kv.DataSourceKinds.exportPath("minha conexão/x", when) !== "exportacoes/minha-conex-o-x-20261008-070503.csv"
            || Kv.DataSourceKinds.exportPath("..", when) !== "exportacoes/dados-20261008-070503.csv"
            || Kv.DataSourceKinds.exportPath(".oculto", when) !== "exportacoes/oculto-20261008-070503.csv") failures += 1;

        // sem projeto, nada sai
        exports.begin("estacao", "id\r\n", 0, when);
        if (root.directories.length !== 0 || exports.busy) failures += 2;

        // a pasta primeiro; um segundo pedido espera o primeiro acabar
        exports.workspaceRoot = "/w";
        exports.begin("estacao", "id\r\n1\r\n", 1, when);
        exports.begin("estacao", "outro", 9, when);
        if (root.directories.length !== 1 || root.directories[0] !== "/w/exportacoes" || !exports.busy) failures += 4;

        // a pasta ja' existia: a falha dela nao e' relatada e o arquivo segue
        exports.handleFailed("fs.createDirectory", "/w/exportacoes", "o arquivo /w/exportacoes ja existe");
        if (root.files.length !== 1 || root.files[0].path !== "/w/exportacoes/estacao-20261008-070503.csv"
            || root.files[0].content !== "id\r\n1\r\n" || root.outcomes.length !== 0) failures += 8;

        // a resposta de outro caminho nao fecha este pedido
        exports.handleSucceeded("fs.createFile", "/w/exportacoes/outro.csv");
        if (!exports.busy || root.outcomes.length !== 0) failures += 16;
        exports.handleSucceeded("fs.createFile", "/w/exportacoes/estacao-20261008-070503.csv");
        if (exports.busy || root.outcomes.length !== 1 || !root.outcomes[0].ok
            || root.outcomes[0].message.indexOf("exportacoes/estacao-20261008-070503.csv") < 0
            || root.outcomes[0].message.indexOf("(1 linha") < 0) failures += 32;

        // o arquivo falha: a mensagem do core chega, e o pedido termina
        exports.begin("estacao", "x", 1, when);
        exports.handleSucceeded("fs.createDirectory", "/w/exportacoes");
        exports.handleFailed("fs.createFile", "/w/exportacoes/estacao-20261008-070503.csv", "o arquivo ja existe");
        if (exports.busy || root.outcomes.length !== 2 || root.outcomes[1].ok
            || root.outcomes[1].message.indexOf("o arquivo ja existe") < 0) failures += 64;

        // trocar de projeto esquece o pedido; a resposta antiga nao fecha nada
        exports.begin("estacao", "x", 1, when);
        exports.workspaceRoot = "/outro";
        exports.handleSucceeded("fs.createFile", "/w/exportacoes/estacao-20261008-070503.csv");
        if (exports.busy || root.outcomes.length !== 2) failures += 128;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
