import QtQuick
import KineinVectis

// A alca vertical que reabre a aba Simbolos (estrutura + busca) recolhida.
//
// POR QUE ESTE ARQUIVO EXISTE (2026-09-03). Sao 50 linhas de widget — retangulo
// com hover, icone, rotulo girado 90 graus, area de mouse e tooltip — que
// moravam soltas no fim do EditorPane. Nada disso e' composicao: e' desenho, e
// desenho tem dono proprio.
//
// A alca aparece sempre que ha' projeto e o painel esta' recolhido (E3-2:
// a busca no projeto nao depende de arquivo aberto); quem decide e' o pai.
Rectangle {
    id: root

    signal expandRequested()

    width: 28
    height: 96
    radius: Theme.radius
    color: handleMouse.containsMouse ? Theme.surfaceSelected : Theme.surface1
    border.color: handleMouse.containsMouse ? Theme.accent : Theme.borderSoft
    border.width: 1

    KvIcon {
        anchors.top: parent.top
        anchors.topMargin: Theme.spacingSmall
        anchors.horizontalCenter: parent.horizontalCenter
        name: "project"
        size: 16
        active: handleMouse.containsMouse
    }

    Text {
        anchors.centerIn: parent
        anchors.verticalCenterOffset: 12
        text: qsTr("Símbolos")
        rotation: -90
        color: Theme.textSecondary
        font.pixelSize: 10
        font.bold: true
    }

    MouseArea {
        id: handleMouse

        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: root.expandRequested()
        onContainsMouseChanged: {
            if (containsMouse) {
                TooltipController.showFor(root, qsTr("Abrir Símbolos (Alt+7)"),
                                          "bottom");
            } else {
                TooltipController.hideFor(root);
            }
        }
    }
}
