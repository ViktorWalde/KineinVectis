pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O que o CLIQUE DIREITO no explorer abre: menu de contexto, renomear, apagar.
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
    // Cancelar um dialogo devolve o foco ao editor; sem isso o teclado fica
    // num overlay que nao existe mais.
    property var editorController: null

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
        onDismissRequested: root.projectTree.entryMenuVisible = false
        onCreateFileRequested: root.projectTree.openEntryCreate("file")
        onCreateDirectoryRequested: root.projectTree.openEntryCreate("directory")
        onRunScriptRequested: root.projectTree.runEntryScript()
        onRenameRequested: root.projectTree.openEntryRename()
        onDeleteRequested: root.projectTree.openEntryDelete()
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
            root.editorController.focusEditor();
        }
    }

    ProjectEntryDeleteDialog {
        anchors.fill: parent
        visible: root.projectTree.entryDeleteVisible
        z: 102
        entryKind: root.projectTree.entryDeleteKind
        entryName: root.projectTree.entryDeleteName
        errorText: root.projectTree.entryDeleteError
        maxAvailableWidth: root.hostWidth - 4 * Theme.spacingMedium
        onConfirmRequested: root.projectTree.confirmEntryDelete()
        onCancelRequested: {
            root.projectTree.entryDeleteVisible = false;
            root.editorController.focusEditor();
        }
    }
}
