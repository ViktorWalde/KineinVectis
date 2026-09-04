import QtQuick
import KineinVectis

// Botao quadrado de LIGA/DESLIGA com rotulo curto ("Aa", "W", ".*").
//
// POR QUE ESTE ARQUIVO EXISTE (2026-09-03). Era um `component ToggleButton`
// declarado DENTRO do EditorFindBar. Desenho de botao nao e' responsabilidade
// da barra de busca, e enquanto ficou la' nenhuma outra tela pode reusa-lo — a
// definicao inline e' visivel so' de dentro do arquivo que a declara.
//
// Ativo e' mostrado por TRES sinais ao mesmo tempo (fundo, borda e peso do
// texto) de proposito: so' cor nao sobrevive a daltonismo nem a tema claro.
Rectangle {
    id: root

    property string labelText: ""
    property bool active: false
    property string tooltip: ""

    signal toggled()

    width: 22
    height: 22
    radius: Theme.radiusXSmall
    color: active ? Theme.surfaceSelected : "transparent"
    border.color: active ? Theme.accent : "transparent"
    border.width: 1

    Text {
        anchors.centerIn: parent
        text: root.labelText
        color: root.active ? Theme.accent : Theme.textMuted
        font.family: Theme.monoFont
        font.pixelSize: 11
        font.bold: root.active
    }

    MouseArea {
        anchors.fill: parent
        cursorShape: Qt.PointingHandCursor
        hoverEnabled: true
        onClicked: root.toggled()
    }
}
