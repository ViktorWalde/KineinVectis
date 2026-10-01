import QtQuick

// As PARTES da SymbolResultsList que o ListView instancia sozinho (secao e
// cabecalho). SEM `pragma ComponentBehavior: Bound` de proposito: no Qt 6.4 do
// AppImage, essas partes declaradas num arquivo Bound nunca sao criadas
// ("Component is not ready"; ver GitChangesListParts.qml e roadmap 53
// §5.2.1). Os tipos acham a lista por `ListView.view`.
QtObject {
    readonly property Component section: Component {
        SymbolResultsSection {}
    }
    readonly property Component header: Component {
        SymbolResultsSearching {}
    }
}
