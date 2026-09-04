import QtQuick
import KineinVectis

// T6: o balao com a mensagem do diagnostico apontado pelo mouse na sarjeta.
//
// POR QUE ESTE ARQUIVO EXISTE (2026-09-03). Sao 30 linhas de popup — geometria
// propria, clamp de largura para nao vazar da janela e uma armadilha de binding
// circular ja documentada — que moravam no fim do EditorTextSurface.qml, um
// arquivo que ja fazia sarjeta, rolagem, texto e barra. O balao e' area visual
// propria e sai inteiro.
//
// A UI da IDE nao usa QtQuick.Controls; este e' um popup proprio, leve.
//
// O POSICIONAMENTO fica no pai (x, lineTop e maxAvailableWidth chegam prontos),
// mesmo padrao do corte do GitPanel (docs/roadmaps/38 §2.2).
Rectangle {
    id: root

    // Vazio = nada apontado; o balao some.
    property string message: ""
    // Topo da linha apontada, em coordenadas do PAI, ja descontada a rolagem.
    property real lineTop: 0
    property real lineHeight: 0
    // Espaco livre a direita do balao; o texto e' cortado nele.
    property real maxAvailableWidth: 420

    readonly property real maxTextWidth: Math.min(420, maxAvailableWidth)

    visible: message !== ""
    z: 30
    y: Math.max(Theme.spacingSmall, lineTop + lineHeight)
    width: tooltipText.width + 2 * Theme.spacingSmall
    height: tooltipText.height + 2 * Theme.spacingSmall
    radius: Theme.radius
    color: Theme.background2
    border.color: Theme.borderStrong
    border.width: 1

    Text {
        id: tooltipText

        x: Theme.spacingSmall
        y: Theme.spacingSmall
        // implicitWidth (não embrulhado) é constante para o texto; cortar no
        // máximo evita o binding circular do wrap.
        width: Math.min(implicitWidth, root.maxTextWidth)
        text: root.message
        color: Theme.textPrimary
        font.pixelSize: Theme.fontSizeEditor - 2
        wrapMode: Text.WordWrap
    }
}
