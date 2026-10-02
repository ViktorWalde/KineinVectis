import QtQuick
import KineinVectis

// O CONTEXTO EFETIVO no cabecalho (0.3.8 F3, 53 §5.7): a toolchain que o
// build vai usar, ou o Python do projeto, ao lado do projeto e do git — antes
// moravam no rodape, longe de quem decide. O clique abre o DONO, para baixo;
// nada se edita no chip. Sem espaco, encolhe com reticencias e, abaixo do
// minimo legivel, sai (o editor nunca encolhe por um chip).
Rectangle {
    id: root

    property string summary: ""
    property string iconName: "cpu"
    property string tooltipText: ""
    property bool menuOpen: false
    // A largura que sobra na barra para este chip (quem o poe decide).
    property real availableWidth: 0

    signal menuRequested(real menuX, real menuY)

    // Icone, texto e seta, sem ler a largura da linha (que depende desta).
    readonly property real chromeWidth: 16 + 12 + 2 * Theme.spacingSmall + 2 * Theme.spacingMedium
    readonly property real naturalWidth: summaryText.implicitWidth + root.chromeWidth
    readonly property real minimumWidth: 120

    visible: root.summary !== "" && root.availableWidth >= root.minimumWidth
    width: Math.min(root.naturalWidth, root.availableWidth)
    height: 32
    radius: Theme.radius
    color: root.menuOpen ? Theme.surfaceSelected
                         : (area.containsMouse ? Theme.surface2 : "transparent")
    border.color: area.containsMouse || root.menuOpen ? Theme.borderSoft : "transparent"
    border.width: 1

    Behavior on color {
        ColorAnimation { duration: Theme.motionFast }
    }

    Row {
        id: content

        anchors.verticalCenter: parent.verticalCenter
        anchors.left: parent.left
        anchors.leftMargin: Theme.spacingMedium
        spacing: Theme.spacingSmall

        KvIcon {
            anchors.verticalCenter: parent.verticalCenter
            name: root.iconName
            size: 16
            iconColor: Theme.textSecondary
        }

        Text {
            id: summaryText

            anchors.verticalCenter: parent.verticalCenter
            width: Math.min(implicitWidth, root.width - root.chromeWidth)
            text: root.summary
            color: Theme.textPrimary
            font.pixelSize: Theme.fontSizeBody
            elide: Text.ElideRight
        }

        KvIcon {
            anchors.verticalCenter: parent.verticalCenter
            name: root.menuOpen ? "chevron-up" : "chevron-down"
            size: 12
            iconColor: Theme.textMuted
        }
    }

    MouseArea {
        id: area

        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: {
            TooltipController.hideFor(root);
            root.menuRequested(0, root.height + Theme.spacingXSmall);
        }
        onContainsMouseChanged: {
            if (containsMouse) {
                TooltipController.showFor(root, root.tooltipText, "bottom");
            } else {
                TooltipController.hideFor(root);
            }
        }
    }
}
