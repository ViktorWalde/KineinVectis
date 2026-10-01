pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O que o CLIQUE DIREITO no explorer abre: menu de contexto, renomear, colar, apagar.
//
// POR QUE ESTE ARQUIVO EXISTE (2026-09-04). O `ShellOverlays` passou de 300
// linhas ao ganhar o painel de fontes de dados, e a catraca disparou. Os tres
// suspeitos da §4 regra 9, na ordem: a mudanca, a categoria, o arquivo.
//
//   a mudanca    onze linhas de fiacao legitima de um overlay novo — nao e' o
//                defeito, e' o gatilho;
//   a categoria  o `ShellOverlays` nao desenha um pixel proprio; e' composicao,
//                medida contra o limite de QML VISUAL. Ha' um caso registrado
//                identico (`AppDomains`, §4 regra 9);
//   o arquivo    e aqui esta' o corte de verdade: dezesseis overlays, e TRES
//                deles formam uma area com dono unico — o explorer.
//
// A §4 regra 8 manda dividir composicao POR AREA, fazendo a contagem de
// arquivos crescer em vez do tamanho. E' o que este arquivo faz, e o teste de
// que a area e' real esta' na interface: ela precisa de TRES controllers, nao
// dos doze que o `ShellOverlays` carrega.
Item {
    id: root

    property real hostWidth: 0
    property var projectTree: null
    // Para mostrar o caminho relativo a raiz do projeto no dialogo de renomear.
    property var shellController: null
    property var runtimeController: null
    // Cancelar um dialogo do explorer devolve o foco a arvore.

    // O nome pre-preenchido vem de fora (a paleta abre o renomear com o nome
    // atual), entao a entrada e' funcao, nao propriedade.
    function openEntryRenameWithName(name) {
        entryRenameDialog.openWithName(name);
    }

    ProjectEntryContextMenu {
        anchors.fill: parent
        visible: root.projectTree.entryMenuVisible
        z: 100
        menuX: root.projectTree.entryMenuX
        menuY: root.projectTree.entryMenuY
        runnableScript: root.projectTree.entryMenuRunnable
        debuggableScript: root.projectTree.entryMenuDebuggable
        selectionCount: root.projectTree.selectedPaths.length
        pasteAvailable: root.projectTree.fileClipboard.pasteAvailable
        onDismissRequested: root.projectTree.dismissEntryMenu()
        onCreateFileRequested: root.projectTree.openEntryCreate("file")
        onCreateDirectoryRequested: root.projectTree.openEntryCreate("directory")
        onRunScriptRequested: root.projectTree.runEntryScript()
        onDebugScriptRequested: root.projectTree.debugEntryScript()
        onRenameRequested: root.projectTree.openEntryRename()
        onDeleteRequested: root.projectTree.openEntryDelete()
        onCopyRequested: root.projectTree.fileClipboard.copySelection(false)
        onCutRequested: root.projectTree.fileClipboard.copySelection(true)
        onPasteRequested: root.projectTree.fileClipboard.openPaste()
        onCopyAbsolutePathRequested: root.projectTree.fileClipboard.copySelectedPaths(false)
        onCopyRelativePathRequested: root.projectTree.fileClipboard.copySelectedPaths(true)
        onOpenFolderRequested: {
            Qt.openUrlExternally(Clipboard.localFileUrl(root.projectTree.entryMenuDirectory()));
            root.projectTree.dismissEntryMenu();
        }
        onOpenTerminalRequested: {
            root.runtimeController.newTerminalAt(root.projectTree.entryMenuDirectory());
            root.projectTree.dismissEntryMenu();
        }
    }

    ProjectEntryRenameDialog {
        id: entryRenameDialog

        anchors.fill: parent
        visible: root.projectTree.entryRenameVisible
        z: 101
        entryKind: root.projectTree.entryRenameKind
        entryDisplayPath: root.shellController.relativeToRoot(
                              root.projectTree.entryRenamePath)
        errorText: root.projectTree.entryRenameError
        maxAvailableWidth: root.hostWidth - 4 * Theme.spacingMedium
        onConfirmRequested: root.projectTree.confirmEntryRename(
                                entryRenameDialog.currentName())
        onCancelRequested: {
            root.projectTree.entryRenameVisible = false;
            root.projectTree.focusTreeRequested();
        }
    }

    ProjectEntryRenameDialog {
        id: pasteDialog

        anchors.fill: parent
        visible: root.projectTree.fileClipboard.dialogVisible
        z: 103
        titleText: root.projectTree.fileClipboard.cut ? qsTr("Mover para") : qsTr("Copiar para")
        confirmText: root.projectTree.fileClipboard.cut ? qsTr("Mover") : qsTr("Copiar")
        operationPending: root.projectTree.fileClipboard.pending
        pendingDismissText: root.projectTree.fileClipboard.cut
                            ? "" : qsTr("Ver em Jobs")
        pendingMessage: root.projectTree.fileClipboard.cut
                        ? qsTr("Movendo arquivo...") : qsTr("Copiando arquivo...")
        sourceDisplayPath: root.shellController.relativeToRoot(
                               root.projectTree.fileClipboard.source)
        entryDisplayPath: root.shellController.relativeToRoot(
                              root.projectTree.fileClipboard.destinationDirectory)
        errorText: root.projectTree.fileClipboard.errorText
        maxAvailableWidth: root.hostWidth - 4 * Theme.spacingMedium
        onConfirmRequested: root.projectTree.fileClipboard.confirm(pasteDialog.currentName())
        onCancelRequested: {
            if (root.projectTree.fileClipboard.pending) return;
            root.projectTree.fileClipboard.clear();
            root.projectTree.focusTreeRequested();
        }
        onPendingDismissRequested: {
            root.projectTree.fileClipboard.dialogVisible = false;
            root.shellController.showTab("jobs");
        }
    }

    Connections {
        target: root.projectTree.fileClipboard
        function onPasteOpened(name) { pasteDialog.openWithName(name); }
    }

    ProjectTransferBatchDialog {
        anchors.fill: parent
        visible: root.projectTree.fileClipboard.batchDialogVisible
        z: 104
        entries: root.projectTree.fileClipboard.batchEntries
        transferController: root.projectTree.fileClipboard
        cut: root.projectTree.fileClipboard.batchCut
        importing: root.projectTree.fileClipboard.batchImport
        pending: root.projectTree.fileClipboard.batchPending
        destinationDisplayPath: root.shellController.relativeToRoot(
                                    root.projectTree.fileClipboard.batchDestinationDirectory)
        errorText: root.projectTree.fileClipboard.batchErrorText
        maxAvailableWidth: root.hostWidth - 4 * Theme.spacingMedium
        onConfirmRequested: function(entries) {
            root.projectTree.fileClipboard.confirmBatch(entries);
        }
        onCancelRequested: {
            root.projectTree.fileClipboard.clear();
            root.projectTree.focusTreeRequested();
        }
        onPendingDismissRequested: {
            root.projectTree.fileClipboard.batchDialogVisible = false;
            root.shellController.showTab("jobs");
        }
    }

    ProjectEntryDeleteDialog {
        anchors.fill: parent
        visible: root.projectTree.entryDeleteVisible
        z: 102
        entryKind: root.projectTree.entryDeleteKind
        entryName: root.projectTree.entryDeleteName
        errorText: root.projectTree.entryDeleteError
        pending: root.projectTree.entryDeletePending
        maxAvailableWidth: root.hostWidth - 4 * Theme.spacingMedium
        onConfirmRequested: root.projectTree.confirmEntryTrash()
        onPermanentRequested: root.projectTree.confirmEntryDelete()
        onCancelRequested: {
            if (root.projectTree.entryDeletePending) return;
            root.projectTree.entryDeleteVisible = false;
            root.projectTree.focusTreeRequested();
        }
    }
}
