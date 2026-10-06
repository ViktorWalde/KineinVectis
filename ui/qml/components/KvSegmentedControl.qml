pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// ESCOLHA UMA ENTRE POUCAS (2026-10-04, padrao novo de frontend): uma trilha
// com a opcao escolhida destacada — no lugar das fileiras de `KvToggleChip`
// em fonte mono, que o autor apontou como legado no dialogo do Banco.
//
//   ┌──────────────────────────────────────────────┐
//   │ [▣ PostgreSQL]   ▤ SQLite    ▥ MongoDB       │   a escolhida: fundo e
//   └──────────────────────────────────────────────┘   texto ambar, icone aceso
//
// `options`: [{ value, label, icon?, tooltip?, enabled? }] (`enabled: false`
// esmaece a opcao e ela nao responde). As opcoes dividem a largura
// por igual (com `fill`) ou medem o rotulo. Teclado: setas trocam a escolha
// quando a trilha tem o foco (Tab chega nela).
FocusScope {
    id: root

    property var options: []
    property string current: ""
    property bool fill: true

    signal selected(string value)

    implicitHeight: 32
    implicitWidth: row.implicitWidth + 2 * root.inset
    activeFocusOnTab: true

    readonly property int inset: 3

    function indexOf(value) {
        for (let i = 0; i < root.options.length; i++) {
            if (root.options[i].value === value) return i;
        }
        return -1;
    }

    function step(delta) {
        if (!root.enabled || root.options.length === 0) return;
        const at = Math.max(0, root.indexOf(root.current));
        // Pula as desligadas; se nenhuma serve, fica onde esta'.
        for (let i = 1; i <= root.options.length; i++) {
            const next = (at + delta * i + root.options.length * i) % root.options.length;
            if (root.options[next].enabled !== false) {
                root.selected(root.options[next].value);
                return;
            }
        }
    }

    Keys.onLeftPressed: root.step(-1)
    Keys.onRightPressed: root.step(1)

    Accessible.role: Accessible.PageTabList

    Rectangle {
        anchors.fill: parent
        radius: Theme.radius
        color: Theme.background0
        border.width: 1
        border.color: root.activeFocus ? Theme.accentDim : Theme.borderSoft

        Behavior on border.color {
            ColorAnimation { duration: Theme.motionFast }
        }
    }

    Row {
        id: row

        anchors.fill: parent
        anchors.margins: root.inset
        spacing: 2

        Repeater {
            model: root.options

            delegate: Rectangle {
                id: segment

                required property var modelData
                required property int index

                readonly property bool chosen: segment.modelData.value === root.current
                readonly property bool usable: segment.modelData.enabled !== false

                width: root.fill && root.options.length > 0
                       ? (row.width - (root.options.length - 1) * row.spacing) / root.options.length
                       : content.implicitWidth + 2 * Theme.spacingMedium
                height: row.height
                radius: Theme.radius - 1
                color: segment.chosen ? Theme.frameAccentTint
                                      : (area.containsMouse ? Theme.surface2 : "transparent")
                border.width: segment.chosen ? 1 : 0
                border.color: Theme.accentDim
                Accessible.role: Accessible.PageTab
                Accessible.name: segment.modelData.label
                Accessible.selected: segment.chosen
                Accessible.onPressAction: {
                    if (segment.usable && root.enabled) root.selected(segment.modelData.value);
                }
                opacity: segment.usable ? 1 : 0.45

                Behavior on color {
                    ColorAnimation { duration: Theme.motionFast }
                }

                Row {
                    id: content

                    anchors.centerIn: parent
                    spacing: Theme.spacingXSmall

                    KvIcon {
                        anchors.verticalCenter: parent.verticalCenter
                        visible: (segment.modelData.icon || "") !== ""
                        name: segment.modelData.icon || "file"
                        size: 14
                        active: segment.chosen || area.containsMouse
                    }

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        text: segment.modelData.label
                        color: segment.chosen ? Theme.accent
                                              : (area.containsMouse ? Theme.textPrimary : Theme.textSecondary)
                        font.pixelSize: Theme.fontSizeSmall
                        font.weight: segment.chosen ? Font.DemiBold : Font.Normal
                        elide: Text.ElideRight
                    }
                }

                MouseArea {
                    id: area

                    anchors.fill: parent
                    hoverEnabled: true
                    enabled: segment.usable
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        root.forceActiveFocus();
                        if (!segment.chosen) root.selected(segment.modelData.value);
                    }
                    onContainsMouseChanged: {
                        const tip = segment.modelData.tooltip || "";
                        if (tip === "") return;
                        if (containsMouse) TooltipController.showFor(segment, tip, "bottom");
                        else TooltipController.hideFor(segment);
                    }
                }
            }
        }
    }
}
