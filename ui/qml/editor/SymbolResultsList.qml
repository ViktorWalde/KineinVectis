pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Os resultados da busca de simbolos da aba Simbolos (E3-2): a pasta do
// arquivo aberto primeiro, depois o projeto — duas secoes com o total de
// cada uma; cada linha `nome` / `tipo · caminho:linha`; clique abre. Quem
// busca e' o SymbolsController (`symbols`); aqui so' o desenho.
ListView {
    id: symbolList

    // O SymbolsController dono dos resultados (null = nada a mostrar).
    property var symbols: null

    clip: true
    model: {
        if (!symbolList.symbols) return [];
        const pasta = symbolList.symbols.folderResults;
        const projeto = symbolList.symbols.results;
        const nesta = qsTr("nesta pasta (%1)").arg(pasta.length);
        const tudo = qsTr("no projeto (%1)").arg(projeto.length);
        return pasta.map(s => ({ symbol: s, group: nesta }))
            .concat(projeto.map(s => ({ symbol: s, group: tudo })));
    }

    section.property: "group"
    section.criteria: ViewSection.FullString
    section.delegate: Text {
        required property string section

        width: symbolList.width
        height: 22
        leftPadding: Theme.spacingSmall
        verticalAlignment: Text.AlignVCenter
        text: section
        color: Theme.textMuted
        font.pixelSize: 10
        font.bold: true
    }

    header: Text {
        width: symbolList.width
        height: symbolList.symbols && symbolList.symbols.searching ? 22 : 0
        visible: height > 0
        leftPadding: Theme.spacingSmall
        verticalAlignment: Text.AlignVCenter
        text: qsTr("procurando…")
        color: Theme.textMuted
        font.pixelSize: 10
    }

    delegate: Rectangle {
        id: symbolRow

        required property var modelData

        width: ListView.view.width
        height: 34
        color: symbolMouse.containsMouse ? Theme.surface2 : "transparent"

        Text {
            x: Theme.spacingSmall
            y: 3
            width: parent.width - 2 * Theme.spacingSmall
            text: symbolRow.modelData.symbol.name
            color: Theme.textPrimary
            font.pixelSize: Theme.fontSizeTree
            elide: Text.ElideRight
        }

        Text {
            x: Theme.spacingSmall
            y: 19
            width: parent.width - 2 * Theme.spacingSmall
            // A FONTE so' aparece quando as DUAS estao na lista (§11.1 do
            // roadmap 48: "mantem a fonte visivel quando isso ajudar a explicar
            // divergencia"). Com uma fonte so', dizer de onde veio nao explica
            // nada e rouba espaco do caminho.
            text: symbolRow.modelData.symbol.kind + " · " + symbolRow.modelData.symbol.path
                  + ":" + symbolRow.modelData.symbol.line
                  + (symbolList.symbols && symbolList.symbols.showSource
                     ? " · " + symbolRow.modelData.symbol.source : "")
            color: Theme.textMuted
            font.family: Theme.monoFont
            font.pixelSize: 9
            elide: Text.ElideMiddle
        }

        MouseArea {
            id: symbolMouse

            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: symbolList.symbols.open(symbolRow.modelData.symbol)
        }
    }

    Text {
        anchors.centerIn: parent
        visible: symbolList.symbols && symbolList.symbols.active && !symbolList.symbols.searching && symbolList.count === 0
        text: symbolList.symbols && symbolList.symbols.indexState !== "" && symbolList.symbols.indexState !== "ready"
              ? qsTr("o índice ainda está lendo o projeto") : qsTr("nenhum símbolo com esse nome")
        color: Theme.textMuted
        font.pixelSize: 11
    }
}
