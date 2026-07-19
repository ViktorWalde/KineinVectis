pragma ComponentBehavior: Bound
import QtQuick

// Trilha do arquivo atual ("src › lsp › manager.rs"), extraida do EditorPane
// (C4). Visual puro: recebe o caminho relativo pronto, nao calcula nada.
Row {
    id: root

    property string path: ""

    height: visible ? 18 : 0
    spacing: Theme.spacingXSmall

    Repeater {
        model: root.path.split("/")

        delegate: Row {
            id: breadcrumbSegment

            required property int index
            required property string modelData

            spacing: Theme.spacingXSmall

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: breadcrumbSegment.modelData
                color: breadcrumbSegment.index
                       === root.path.split("/").length - 1
                       ? Theme.textSecondary : Theme.textMuted
                font.pixelSize: 11
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                visible: breadcrumbSegment.index
                         < root.path.split("/").length - 1
                text: "›"
                color: Theme.textMuted
                font.pixelSize: 11
            }
        }
    }
}
