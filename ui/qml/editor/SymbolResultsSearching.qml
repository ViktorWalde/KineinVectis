import QtQuick
import KineinVectis

// O "procurando…" no topo da SymbolResultsList enquanto a busca corre.
// Criado pela SymbolResultsListParts; a lista e' `ListView.view`.
Text {
    readonly property var view: ListView.view
    readonly property bool searching: view !== null && view.symbols !== null
                                      && view.symbols.searching === true

    width: view ? view.width : 0
    height: searching ? 22 : 0
    visible: height > 0
    leftPadding: Theme.spacingSmall
    verticalAlignment: Text.AlignVCenter
    text: qsTr("procurando…")
    color: Theme.textMuted
    font.pixelSize: Theme.fontSizeCaption
}
