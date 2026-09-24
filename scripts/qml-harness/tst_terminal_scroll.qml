import QtQuick
import "../../ui/qml/panels/bottom"

Item {
    id: root
    width: 100
    height: 100

    property int requestedOffset: -1

    TerminalScrollController {
        id: scroll
        onScrollRequested: function(offset) {
            root.requestedOffset = offset;
        }
    }

    Component.onCompleted: {
        let failures = 0;

        scroll.handleRender({ id: "t1", scrollback: 0, scrollbackMax: 100 });
        // A troca de sessao aparece no `renderedSessionId` (o sinal que so' o
        // harness ouvia saiu no pente-fino de 2026-09-18).
        if (scroll.renderedSessionId !== "t1" || scroll.scrollbackMax !== 100) failures += 1;

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
        if (scroll.renderedSessionId !== "t2"
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
        scroll.queueScroll(20);
        scroll.flushPending();
        scroll.handleRender({ id: "t3", scrollback: 0, scrollbackMax: 0 });
        if (scroll.scrollOffset !== 0 || scroll.awaitingScrollOffset !== -1)
            failures += 8192;
        scroll.handleRender({ id: "t3", scrollback: 0, scrollbackMax: 100 });
        scroll.queueScroll(20);
        scroll.handleRender({ id: "t3", scrollback: 0, scrollbackMax: 0 });
        if (scroll.pendingScrollOffset !== -1) failures += 16384;
        // O codigo de saida de um processo tem 8 BITS: Qt.exit(256) sai como 0.
        // Enquanto o bitmask ia direto para o exit, todo check com bit >= 256
        // era letra morta: passava verde mesmo quebrado, que e exatamente a
        // doenca que esta suite existe para impedir. O mask agora vai para a
        // SAIDA (onde nao trunca) e o exit so diz passou/falhou.
        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
