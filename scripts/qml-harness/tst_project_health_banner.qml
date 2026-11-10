// C1-33 (consolidação C §20.1): numa janela estreita, a faixa do Project
// Health corta a mensagem, e pairar sobre ela mostra a mensagem inteira. A
// ProjectHealthBanner e o TooltipController REAIS; a mensagem é a do
// multi-config, a mais longa que o C1-22 trouxe.
import QtQuick
import KineinVectis

Item {
    id: root
    width: 1400
    height: 200

    readonly property string longMessage: "a IDE compila e testa Release, a que o clangd lê; "
                                          + "o cmake --build sem --config compila Debug"

    ProjectHealthBanner {
        id: banner

        width: 520
        menuSpace: root
        active: true
        status: "info"
        message: root.longMessage
    }

    function messageText() {
        for (let i = 0; i < banner.children.length; i++) {
            const child = banner.children[i];
            if (child.text === banner.message && child.elide === Text.ElideRight) return child;
        }
        return null;
    }

    function areaOf(item) {
        for (let i = 0; i < item.children.length; i++) {
            if (item.children[i].hoverEnabled !== undefined) return item.children[i];
        }
        return null;
    }

    Timer {
        interval: 50
        running: true
        onTriggered: {
            let failures = 0;
            const text = root.messageText();
            const area = text === null ? null : root.areaOf(text);
            if (text === null || area === null || !text.truncated) failures += 1 << 0;
            // A área só paira: o clique segue para o que está embaixo.
            if (area !== null && (!area.hoverEnabled || area.acceptedButtons !== Qt.NoButton)) failures += 1 << 1;
            if (area !== null) {
                area.entered();
                if (!TooltipController.shown || TooltipController.text !== root.longMessage
                        || TooltipController.owner !== text)
                    failures += 1 << 2;
                area.exited();
                if (TooltipController.shown) failures += 1 << 3;
                // Inteira, a mensagem não precisa de dica.
                banner.width = 1400;
                if (text.truncated) failures += 1 << 4;
                area.entered();
                if (TooltipController.shown) failures += 1 << 5;
                area.exited();
                // Com o botão de ação, a mensagem cabe em menos: cortada de novo.
                banner.actionLabel = "Configurar";
                banner.width = 640;
                area.entered();
                if (!text.truncated || !TooltipController.shown
                        || TooltipController.text !== root.longMessage)
                    failures += 1 << 6;
                area.exited();
            }
            if (failures !== 0) console.error("FALHAS bitmask=" + failures);
            Qt.exit(failures === 0 ? 0 : 1);
        }
    }
}
