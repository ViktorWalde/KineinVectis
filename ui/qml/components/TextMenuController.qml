pragma Singleton
import QtQuick

// O MENU DE CONTEXTO DOS CAMPOS DE TEXTO (2026-10-04, pedido do autor:
// "copiar/colar tanto pelo atalho do teclado quanto pelo mouse"). Dono unico,
// como o TooltipController: o KvTextField pede com o clique direito; o
// KvTextMenuHost (na camada flutuante, acima de qualquer dialogo) desenha o
// AppMenuPopup. Recortar, Copiar, Colar, Selecionar tudo e Limpar, cada um
// apagado quando nao se aplica (campo de senha nao copia; somente leitura nao
// cola nem recorta).
QtObject {
    id: root

    property bool open: false
    property var field: null
    property real menuX: 0
    property real menuY: 0
    property var items: []
    // A selecao no momento do clique: abrir o menu tira o foco do campo, e a
    // entrada desfaz a selecao — sem isto, Copiar nao copiava nada.
    property int selectionStart: 0
    property int selectionEnd: 0

    function openFor(field, sceneX, sceneY, clipboardHasText) {
        const input = field.input;
        const secret = input.echoMode !== TextInput.Normal;
        const selected = input.selectedText !== "";
        const editable = !input.readOnly;
        const sep = { separator: true, label: "", action: "", enabled: false };
        root.items = [
            { label: qsTr("Recortar"), action: "cut", icon: "cut", shortcut: "Ctrl+X", enabled: selected && editable && !secret },
            { label: qsTr("Copiar"), action: "copy", icon: "copy", shortcut: "Ctrl+C", enabled: selected && !secret },
            { label: qsTr("Colar"), action: "paste", icon: "paste", shortcut: "Ctrl+V", enabled: editable && clipboardHasText },
            sep,
            { label: qsTr("Selecionar tudo"), action: "selectAll", icon: "", shortcut: "Ctrl+A", enabled: input.text !== "" },
            { label: qsTr("Limpar"), action: "clear", icon: "close", shortcut: "", enabled: editable && input.text !== "" }
        ];
        root.field = field;
        root.selectionStart = input.selectionStart;
        root.selectionEnd = input.selectionEnd;
        root.menuX = sceneX;
        root.menuY = sceneY;
        root.open = true;
    }

    function close() {
        root.open = false;
    }

    // O gesto escolhido, no campo que pediu (o menu ja' fechou).
    function run(action) {
        const field = root.field;
        root.open = false;
        root.field = null;
        if (field !== null) field.runMenuAction(action, root.selectionStart, root.selectionEnd);
    }
}
