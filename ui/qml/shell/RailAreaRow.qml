import QtQuick
import KineinVectis

// Uma linha do painel de areas do trilho (0.3.7; retorno do autor de
// 2026-10-02). Hierarquia visivel: o icone e o nome (o que e'), o estado em
// palavras e na cor dele (como esta'), e as acoes como botoes de icone com
// estado aceso (o alfinete fixado, o olho riscado quando oculta) — nada de
// lista de frases numa cor so'.
Rectangle {
    id: root

    property var entry: ({})
    property var status: ({ text: "", tone: "normal" })
    property bool pinned: false
    property bool hidden: false
    property bool onRail: false
    property bool highlighted: false

    signal openRequested()
    signal pinToggled()
    signal hideToggled()

    width: parent ? parent.width : 340
    height: 46
    radius: Theme.radius
    color: highlighted ? Theme.surfaceSelected : (rowArea.containsMouse ? Theme.surface2 : "transparent")
    border.color: highlighted ? Theme.accentDim : "transparent"
    border.width: 1

    // Hover e selecao trocam de cor em `motionFast`, nao num salto (0.3.9:
    // fluidez, decisao do autor de 2026-10-01).
    Behavior on color { ColorAnimation { duration: Theme.motionFast; easing.type: Theme.easingStandard } }
    Behavior on border.color { ColorAnimation { duration: Theme.motionFast; easing.type: Theme.easingStandard } }

    MouseArea {
        id: rowArea

        anchors.fill: parent
        hoverEnabled: true
        onClicked: root.openRequested()
    }

    KvIcon {
        id: areaIcon

        anchors.left: parent.left
        anchors.leftMargin: Theme.spacingSmall
        anchors.verticalCenter: parent.verticalCenter
        name: root.entry.icon !== undefined ? root.entry.icon : "file"
        size: 18
        active: root.onRail
    }

    Column {
        anchors.left: areaIcon.right
        anchors.leftMargin: Theme.spacingSmall
        anchors.right: actions.left
        anchors.rightMargin: Theme.spacingSmall
        anchors.verticalCenter: parent.verticalCenter
        spacing: 2

        Text {
            width: parent.width
            text: root.entry.title !== undefined ? root.entry.title : ""
            color: root.hidden ? Theme.textSecondary : Theme.textPrimary
            font.pixelSize: Theme.fontSizeBody
            font.weight: Font.DemiBold
            elide: Text.ElideRight
        }

        Text {
            width: parent.width
            text: root.status.text
            color: root.status.tone === "accent" ? Theme.accent
                   : (root.status.tone === "muted" ? Theme.textMuted : Theme.textSecondary)
            font.pixelSize: Theme.fontSizeCaption
            elide: Text.ElideRight
        }
    }

    Row {
        id: actions

        anchors.right: parent.right
        anchors.rightMargin: Theme.spacingXSmall
        anchors.verticalCenter: parent.verticalCenter
        spacing: Theme.spacingXSmall

        // O atalho, quando ha': o caminho de teclado aparece junto.
        Rectangle {
            anchors.verticalCenter: parent.verticalCenter
            visible: root.entry.shortcut !== undefined && root.entry.shortcut !== ""
            width: shortcutText.implicitWidth + 2 * Theme.spacingXSmall
            height: 18
            radius: Theme.radiusXSmall
            color: "transparent"
            border.color: Theme.borderSoft
            border.width: 1

            Text {
                id: shortcutText

                anchors.centerIn: parent
                text: root.entry.shortcut !== undefined ? root.entry.shortcut : ""
                color: Theme.textMuted
                font.family: Theme.monoFont
                font.pixelSize: Theme.fontSizeMicro
            }
        }

        KvIconButton {
            anchors.verticalCenter: parent.verticalCenter
            // O foco do teclado fica com o painel (Esc fecha), nao com um
            // botao qualquer que acenderia a borda sem ninguem pedir.
            focus: false
            compact: true
            iconName: "pin"
            iconSize: 16
            active: root.pinned
            tooltip: root.pinned ? qsTr("Desafixar do trilho") : qsTr("Fixar no trilho")
            onClicked: root.pinToggled()
        }

        KvIconButton {
            anchors.verticalCenter: parent.verticalCenter
            // O foco do teclado fica com o painel (Esc fecha), nao com um
            // botao qualquer que acenderia a borda sem ninguem pedir.
            focus: false
            compact: true
            iconName: root.hidden ? "eye-off" : "eye"
            iconSize: 16
            active: root.hidden
            tooltip: root.hidden ? qsTr("Mostrar de novo") : qsTr("Ocultar neste projeto")
            onClicked: root.hideToggled()
        }

        KvButton {
            anchors.verticalCenter: parent.verticalCenter
            focus: false
            visible: !root.onRail
            compact: true
            text: qsTr("Abrir")
            enabled: root.entry.available !== false
            onClicked: root.openRequested()
        }
    }
}
