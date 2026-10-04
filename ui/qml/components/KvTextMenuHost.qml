import QtQuick
import KineinVectis

// Desenha o menu de contexto dos campos de texto (TextMenuController) com o
// AppMenuPopup de toda a IDE: mesmo visual, mesmo teclado, clique fora fecha.
Item {
    id: root

    visible: TextMenuController.open

    AppMenuPopup {
        anchors.fill: parent
        menuX: TextMenuController.menuX
        menuY: TextMenuController.menuY
        menuWidth: 230
        items: TextMenuController.items
        onDismissRequested: TextMenuController.close()
        onActionRequested: function(action) { TextMenuController.run(action); }
    }
}
