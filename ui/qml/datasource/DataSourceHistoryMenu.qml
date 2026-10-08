pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O MENU do historico, aberto pelo cabecalho do console (passo 13b): as
// ultimas instrucoes da conexao, a mais recente primeiro, com a hora e o
// aviso das que falharam. Escolher insere a instrucao no fim do console pelo
// caminho dos modelos (selecionada, desfazivel, sem executar); limpar pede
// confirmacao no proprio menu. Quem o hospeda decide quando ele aparece.
Item {
    id: root

    property var history: null
    property real menuX: 0
    property real menuY: 0
    property bool confirming: false

    signal dismissRequested()
    signal statementChosen(string sql)

    readonly property string entryPrefix: "history:"

    function entries() {
        if (root.history === null) return [];
        if (root.confirming) {
            return [{ label: qsTr("Apagar o histórico de «%1»").arg(root.history.name), action: "history.clear.confirm",
                      icon: "trash", enabled: true, shortcut: "" },
                    { label: qsTr("Cancelar"), action: "history.clear.cancel", icon: "close", enabled: true, shortcut: "" }];
        }
        const items = [];
        const list = root.history.entries;
        if (root.history.loading) items.push({ label: qsTr("Carregando…"), action: "", icon: "", enabled: false, shortcut: "" });
        else if (root.history.error !== "") items.push({ label: root.history.error, action: "", icon: "warning", enabled: false, shortcut: "" });
        else if (list.length === 0) items.push({ label: qsTr("Nada executado nesta conexão ainda"), action: "", icon: "", enabled: false, shortcut: "" });
        const now = new Date();
        for (let index = 0; index < list.length; index++) {
            items.push({ label: root.history.label(list[index]), action: root.entryPrefix + index,
                         icon: list[index].outcome === "failed" ? "warning" : "", enabled: true,
                         shortcut: root.history.when(list[index], now) });
        }
        if (list.length > 0) {
            items.push({ separator: true, label: "", action: "", enabled: false });
            items.push({ label: qsTr("Limpar histórico…"), action: "history.clear", icon: "trash", enabled: true, shortcut: "" });
        }
        return items;
    }

    function dispatch(action) {
        if (action.indexOf(root.entryPrefix) === 0) {
            const entry = root.history.entries[Number(action.slice(root.entryPrefix.length))];
            root.dismissRequested();
            if (entry) root.statementChosen(entry.sql);
        } else if (action === "history.clear") {
            root.confirming = true;
        } else if (action === "history.clear.confirm") {
            root.history.clear();
            root.confirming = false;
            root.dismissRequested();
        } else if (action === "history.clear.cancel") {
            root.confirming = false;
        }
    }

    onVisibleChanged: if (!visible) root.confirming = false

    AppMenuPopup {
        anchors.fill: parent
        visible: root.visible
        menuX: root.menuX
        menuY: root.menuY
        menuWidth: 420
        items: root.entries()
        onDismissRequested: root.dismissRequested()
        onActionRequested: action => root.dispatch(action)
    }
}
