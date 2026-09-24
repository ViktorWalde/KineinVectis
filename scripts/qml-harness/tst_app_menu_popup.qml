import QtQuick
import KineinVectis

Item {
    id: root
    width: 220
    height: 120
    property string action: ""
    property int dismissals: 0
    property bool restoreFocus: false
    property int failures: 0
    Item { id: otherFocus }
    AppMenuPopup {
        id: menu
        anchors.fill: parent
        menuWidth: 320
        menuX: 210
        menuY: 110
        items: [
            { label: "Indisponivel", action: "disabled", enabled: false },
            { label: "Copiar", action: "copy", enabled: true },
            { label: "Colar", action: "paste", enabled: false },
            { label: "Selecionar", action: "select", enabled: true },
            { label: "Limpar", action: "clear", enabled: true },
            { label: "Novo", action: "new", enabled: true },
            { label: "Fechar", action: "close", enabled: true }
        ]
        onActionRequested: function(id) { root.action = id; }
        onDismissRequested: function(restoreFocus) {
            root.dismissals++;
            root.restoreFocus = restoreFocus;
        }
    }
    function check(ok, reason) {
        if (!ok) { failures++; console.error(reason); }
    }
    function press(key) {
        const event = { key: key, accepted: false };
        menu.handleKey(event);
        check(event.accepted, "menu precisa consumir tecla");
    }
    Timer {
        interval: 100
        running: true
        onTriggered: {
            otherFocus.forceActiveFocus();
            menu.prepare();
            check(menu.activeFocus, "menu precisa de foco ativo");
            check(menu.previousFocusItem === otherFocus, "menu guarda foco anterior para o host");
            check(menu.currentIndex === 1, "primeira acao disponivel");
            press(Qt.Key_Down);
            check(menu.currentIndex === 3, "navegacao pula item desabilitado");
            press(Qt.Key_Return);
            check(root.action === "select", "Enter executa item escolhido");
            menu.activate(0);
            check(root.action === "select", "acao desabilitada nao dispara");
            press(Qt.Key_End);
            check(menu.currentIndex === 6, "ultima acao alcancavel em painel baixo");
            press(Qt.Key_A);
            check(root.action === "select", "letra nao executa acao");
            const frame = menu.children[1];
            check(frame.width <= root.width && frame.height <= root.height,
                  "menu deve caber no painel");
            check(frame.x >= 0 && frame.y >= 0
                  && frame.x + frame.width <= root.width
                  && frame.y + frame.height <= root.height, "menu dentro das bordas");
            const before = root.dismissals;
            press(Qt.Key_Escape);
            check(root.dismissals === before + 1, "Escape fecha sem chegar ao shell");
            check(root.restoreFocus, "Escape devolve foco ao chamador");
            otherFocus.forceActiveFocus();
            check(root.dismissals === before + 2 && !root.restoreFocus,
                  "perder foco fecha sem roubar foco da nova superficie");
            menu.visible = false;
            root.forceActiveFocus();
            menu.restorePreviousFocus();
            check(otherFocus.activeFocus, "host pode restaurar foco anterior ao fechar");
            Qt.exit(root.failures === 0 ? 0 : 1);
        }
    }
}
