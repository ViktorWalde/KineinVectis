// A colagem usa o controller real: pedido, colisão, resposta e recorte.
import QtQuick
import KineinVectis

Item {
    id: root
    property int failures: 0
    property var copies: []
    property var moves: []
    property var batches: []
    property var listed: []
    property var renamedTabs: []

    QtObject {
        id: fakeClipboard
        property bool filesAvailable: false
        property var paths: []
        property bool cut: false
        property string copiedText: ""
        function setText(value) { copiedText = value; paths = []; cut = false; filesAvailable = false; }
        function setFiles(values, asCut) {
            paths = values;
            cut = asCut;
            filesAvailable = values.length > 0;
        }
        function filePaths() { return paths; }
        function filesCut() { return cut; }
        function clearCutFileIfMatches(path) {
            if (cut && paths.length === 1 && paths[0] === path) {
                paths = [];
                cut = false;
                filesAvailable = false;
            }
        }
        function removeCutFilesIfMatches(expected, moved) {
            if (!cut || paths.length !== expected.length
                    || paths.some((path, index) => path !== expected[index])) return;
            paths = paths.filter(path => moved.indexOf(path) < 0);
            filesAvailable = paths.length > 0;
            if (!filesAvailable) cut = false;
        }
    }

    ProjectTreeController {
        id: tree
        workspaceRoot: "/w"
        clipboard: fakeClipboard
        onCopyPathRequested: (from, to) => root.copies.push([from, to])
        onRenamePathRequested: (from, to) => root.moves.push([from, to])
        onTransferBatchRequested: (operation, items) => root.batches.push([operation, items])
        onListDirRequested: path => root.listed.push(path)
        onTabsRenameRequested: (from, to) => root.renamedTabs.push([from, to])
    }

    function check(condition, message) {
        if (!condition) { console.error("FALHOU: " + message); failures += 1; }
    }

    Component.onCompleted: {
        tree.setDirectoryListing("/w", [
            {name: "dst", kind: "directory"}, {name: "a.bin", kind: "file"}
        ]);
        tree.selectEntry("/w/a.bin", "file");
        check(tree.fileClipboard.copySelection(false), "copiar item da seleção");
        check(fakeClipboard.paths[0] === "/w/a.bin" && !fakeClipboard.cut,
              "clipboard perdeu a origem");
        tree.selectEntry("/w/dst", "directory");
        check(tree.fileClipboard.openPaste(), "abrir cópia no destino");
        tree.fileClipboard.confirm("a.bin");
        check(copies.length === 1 && copies[0][1] === "/w/dst/a.bin",
              "cópia enviou destino incorreto");
        tree.handleCopyFailed("/outro/arquivo", "erro antigo");
        check(tree.fileClipboard.pending, "erro de outro destino afetou pedido ativo");
        tree.handleCopyFailed("/w/dst/a.bin", "já existe");
        check(tree.fileClipboard.dialogVisible && !tree.fileClipboard.pending
              && tree.fileClipboard.errorText === "já existe", "colisão apagou diálogo");
        tree.fileClipboard.confirm("a-copia.bin");
        tree.fileClipboard.dialogVisible = false; // usuário foi acompanhar em Jobs
        tree.handlePathCopied("/w/a.bin", "/w/dst/a-copia.bin");
        check(!tree.fileClipboard.dialogVisible && tree.selectedPath === "/w/dst/a-copia.bin",
              "sucesso não atualizou seleção");
        check(listed[listed.length - 1] === "/w/dst", "sucesso não atualizou pasta");

        tree.selectEntry("/w/dst", "directory");
        tree.fileClipboard.copySelection(false);
        tree.fileClipboard.openPaste();
        check(tree.fileClipboard.destinationDirectory === "/w",
              "duplicar pasta escolheu a própria pasta como destino");
        tree.fileClipboard.clear();

        tree.selectEntry("/w/a.bin", "file");
        tree.fileClipboard.copySelection(true);
        tree.selectEntry("/w/dst", "directory");
        tree.fileClipboard.openPaste();
        tree.fileClipboard.confirm("movido.bin");
        check(moves.length === 1 && moves[0][1] === "/w/dst/movido.bin",
              "recorte criou outro motor de movimento");
        tree.handlePathRenamed("/w/a.bin", "/w/dst/movido.bin");
        check(!fakeClipboard.filesAvailable && renamedTabs.length === 1,
              "recorte não limpou clipboard ou preservou abas");
        check(listed.indexOf("/w") >= 0, "movimento não atualizou origem");

        tree.setDirectoryListing("/w", [
            {name: "dst", kind: "directory"},
            {name: "a.bin", kind: "file"}, {name: "b.bin", kind: "file"}
        ]);
        tree.selectEntry("/w/a.bin", "file");
        tree.selectEntry("/w/b.bin", "file", Qt.ControlModifier);
        check(tree.fileClipboard.copySelection(true) && fakeClipboard.paths.length === 2,
              "recorte múltiplo não preservou seleção");
        tree.selectEntry("/w/dst", "directory");
        check(tree.fileClipboard.openPaste() && tree.fileClipboard.batchDialogVisible,
              "lote não abriu revisão");
        const draft = tree.fileClipboard.batchEntries.map(entry => ({
            from: entry.from, name: entry.name, included: entry.included,
            status: entry.status, error: entry.error
        }));
        draft[1].name = "b-moved.bin";
        tree.fileClipboard.confirmBatch(draft);
        check(batches.length === 1 && batches[0][0] === "move"
              && batches[0][1][1].to === "/w/dst/b-moved.bin",
              "lote não respeitou nomes revisados");
        tree.fileClipboard.batchDialogVisible = false;
        tree.fileClipboard.batchTransferred({operation: "move", items: [
            {from: "/w/a.bin", to: "/w/dst/a.bin", status: "success"},
            {from: "/w/b.bin", to: "/w/dst/b-moved.bin", status: "failed", error: "já existe"}
        ]});
        check(tree.fileClipboard.batchDialogVisible && !tree.fileClipboard.batchPending
              && fakeClipboard.paths.length === 1 && fakeClipboard.paths[0] === "/w/b.bin",
              "falha parcial repetiria sucesso ou perderia recorte");
        check(renamedTabs.length === 2 && tree.fileClipboard.batchEntries[0].included === false,
              "sucesso parcial não atualizou abas/revisão");
        tree.fileClipboard.confirmBatch();
        check(batches.length === 2 && batches[1][1].length === 1,
              "tentativa seguinte reenviou item concluído");
        tree.fileClipboard.batchTransferred({operation: "move", items: [
            {from: "/w/b.bin", to: "/w/dst/b-moved.bin", status: "success"}
        ]});
        check(!fakeClipboard.filesAvailable && !tree.fileClipboard.batchDialogVisible,
              "sucesso final não limpou recorte/lote");

        fakeClipboard.setFiles(["/w/b.bin"], true);
        check(tree.fileClipboard.openTransfer(["/w/a.bin"], false, "/w/dst", false),
              "arrasto interno não reutilizou o diálogo de cópia");
        tree.fileClipboard.confirm("a-drag.bin");
        check(copies[copies.length - 1][1] === "/w/dst/a-drag.bin",
              "arrasto enviou destino errado");
        tree.fileClipboard.copied("/w/a.bin", "/w/dst/a-drag.bin");
        check(fakeClipboard.filesCut() && fakeClipboard.paths[0] === "/w/b.bin",
              "arrasto modificou o clipboard independente");

        check(tree.fileClipboard.openImport(["/tmp/ação #1.bin"], "/w/dst")
              && tree.fileClipboard.batchDialogVisible && tree.fileClipboard.batchImport,
              "importação externa não abriu revisão de lote");
        tree.fileClipboard.confirmBatch();
        check(batches.length === 3 && batches[2][0] === "import"
              && batches[2][1][0].to === "/w/dst/ação #1.bin",
              "importação não preservou path e nome Unicode");
        tree.fileClipboard.batchTransferred({operation: "import", items: [
            {from: "/tmp/ação #1.bin", to: "/w/dst/ação #1.bin", status: "success"}
        ]});
        check(!tree.fileClipboard.batchDialogVisible && fakeClipboard.filesCut()
              && fakeClipboard.paths[0] === "/w/b.bin",
              "importação alterou clipboard ou deixou diálogo aberto");

        tree.selectEntry("/w/a.bin", "file");
        tree.selectEntry("/w/b.bin", "file", Qt.ControlModifier);
        check(tree.fileClipboard.copySelectedPaths(true)
              && fakeClipboard.copiedText === "a.bin\nb.bin",
              "copiar caminhos relativos não usou o clipboard de texto");
        check(tree.fileClipboard.copySelectedPaths(false)
              && fakeClipboard.copiedText === "/w/a.bin\n/w/b.bin",
              "copiar caminhos absolutos perdeu a seleção múltipla");
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
