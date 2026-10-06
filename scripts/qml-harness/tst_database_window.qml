pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Composição real: fechar/restaurar foco não pode cancelar a ação do menu.
Item {
    id: root
    width: 640
    height: 480
    property int stage: 0
    property int failures: 0
    property var view: null
    property var popup: null
    property string action: ""
    property bool focusedBeforeAction: false

    Item { id: dialogFocus }
    DataSourceController {
        id: bankController
        workspaceRoot: "/projeto"
    }
    DatabaseWindow {
        id: bank
        width: 260
        height: 400
        controller: bankController
        menuLayer: root
        onNewRequested: engine => {
            root.focusedBeforeAction = root.view.activeFocus;
            root.action = "new." + engine;
            dialogFocus.forceActiveFocus();
        }
        onEditRequested: name => {
            root.focusedBeforeAction = root.view.activeFocus;
            root.action = "edit." + name;
            dialogFocus.forceActiveFocus();
        }
    }
    function check(condition, label) {
        if (!condition) { root.failures++; console.error("FALHOU: " + label); }
    }

    Timer {
        interval: 50
        running: true
        repeat: true
        onTriggered: {
            if (root.stage === 0) {
                bankController.profiles = [Object.assign(DataSourceKinds.emptyProfile(), { name: "loja" })];
                for (let index = 0; index < root.children.length; index++) {
                    if (typeof root.children[index].restorePreviousFocus === "function") root.popup = root.children[index];
                }
                for (let index = 0; index < bank.children.length; index++) {
                    if (typeof bank.children[index].menuForSelection === "function") root.view = bank.children[index];
                }
                root.check(!!root.popup && !!root.view, "menu fora do recorte da janela");
                bank.focusTree();
                bank.showNewMenu(bank, bank.width, bank.height);
            } else if (root.stage === 1) {
                root.check(root.popup.visible && root.popup.activeFocus, "menu com foco real");
                const frame = root.popup.children[1];
                root.check(frame.x + frame.width > bank.width && frame.x + frame.width <= root.width
                    && frame.y + frame.height <= root.height, "menu usa o workspace e cabe na tela");
                root.popup.activate(2);
                root.check(root.action === "new.mongo" && !root.popup.visible, "motor escolhido chega ao host");
                root.check(root.focusedBeforeAction && dialogFocus.activeFocus, "restaura antes da ação e conserva foco do diálogo");
                bank.focusTree();
                root.view.menuForSelection();
            } else if (root.stage === 2) {
                root.popup.activate(2);
                root.check(root.action === "edit.loja" && root.focusedBeforeAction && dialogFocus.activeFocus,
                    "editar conexão depois de restaurar foco");
                bank.focusTree();
                root.view.menuForSelection();
            } else if (root.stage === 3) {
                root.popup.handleKey({ key: Qt.Key_Escape, accepted: false });
                root.check(!root.popup.visible && root.view.activeFocus, "Escape devolve foco à árvore");
                root.view.menuForSelection();
            } else if (root.stage === 4) {
                bankController.profiles = [];
                root.check(!root.popup.visible && root.view.treeModel.selectedKey === "", "remover perfil fecha menu e limpa seleção");
                if (root.failures) console.error("FALHAS " + root.failures);
                Qt.exit(root.failures === 0 ? 0 : 1);
            }
            root.stage++;
        }
    }
}
