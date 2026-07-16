import QtQuick
import "../../ui/qml/panels/bottom"

Item {
    id: root
    width: 100
    height: 100

    property int requestedOffset: -1
    property int sessionChanges: 0

    TerminalScrollController {
        id: scroll
        onScrollRequested: function(offset) {
            root.requestedOffset = offset;
        }
        onSessionChanged: root.sessionChanges += 1
    }

    Component.onCompleted: {
        let failures = 0;

        scroll.handleRender({ id: "t1", scrollback: 0, scrollbackMax: 100 });
        if (root.sessionChanges !== 1 || scroll.scrollbackMax !== 100) failures += 1;

        scroll.queueScroll(30);
        scroll.flushPending();
        if (root.requestedOffset !== 30 || scroll.awaitingScrollOffset !== 30) failures += 2;

        // Output novo durante a leitura desloca o offset para preservar a
        // mesma pagina; isso confirma o gesto em vez de congelar o controller.
        scroll.handleRender({ id: "t1", scrollback: 33, scrollbackMax: 103 });
        if (scroll.scrollOffset !== 33 || scroll.awaitingScrollOffset !== -1) failures += 4;

        scroll.queueScroll(999);
        if (scroll.scrollOffset !== 103) failures += 8;
        scroll.flushPending();
        scroll.snapToBottom();
        if (root.requestedOffset !== 0 || scroll.scrollOffset !== 0
                || scroll.awaitingScrollOffset !== 0) failures += 16;

        // Frame velho do historico nao desfaz o retorno ao prompt vivo.
        scroll.handleRender({ id: "t1", scrollback: 80, scrollbackMax: 103 });
        if (scroll.scrollOffset !== 0 || scroll.awaitingScrollOffset !== 0) failures += 32;
        scroll.handleRender({ id: "t1", scrollback: 0, scrollbackMax: 103 });
        if (scroll.awaitingScrollOffset !== -1) failures += 64;

        scroll.handleRender({ id: "t2", scrollback: 0, scrollbackMax: 0 });
        if (root.sessionChanges !== 2 || scroll.renderedSessionId !== "t2"
                || scroll.scrollOffset !== 0) failures += 128;

        // A roda so CONVERTE unidade de dispositivo em linhas; nao rola nada e
        // nao toca no estado. Quem decide o destino do gesto e o core, que le o
        // modo VT (protocolo 0.60.0). Ver TerminalScrollController.
        scroll.handleRender({ id: "t3", scrollback: 0, scrollbackMax: 100 });

        // No Arch/Wayland um gesto de alta resolucao pode chegar sem
        // angleDelta. O fallback de pixelDelta precisa dar linhas para cima.
        if (scroll.linesFromWheel(0, 36, 18) !== 2) failures += 256;

        // Roda tradicional preserva os tres passos por notch, nos dois sentidos.
        if (scroll.linesFromWheel(120, 0, 18) !== 3) failures += 512;
        if (scroll.linesFromWheel(-120, 0, 18) !== -3) failures += 1024;

        // Delta nulo nao fabrica gesto.
        if (scroll.linesFromWheel(0, 0, 18) !== 0) failures += 2048;

        // A regressao que originou a fatia: converter a roda NAO pode mexer no
        // offset. Se voltar a mexer, a regra de negocio voltou para a UI e o
        // Claude para de rolar de novo.
        if (scroll.scrollOffset !== 0
                || scroll.pendingScrollOffset !== -1) failures += 4096;

        Qt.exit(failures);
    }
}
