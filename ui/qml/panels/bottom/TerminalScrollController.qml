import QtQuick

// Estado e coalescencia da rolagem EXPLICITA do terminal (barra e
// snap-to-bottom). O core continua sendo a fonte da verdade; este controller
// apenas impede que renders de output cruzados com arrasto façam a leitura
// saltar ou congelem uma confirmacao para sempre.
//
// Este controller NAO decide o que a roda significa. Ate 0.59 ele traduzia todo
// gesto de roda em offset de scrollback e mandava `terminal.scroll`, sempre —
// uma regra de negocio na UI, e a causa de o Claude nao rolar: ele desenha em
// tela alternada (sem historico por semantica VT) e captura o mouse, entao o
// pedido caia no vazio. Quem sabe o destino de um gesto e o terminal, porque so
// ele conhece o modo VT que a aplicacao ligou. A UI reporta o gesto cru por
// `terminal.mouse` (protocolo 0.60.0) e desenha o que voltar.
Item {
    id: root

    property int scrollOffset: 0
    property int scrollbackMax: 0
    property int pendingScrollOffset: -1
    property int awaitingScrollOffset: -1
    property string renderedSessionId: ""

    signal scrollRequested(int offset)
    signal sessionChanged(string id)

    visible: false

    Timer {
        id: scrollFlush

        interval: 16
        repeat: false
        onTriggered: root.flushPending()
    }

    function resetSession(id) {
        scrollFlush.stop();
        scrollOffset = 0;
        scrollbackMax = 0;
        pendingScrollOffset = -1;
        awaitingScrollOffset = -1;
        renderedSessionId = id;
        sessionChanged(id);
    }

    function handleRender(render) {
        const nextSessionId = render && render.id !== undefined
                ? String(render.id) : "";
        if (nextSessionId !== renderedSessionId) {
            resetSession(nextSessionId);
        }
        if (render && render.scrollbackMax !== undefined) {
            scrollbackMax = Math.max(0, Number(render.scrollbackMax));
        }
        if (!render || render.scrollback === undefined) {
            return;
        }

        const confirmed = Math.max(0, Math.min(
            scrollbackMax, Number(render.scrollback)));
        if (pendingScrollOffset >= 0) {
            return;
        }
        if (awaitingScrollOffset >= 0) {
            // Nova saida aumenta o offset do vt100 para manter a pagina lida.
            // Dois offsets positivos diferentes ainda confirmam a mesma
            // intencao; zero vs. positivo pode ser apenas um frame cruzado.
            const sameReadingMode = awaitingScrollOffset > 0 && confirmed > 0;
            if (confirmed !== awaitingScrollOffset && !sameReadingMode) {
                return;
            }
            awaitingScrollOffset = -1;
        }
        scrollOffset = confirmed;
    }

    function queueScroll(next) {
        const clamped = Math.max(0, Math.min(scrollbackMax, next));
        scrollOffset = clamped;
        pendingScrollOffset = clamped;
        if (!scrollFlush.running) {
            scrollFlush.start();
        }
    }

    // Converte um gesto de roda em LINHAS. Só isso: quantas linhas o usuário
    // pediu, com o sinal do gesto (positivo = para cima). O que fazer com elas
    // é decisão do core.
    //
    // Esta conversão é legitimamente da UI — é unidade de dispositivo, não
    // semântica de terminal. Roda tradicional costuma preencher angleDelta (120
    // por notch), mas Qt/Wayland e dispositivos de alta resolucao podem fornecer
    // somente pixelDelta. Aceitar ambos evita interpretar um gesto para cima
    // como zero/para baixo.
    //
    // Devolve 0 quando não há gesto — o chamador não deve reportar nada.
    function linesFromWheel(angleDeltaY, pixelDeltaY, pixelsPerLine) {
        const angle = Number(angleDeltaY);
        const pixels = Number(pixelDeltaY);
        const delta = angle !== 0 ? angle : pixels;
        if (!Number.isFinite(delta) || delta === 0) {
            return 0;
        }
        const magnitude = angle !== 0
                ? Math.max(1, Math.round(Math.abs(angle) / 120 * 3))
                : Math.max(1, Math.round(
                    Math.abs(pixels) / Math.max(1, pixelsPerLine)));
        return delta > 0 ? magnitude : -magnitude;
    }

    function flushPending() {
        if (pendingScrollOffset < 0) {
            return;
        }
        const offset = pendingScrollOffset;
        pendingScrollOffset = -1;
        awaitingScrollOffset = offset;
        scrollRequested(offset);
    }

    function snapToBottom() {
        const needsRequest = scrollOffset !== 0
                || pendingScrollOffset >= 0 || awaitingScrollOffset >= 0;
        scrollFlush.stop();
        pendingScrollOffset = -1;
        scrollOffset = 0;
        if (needsRequest) {
            // Mantem a confirmacao pendente para ignorar um render antigo do
            // historico que cruze com o retorno ao prompt vivo.
            awaitingScrollOffset = 0;
            scrollRequested(0);
        } else {
            awaitingScrollOffset = -1;
        }
    }
}
