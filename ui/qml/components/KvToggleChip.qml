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
//
// O TAMANHO E' DELE, e isto foi corrigido em 2026-09-04. Ele nasceu quadrado
// (22x22) para rotulo de UM caractere ("Aa", ".*"), e quando passou a receber
// palavra — nome de alvo, "Rust · Cargo", "PRIVATE" — cada chamador
// recalculava a largura com um `TextMetrics` proprio. Eram TRES copias da
// mesma conta, e o autor sentiu o resultado: "o dimensionamento precisa de
// polimento".
//
// Agora o chip mede o proprio rotulo. Quem quiser um tamanho diferente ainda
// pode fixar `width`, porque `implicitWidth` so' vale quando ninguem manda.
Rectangle {
    id: root

    property string labelText: ""
    property bool active: false
    property string tooltip: ""

    signal toggled()

    // Minimo de 22 para o chip de um caractere continuar quadrado.
    implicitWidth: Math.max(22, rotulo.implicitWidth + 2 * Theme.spacingMedium)
    implicitHeight: 22
    width: implicitWidth
    height: implicitHeight
    radius: Theme.radiusXSmall
    color: active ? Theme.surfaceSelected : "transparent"
    border.color: active ? Theme.accent : "transparent"
    border.width: 1

    Text {
        id: rotulo

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
