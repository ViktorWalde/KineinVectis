pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O menu de contexto da arvore do projeto.
//
// DESDE 2026-10-03 e' o MESMO menu da barra e do terminal (AppMenuPopup):
// grupos com separador, icone, atalho a direita e a barra ambar no item da
// vez. Antes eram 13 linhas desenhadas a mao numa cor so', com um teclado
// proprio (ProjectEntryMenuKeyboard) e um pedaco em outro arquivo
// (ProjectEntryQuickActions) — o autor rejeita lista plana numa cor, e tres
// menus com tres teclados eram tres jeitos de divergir.
//
// O que nao se aplica ao item some (executar/depurar de quem nao e' script);
// o que se aplica mas nao agora fica APAGADO e diz por que ("selecione 1
// item").
Item {
    id: root

    property real menuX: 0
    property real menuY: 0
    property bool runnableScript: false
    property bool debuggableScript: false
    property int selectionCount: 1
    property bool pasteAvailable: false

    signal dismissRequested()
    signal createFileRequested()
    signal createDirectoryRequested()
    signal runScriptRequested()
    signal debugScriptRequested()
    signal renameRequested()
    signal deleteRequested()
    signal copyRequested()
    signal cutRequested()
    signal pasteRequested()
    signal copyAbsolutePathRequested()
    signal copyRelativePathRequested()
    signal openFolderRequested()
    signal openTerminalRequested()

    function entry(label, action, icon, enabled, shortcut) {
        return { label: label, action: action, icon: icon, enabled: enabled, shortcut: shortcut || "" };
    }

    function entries() {
        const single = root.selectionCount <= 1;
        const files = root.selectionCount >= 1 && root.selectionCount <= 128;
        const groups = [
            [root.entry(qsTr("Adicionar arquivo"), "create.file", "file", true),
             root.entry(qsTr("Adicionar pasta"), "create.directory", "folder", true)],
            [],
            [root.entry(single ? qsTr("Renomear") : qsTr("Renomear: selecione 1 item"), "rename", "rename", single),
             root.entry(single ? qsTr("Excluir") : qsTr("Excluir: selecione 1 item"), "delete", "trash", single)],
            [root.entry(qsTr("Copiar"), "copy", "copy", files, "Ctrl+C"),
             root.entry(qsTr("Recortar"), "cut", "cut", files, "Ctrl+X"),
             root.entry(root.pasteAvailable ? qsTr("Colar") : qsTr("Colar: itens do projeto"), "paste", "paste",
                        root.pasteAvailable, "Ctrl+V")],
            [root.entry(qsTr("Copiar caminho absoluto"), "path.absolute", "copy", files),
             root.entry(qsTr("Copiar caminho relativo"), "path.relative", "copy", files)],
            [root.entry(qsTr("Abrir pasta no gerenciador"), "open.folder", "external", single),
             root.entry(qsTr("Abrir terminal nesta pasta"), "open.terminal", "terminal", single)]
        ];
        if (root.runnableScript) groups[1].push(root.entry(qsTr("Executar script"), "run", "run", true));
        if (root.debuggableScript) groups[1].push(root.entry(qsTr("Depurar"), "debug", "debug", true));
        const out = [];
        for (const group of groups) {
            if (group.length === 0) continue;
            if (out.length > 0) out.push({ separator: true, label: "", action: "", enabled: false });
            for (const item of group) out.push(item);
        }
        return out;
    }

    function dispatch(action) {
        switch (action) {
        case "create.file": root.createFileRequested(); break;
        case "create.directory": root.createDirectoryRequested(); break;
        case "run": root.runScriptRequested(); break;
        case "debug": root.debugScriptRequested(); break;
        case "rename": root.renameRequested(); break;
        case "delete": root.deleteRequested(); break;
        case "copy": root.copyRequested(); break;
        case "cut": root.cutRequested(); break;
        case "paste": root.pasteRequested(); break;
        case "path.absolute": root.copyAbsolutePathRequested(); break;
        case "path.relative": root.copyRelativePathRequested(); break;
        case "open.folder": root.openFolderRequested(); break;
        case "open.terminal": root.openTerminalRequested(); break;
        }
    }

    AppMenuPopup {
        anchors.fill: parent
        visible: root.visible
        menuX: root.menuX
        menuY: root.menuY
        menuWidth: 260
        items: root.entries()
        onDismissRequested: root.dismissRequested()
        onActionRequested: function(action) { root.dispatch(action); }
    }
}
