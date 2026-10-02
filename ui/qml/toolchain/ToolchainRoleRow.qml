pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Um PAPEL da toolchain (0.3.8 F3): recolhido, uma linha — o nome do papel a
// esquerda, o que o build vai usar a direita (com "automático" quando quem
// escolheu foi o core). Aberto, as opcoes: "Automático (PATH)" primeiro, que
// e' o padrao e sair de uma escolha e' tao facil quanto entrar, e depois so'
// os candidatos DETECTADOS nesta maquina.
Column {
    id: root

    property var controller
    property string roleKey: ""
    property string roleLabel: ""
    property bool expanded: false

    signal toggleRequested()

    readonly property var options: root.controller.candidatesFor(root.roleKey)
    readonly property var selection: root.controller.selectionFor(root.roleKey)
    readonly property bool automatic: root.selection === null || root.selection.id === undefined

    spacing: 2

    Rectangle {
        width: root.width
        height: 34
        radius: Theme.radius
        color: root.expanded ? Theme.surfaceSelected
                             : (headerArea.containsMouse ? Theme.surface2 : "transparent")

        Behavior on color {
            ColorAnimation { duration: Theme.motionFast }
        }

        Text {
            id: roleText

            anchors.left: parent.left
            anchors.leftMargin: Theme.spacingMedium
            anchors.verticalCenter: parent.verticalCenter
            text: root.roleLabel
            color: Theme.textSecondary
            font.pixelSize: Theme.fontSizeBody
        }

        Text {
            anchors.left: roleText.right
            anchors.leftMargin: Theme.spacingMedium
            anchors.right: chevron.left
            anchors.rightMargin: Theme.spacingSmall
            anchors.verticalCenter: parent.verticalCenter
            horizontalAlignment: Text.AlignRight
            text: root.controller.labelFor(root.roleKey)
            color: root.automatic ? Theme.textPrimary : Theme.accent
            font.pixelSize: Theme.fontSizeBody
            font.bold: !root.automatic
            elide: Text.ElideMiddle
        }

        KvIcon {
            id: chevron

            anchors.right: parent.right
            anchors.rightMargin: Theme.spacingMedium
            anchors.verticalCenter: parent.verticalCenter
            name: root.expanded ? "chevron-up" : "chevron-down"
            size: 12
            iconColor: Theme.textMuted
        }

        MouseArea {
            id: headerArea

            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: root.toggleRequested()
        }
    }

    Column {
        visible: root.expanded
        width: root.width
        leftPadding: Theme.spacingLarge
        spacing: 1

        ToolchainOptionRow {
            width: root.width - Theme.spacingLarge
            label: qsTr("Automático (PATH)")
            hint: qsTr("o core escolhe e diz qual")
            chosen: root.automatic
            onClicked: root.controller.choose(root.roleKey, "")
        }

        Repeater {
            model: root.options

            delegate: ToolchainOptionRow {
                required property var modelData

                width: root.width - Theme.spacingLarge
                label: modelData.label
                chosen: !root.automatic && root.selection.id === modelData.id
                onClicked: root.controller.choose(root.roleKey, modelData.id)
            }
        }
    }
}
