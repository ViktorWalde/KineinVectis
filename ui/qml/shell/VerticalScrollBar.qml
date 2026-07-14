pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Barra de rolagem vertical PRÓPRIA (B2, docs/24).
//
// Por que existe: o projeto não usa QtQuick.Controls (mesma decisão do tooltip
// e do SettingsDialog), e até 2026-07-12 a IDE não tinha barra de rolagem em
// lugar NENHUM — nem no editor, nem nos painéis, nem no terminal. Sem ela o
// usuário não sabe que há conteúdo fora da tela, onde está no arquivo, nem se
// uma rolagem aconteceu.
//
// É genérica na UNIDADE: serve pra PIXELS (Flickable do editor:
// contentHeight/height/contentY) e pra LINHAS (terminal: total/rows/offset).
// Quem usa converte; aqui só há proporção e arrasto.
//
// O componente NUNCA muda `position` sozinho — ele PEDE por `moveRequested` e o
// dono aplica. Isso mantém uma fonte da verdade só (o Flickable, ou o core no
// caso do terminal, que clampa o offset ao histórico real).
Item {
    id: bar

    // Todos na mesma unidade.
    property real contentSize: 0
    property real viewportSize: 0
    property real position: 0

    signal moveRequested(real position)

    readonly property real maxPosition: Math.max(0, contentSize - viewportSize)
    readonly property bool scrollable: maxPosition > 0

    // Trilho estreito; o polegar engorda no hover (padrão JetBrains).
    implicitWidth: 10
    width: implicitWidth
    visible: scrollable

    Rectangle {
        id: track

        anchors.fill: parent
        color: Theme.surface2
        opacity: thumbMouse.containsMouse || thumbMouse.pressed ? 0.6 : 0
        radius: Theme.radiusXSmall

        Behavior on opacity {
            NumberAnimation { duration: 120 }
        }
    }

    Rectangle {
        id: thumb

        // Piso de 24px: com arquivo enorme o polegar não pode virar um fio
        // impossível de pegar com o mouse.
        readonly property real minHeight: 24
        readonly property real ratio: bar.contentSize > 0
                ? Math.min(1, bar.viewportSize / bar.contentSize) : 1
        readonly property real trackHeight: bar.height
        readonly property real span: Math.max(0, trackHeight - height)

        width: thumbMouse.containsMouse || thumbMouse.pressed
                ? bar.width - 2 : bar.width - 4
        height: Math.max(minHeight, trackHeight * ratio)
        x: (bar.width - width) / 2
        y: bar.maxPosition > 0
                ? span * Math.min(1, Math.max(0, bar.position / bar.maxPosition))
                : 0
        radius: width / 2
        color: thumbMouse.containsMouse || thumbMouse.pressed
                ? Theme.textMuted : Theme.borderStrong

        Behavior on width {
            NumberAnimation { duration: 120 }
        }
    }

    MouseArea {
        id: thumbMouse

        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.ArrowCursor

        // Deslocamento do clique DENTRO do polegar, pra ele não "pular" e
        // centralizar no cursor ao começar o arrasto.
        property real grabOffset: 0

        function positionFor(mouseY) {
            if (thumb.span <= 0) {
                return 0;
            }
            const top = Math.min(thumb.span, Math.max(0, mouseY - grabOffset));
            return (top / thumb.span) * bar.maxPosition;
        }

        onPressed: function(mouse) {
            if (mouse.y >= thumb.y && mouse.y <= thumb.y + thumb.height) {
                grabOffset = mouse.y - thumb.y;
                return;
            }
            // Clique no trilho: salta com o polegar centrado no cursor.
            grabOffset = thumb.height / 2;
            bar.moveRequested(positionFor(mouse.y));
        }

        onPositionChanged: function(mouse) {
            if (!pressed) {
                return;
            }
            bar.moveRequested(positionFor(mouse.y));
        }
    }
}
