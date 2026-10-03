import QtQuick

// DE QUE LADO cada janela acoplada abre (2026-10-03, pedido do autor: "o
// painel aparecer no canto de acordo com o trilho em que o icone estiver").
// As janelas sao as do slot (Projeto, Git, Banco); o lado e' o do icone no
// trilho (`railState.sides`, o que o usuario arrastou), padrao esquerda.
//
// Um slot por lado: a esquerda e' o de sempre (`leftWindow` + `showExplorer`),
// a direita e' `rightWindow` ("" = fechado). A MESMA janela nunca fica nos
// dois: abrir de um lado fecha do outro, e arrastar o icone com a janela
// aberta leva a janela junto. Puro sobre o estado do ShellController.
QtObject {
    id: root

    property var shell: null

    // O slot de um lado tem o que mostrar agora (o host liga o `visible`).
    // Sem projeto, nenhum: a tela de boas-vindas e' so' de boas-vindas
    // (decisao do autor, 2026-10-03).
    function slotVisible(side, workspaceOpen) {
        const name = root.windowOn(side);
        if (name === "" || !workspaceOpen) return false;
        if (side === "right") return root.shell.effectiveOutlineCollapsed;
        return root.shell.showExplorer && name !== root.shell.rightWindow;
    }

    function sideOf(name) {
        const sides = root.shell.railState.sides;
        return sides !== undefined && sides !== null && sides[name] === "right" ? "right" : "left";
    }

    // A janela posta no slot de um lado ("" = nenhuma).
    function windowOn(side) {
        return side === "right" ? root.shell.rightWindow : root.shell.leftWindow;
    }

    // A barra de ordem do trilho de um lado (ShellController.savedOrder).
    function barOf(side) {
        return side === "right" ? "railRight" : "rail";
    }

    // Em que slot a janela esta' POSTA: "left", "right" ou "".
    function placed(name) {
        if (root.shell.rightWindow === name) return "right";
        return root.shell.showExplorer && root.shell.leftWindow === name ? "left" : "";
    }

    // Onde ela esta' VISIVEL agora. Os Simbolos (o ☰ da direita, decisao do
    // autor de 2026-10-03) ocupam o lado direito: abertos, o slot da direita
    // some por um momento e volta igual quando eles fecham.
    function showing(name) {
        const side = root.placed(name);
        return side === "right" && !root.shell.effectiveOutlineCollapsed ? "" : side;
    }

    function show(name) {
        if (root.sideOf(name) === "right") {
            if (root.shell.leftWindow === name) root.shell.showExplorer = false;
            root.shell.rightWindow = name;
            // Pedir a janela escondida pelos Simbolos fecha os Simbolos.
            if (!root.shell.outlineCollapsed) root.shell.toggleOutline();
            return;
        }
        if (root.shell.rightWindow === name) root.shell.rightWindow = "";
        root.shell.leftWindow = name;
        root.shell.showExplorer = true;
    }

    function close(name) {
        if (root.shell.rightWindow === name) root.shell.rightWindow = "";
        if (root.shell.leftWindow === name) root.shell.showExplorer = false;
    }

    // O icone do que esta' aberto fecha; o do outro traz o outro.
    function toggle(name) {
        if (root.showing(name) !== "") root.close(name);
        else root.show(name);
    }

    // Layout gravado antes de o icone mudar de trilho (ou antes de existir o
    // slot da direita): cada janela aberta vai para o lado do icone dela.
    function normalize() {
        const left = root.placed(root.shell.leftWindow) === "left" ? root.shell.leftWindow : "";
        const right = root.shell.rightWindow;
        if (left !== "" && root.sideOf(left) === "right") root.show(left);
        if (right !== "" && root.sideOf(right) === "left") root.show(right);
    }

    // A janela aberta pede mais largura (o Banco com uma grade larga): o
    // slot dela ALARGA ate' `width` — o teto do ShellController ainda vale —
    // e nunca encolhe por aqui.
    function widen(name, width) {
        const side = root.showing(name);
        if (side === "right" && root.shell.rightPreferredWidth < width) {
            root.shell.rightPreferredWidth = width;
        } else if (side === "left" && root.shell.explorerPreferredWidth < width) {
            root.shell.explorerPreferredWidth = width;
        } else {
            return;
        }
        root.shell.persistLayoutSoon();
    }

    // O icone mudou de trilho: aberta do lado antigo, a janela vai junto.
    function followSide(name, side) {
        const now = root.placed(name);
        if (root.shell.leftWindows.indexOf(name) >= 0 && now !== "" && now !== side) {
            root.show(name);
        }
    }
}
