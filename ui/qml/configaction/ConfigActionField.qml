pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Sugestões do core são selecionáveis; valores adicionais continuam livres.
Item {
    id: root
    property var param: null
    property string value: ""
    readonly property var suggestions: param && param.suggestions ? param.suggestions : []
    signal edited(string text)
    implicitHeight: content.implicitHeight
    Column {
        id: content
        anchors.left: parent.left
        anchors.right: parent.right
        spacing: 2
        Row {
            width: parent.width
            spacing: Theme.spacingSmall
            Text {
                width: 110
                topPadding: 6
                text: root.param ? root.param.label + (root.param.required ? " *" : "") : ""
                color: Theme.textSecondary
                font.pixelSize: Theme.fontSizeSmall
                elide: Text.ElideRight
            }
            Column {
                width: parent.width - 110 - Theme.spacingSmall
                KvSelect {
                    width: parent.width
                    visible: root.suggestions.length > 0
                    choices: root.suggestions
                    value: root.value
                    emptyLabel: root.param && root.param.required ? qsTr("Selecione…") : qsTr("Padrão da ação")
                    accessibleLabel: root.param ? root.param.label : ""
                    customPlaceholder: root.param && root.param.placeholder ? root.param.placeholder : ""
                    onEdited: value => root.edited(value)
                }
                KvTextField {
                    width: parent.width
                    height: 30
                    visible: root.suggestions.length === 0
                    placeholder: root.param && root.param.placeholder ? root.param.placeholder : ""
                    text: root.value
                    onEdited: text => root.edited(text)
                }
            }
        }
        Text {
            x: 110 + Theme.spacingSmall
            width: parent.width - x
            visible: text !== ""
            wrapMode: Text.WordWrap
            text: root.param && root.param.description ? root.param.description : ""
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeCaption
        }
    }
}
