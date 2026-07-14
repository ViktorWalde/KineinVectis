import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "IconRegistry.js" as IconRegistry

Item {
    id: root
    property color backgroundColor: "#0B0D10"
    property color panelColor: "#111418"
    property color borderColor: "#252A32"
    property color textColor: "#E7E2D8"
    property color secondaryTextColor: "#AAA39A"
    property color iconColor: "#AAA39A"
    property color activeColor: "#FFB000"
    property int previewSize: 20
    property string filterText: ""

    Rectangle {
        anchors.fill: parent
        color: root.backgroundColor
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 16
        spacing: 12

        RowLayout {
            Layout.fillWidth: true
            spacing: 12

            Label {
                text: "Kinein Vectis — Icon Gallery"
                color: root.textColor
                font.pixelSize: 17
                font.weight: Font.DemiBold
            }

            Item { Layout.fillWidth: true }

            ComboBox {
                id: sizeCombo
                model: [16, 20, 24, 32]
                currentIndex: 1
                onCurrentValueChanged: root.previewSize = currentValue
            }

            TextField {
                id: filter
                placeholderText: "Filtrar por ID ou nome"
                Layout.preferredWidth: 260
                onTextChanged: root.filterText = text.toLowerCase()
            }
        }

        GridView {
            id: grid
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            cellWidth: 205
            cellHeight: 78
            model: IconRegistry.catalog

            delegate: Rectangle {
                required property var modelData
                width: grid.cellWidth - 8
                height: grid.cellHeight - 8
                radius: 6
                color: hover.hovered ? "#1A1F27" : root.panelColor
                border.color: hover.hovered ? "#343A44" : root.borderColor
                visible: {
                    const q = root.filterText
                    return q.length === 0 ||
                        modelData.id.toLowerCase().includes(q) ||
                        modelData.label.toLowerCase().includes(q)
                }

                HoverHandler { id: hover }

                Row {
                    anchors.fill: parent
                    anchors.margins: 11
                    spacing: 12

                    Rectangle {
                        width: 42
                        height: 42
                        radius: 6
                        color: "#171B21"
                        border.color: "#252A32"

                        KIcon {
                            anchors.centerIn: parent
                            iconId: modelData.id
                            iconSize: root.previewSize
                            color: hover.hovered ? root.activeColor : root.iconColor
                        }
                    }

                    Column {
                        width: parent.width - 60
                        anchors.verticalCenter: parent.verticalCenter
                        spacing: 3

                        Text {
                            width: parent.width
                            text: modelData.label
                            color: root.textColor
                            font.pixelSize: 12
                            font.weight: Font.Medium
                            elide: Text.ElideRight
                        }

                        Text {
                            width: parent.width
                            text: modelData.id
                            color: root.secondaryTextColor
                            font.family: "monospace"
                            font.pixelSize: 9
                            elide: Text.ElideMiddle
                        }

                        Text {
                            width: parent.width
                            text: modelData.category
                            color: "#737A84"
                            font.pixelSize: 9
                            elide: Text.ElideRight
                        }
                    }
                }
            }

            ScrollBar.vertical: ScrollBar {}
        }
    }
}
