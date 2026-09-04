import QtQuick
import KineinVectis

// A alca vertical que reabre o painel Estrutura depois de recolhido.
//
// POR QUE ESTE ARQUIVO EXISTE (2026-09-03). Sao 50 linhas de widget — retangulo
// com hover, icone, rotulo girado 90 graus, area de mouse e tooltip — que
// moravam soltas no fim do EditorPane. Nada disso e' composicao: e' desenho, e
// desenho tem dono proprio.
//
// A alca so' aparece quando HA estrutura para mostrar e o painel esta
// recolhido; quem decide isso e' o pai, por `visible`.
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
        text: qsTr("Estrutura")
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
                TooltipController.showFor(root, qsTr("Expandir Estrutura"),
                                          "bottom");
            } else {
                TooltipController.hideFor(root);
            }
        }
    }
}
