pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// UMA LINHA dos projetos recentes (saiu do RecentWorkspacesCard em
// 2026-10-03, quando ganhou interacao): a placa do projeto (ambar se fixado),
// nome e caminho, "ha' 2 h" ou "Enter ↵" quando destacada, e fixar/remover
// com o mouse. Pairar DESTACA (o Enter abre o que se esta' vendo); a linha
// acende com transicao curta e a barra ambar a esquerda.
Rectangle {
    id: root

    required property var modelData
    required property int index
    property var controller: null
    // "agora", "há 5 min", "há 2 h", "ontem", "há 3 dias", ou a data.
    function since(epochMs) {
        if (epochMs === undefined || epochMs === null || epochMs <= 0) return "";
        const minutes = Math.floor((Date.now() - epochMs) / 60000);
        if (minutes < 1) return qsTr("agora");
        if (minutes < 60) return qsTr("há %1 min").arg(minutes);
        const hours = Math.floor(minutes / 60);
        if (hours < 24) return qsTr("há %1 h").arg(hours);
        const days = Math.floor(hours / 24);
        if (days === 1) return qsTr("ontem");
        if (days < 30) return qsTr("há %1 dias").arg(days);
        return Qt.formatDate(new Date(epochMs), "dd/MM/yyyy");
    }

    readonly property bool available: root.modelData.available
    readonly property bool highlighted: root.index === root.controller.highlightedIndex
    readonly property bool lit: root.highlighted || rowHover.hovered

    height: 46
    radius: Theme.radius
    color: openArea.pressed ? Theme.surfaceSelected
           : (root.lit && root.available ? Theme.surface2 : "transparent")
    opacity: root.available ? 1.0 : 0.72

    Behavior on color {
        ColorAnimation { duration: Theme.motionFast }
    }

    // Pairar destaca: o Enter abre o que se esta' vendo.
    HoverHandler {
        id: rowHover

        onHoveredChanged: if (hovered && root.available) root.controller.highlightedIndex = root.index
    }

    // A barra ambar do destaque.
    Rectangle {
        anchors.left: parent.left
        anchors.verticalCenter: parent.verticalCenter
        width: 3
        height: root.lit && root.available ? parent.height - 14 : 0
        radius: width / 2
        color: Theme.accent

        Behavior on height {
            NumberAnimation { duration: Theme.motionFast }
        }
    }

    // A placa do projeto; fixado, em ambar.
    Rectangle {
        id: tile

        anchors.left: parent.left
        anchors.leftMargin: Theme.spacingSmall + 4
        anchors.verticalCenter: parent.verticalCenter
        width: 30
        height: 30
        radius: Theme.radius
        color: Theme.background0
        border.width: 1
        border.color: root.modelData.pinned ? Theme.accent : Theme.borderSoft

        KvIcon {
            anchors.centerIn: parent
            name: "project"
            size: 16
            disabled: !root.available
            active: root.modelData.pinned || root.lit
        }
    }

    Column {
        anchors.left: tile.right
        anchors.leftMargin: Theme.spacingSmall + 2
        anchors.right: trailing.left
        anchors.rightMargin: Theme.spacingSmall
        anchors.verticalCenter: parent.verticalCenter
        spacing: 1

        Text {
            width: parent.width
            text: root.modelData.name
                  + (root.available ? "" : qsTr(" — caminho ausente"))
            color: root.available ? Theme.textPrimary : Theme.warningSoft
            font.pixelSize: Theme.fontSizeSmall
            font.weight: Font.DemiBold
            elide: Text.ElideRight
        }

        Text {
            width: parent.width
            text: root.modelData.root
            color: Theme.textMuted
            font.family: Theme.monoFont
            font.pixelSize: Theme.fontSizeMicro
            elide: Text.ElideMiddle
        }
    }

    Row {
        id: trailing

        anchors.right: parent.right
        anchors.rightMargin: Theme.spacingXSmall
        anchors.verticalCenter: parent.verticalCenter
        spacing: 2

        // Destacado: a tecla que abre; senao, quando foi aberto.
        Text {
            anchors.verticalCenter: parent.verticalCenter
            text: root.highlighted && root.available ? qsTr("Enter ↵")
                                                                     : root.since(root.modelData.lastOpenedAt)
            color: root.highlighted ? Theme.accent : Theme.textMuted
            font.pixelSize: Theme.fontSizeCaption
            rightPadding: Theme.spacingSmall
        }

        KvIconButton {
            anchors.verticalCenter: parent.verticalCenter
            visible: root.lit || root.modelData.pinned
            compact: true
            iconName: "pin"
            iconSize: 14
            active: root.modelData.pinned
            tooltip: root.modelData.pinned ? qsTr("Soltar (deixa de ficar no topo)")
                                                   : qsTr("Fixar no topo")
            onClicked: root.controller.togglePinned(root.modelData.root)
        }

        KvIconButton {
            anchors.verticalCenter: parent.verticalCenter
            opacity: root.lit ? 1 : 0
            enabled: root.lit
            compact: true
            iconName: "close"
            iconSize: 14
            tooltip: qsTr("Remover dos projetos recentes (a pasta não é tocada)")
            onClicked: root.controller.removeWorkspace(root.modelData.root)
        }
    }

    MouseArea {
        id: openArea

        anchors.left: parent.left
        anchors.right: trailing.left
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        enabled: root.available
        cursorShape: enabled ? Qt.PointingHandCursor : Qt.ArrowCursor
        onClicked: root.controller.openWorkspace(root.modelData.root)
    }
}
