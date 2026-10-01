import QtQuick
import KineinVectis

// O titulo de um grupo da SymbolResultsList ("nesta pasta (n)", "no projeto
// (n)"). Criado pela SymbolResultsListParts; a lista e' `ListView.view`.
Text {
    required property string section

    readonly property var view: ListView.view

    width: view ? view.width : 0
    height: 22
    leftPadding: Theme.spacingSmall
    verticalAlignment: Text.AlignVCenter
    text: section
    color: Theme.textMuted
    font.pixelSize: 10
    font.bold: true
}
