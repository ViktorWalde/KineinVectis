import QtQuick
import KineinVectis

// O menu de contexto da arvore (2026-10-03: o mesmo AppMenuPopup da barra).
//
// O que se prova: o que nao se aplica SOME (executar/depurar de quem nao e'
// script) sem deixar separador dobrado; o que nao vale agora fica APAGADO e
// diz por que; cada acao chega ao sinal certo; e o teclado do menu pula o
// apagado e o separador (o AppMenuPopup e' o dono dele).
Item {
    id: root

    width: 640
    height: 480

    property string fired: ""

    ProjectEntryContextMenu {
        id: menu

        anchors.fill: parent
        visible: false
        onRenameRequested: root.fired += "rename,"
        onDeleteRequested: root.fired += "delete,"
        onPasteRequested: root.fired += "paste,"
        onRunScriptRequested: root.fired += "run,"
        onOpenTerminalRequested: root.fired += "terminal,"
    }

    function actions(list) {
        return list.map(function(entry) { return entry.separator === true ? "|" : entry.action; }).join(" ");
    }

    Component.onCompleted: {
        let failures = 0;

        // Arquivo comum: sem executar/depurar, e sem separador dobrado.
        let list = menu.entries();
        if (root.actions(list).indexOf("run") >= 0 || root.actions(list).indexOf("| |") >= 0) failures += 1;
        if (list[0].separator === true || list[list.length - 1].separator === true) failures += 2;

        // Script: executar e depurar aparecem, no grupo deles.
        menu.runnableScript = true;
        menu.debuggableScript = true;
        list = menu.entries();
        if (root.actions(list).indexOf("| run debug |") < 0) failures += 4;

        // Dois itens selecionados: renomear e excluir apagados, com o motivo.
        menu.selectionCount = 2;
        const rename = menu.entries().filter(function(entry) { return entry.action === "rename"; })[0];
        if (rename.enabled || rename.label.indexOf("selecione 1") < 0) failures += 8;

        // Colar sem nada copiado: apagado e dizendo o que cola.
        const paste = menu.entries().filter(function(entry) { return entry.action === "paste"; })[0];
        if (paste.enabled || paste.shortcut !== "Ctrl+V" || paste.icon !== "paste") failures += 16;

        // Cada acao chega ao sinal certo.
        ["rename", "delete", "paste", "run", "open.terminal", "nada"].forEach(function(action) { menu.dispatch(action); });
        if (root.fired !== "rename,delete,paste,run,terminal,") failures += 32;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
