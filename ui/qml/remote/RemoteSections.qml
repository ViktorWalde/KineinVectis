pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// As SECOES do Remoto (fatia R1/V2, 2026-09-24; abas de janela acoplada desde
// 2026-10-04).
//
// A §5.1 da especificacao pede Overview, Workspace, Run & Debug, Sistema e
// Configurar, e diz: "nao devem ser cinco cards longos na mesma rolagem".
// Uma seccao por vez, entao. Na coluna estreita da janela acoplada as cinco
// cabem numa linha: a ultima (Configurar, a rara) e' so' a engrenagem, e a
// linha ambar DESLIZA ate' a aba escolhida.
//
//   Visão  Projeto  Executar  Sistema  ⚙
//          ━━━━━━━
Item {
    id: root

    // [{ id, label, icon, tooltip }] — sem `label`, a aba e' so' o icone (o
    // rotulo vai no tooltip).
    property var sections: []
    property string current: ""

    signal selected(string id)

    implicitHeight: 30

    readonly property Item currentTab: {
        for (let i = 0; i < root.sections.length; i++) {
            if (root.sections[i].id === root.current) return tabs.count > i ? tabs.itemAt(i) : null;
        }
        return null;
    }

    Rectangle {
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        height: 1
        color: Theme.borderSoft
    }

    Row {
        id: row

        height: parent.height

        Repeater {
            id: tabs

            model: root.sections

            delegate: Item {
                id: tab

                required property var modelData

                readonly property string sectionId: tab.modelData.id
                readonly property bool current: tab.sectionId === root.current
                readonly property bool iconOnly: tab.modelData.label === undefined || tab.modelData.label === ""

                width: (tab.iconOnly ? 16 : label.implicitWidth) + 2 * Theme.spacingSmall
                height: row.height
                Accessible.role: Accessible.PageTab
                Accessible.name: tab.iconOnly ? tab.modelData.tooltip : tab.modelData.label

                Rectangle {
                    anchors.fill: parent
                    anchors.bottomMargin: 3
                    radius: Theme.radius
                    color: area.containsMouse && !tab.current ? Theme.surface2 : "transparent"

                    Behavior on color {
                        ColorAnimation { duration: Theme.motionFast }
                    }
                }

                Text {
                    id: label

                    anchors.centerIn: parent
                    anchors.verticalCenterOffset: -1
                    visible: !tab.iconOnly
                    text: tab.modelData.label || ""
                    color: tab.current ? Theme.textPrimary : (area.containsMouse ? Theme.textSecondary : Theme.textMuted)
                    font.pixelSize: Theme.fontSizeSmall
                    font.weight: tab.current ? Font.DemiBold : Font.Normal
                }

                KvIcon {
                    anchors.centerIn: parent
                    anchors.verticalCenterOffset: -1
                    visible: tab.iconOnly
                    name: tab.modelData.icon || "settings"
                    size: 15
                    active: tab.current || area.containsMouse
                }

                MouseArea {
                    id: area

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.selected(tab.sectionId)
                    onContainsMouseChanged: {
                        if (!tab.iconOnly) return;
                        if (containsMouse) TooltipController.showFor(tab, tab.modelData.tooltip, "bottom");
                        else TooltipController.hideFor(tab);
                    }
                }
            }
        }
    }

    // A linha da aba escolhida: desliza e muda de largura.
    Rectangle {
        anchors.bottom: parent.bottom
        height: 2
        radius: height / 2
        color: Theme.accent
        x: root.currentTab !== null ? root.currentTab.x + Theme.spacingXSmall : 0
        width: root.currentTab !== null ? root.currentTab.width - 2 * Theme.spacingXSmall : 0

        Behavior on x {
            NumberAnimation { duration: Theme.motionMedium; easing.type: Theme.easingStandard }
        }

        Behavior on width {
            NumberAnimation { duration: Theme.motionMedium; easing.type: Theme.easingStandard }
        }
    }
}
