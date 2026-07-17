import QtQuick

// Politica de teclado do editor: decide, na PRECEDENCIA correta, se a tecla e'
// consumida e qual sinal dispara — pares primeiro, depois navegacao dos popups
// (acoes > completion > usages/hover), depois edicao inteligente (Tab, Home,
// Enter). Vive fora da EditorTextSurface porque "o que cada tecla faz" e' uma
// politica com ordem propria e donos proprios; a superficie so pergunta
// "consumida?" e aplica event.accepted. Extraida na fatia E6 (§4 regra 9:
// corte por responsabilidade — a superficie perdeu o CONCEITO de navegacao de
// popup, nao apenas as linhas).
QtObject {
    id: root

    // A superficie: dona dos sinais e das flags de visibilidade dos popups.
    required property var surface
    // As regras de par (EditorAutoClosePairs): precedencia maxima.
    required property var pairs

    // true = tecla consumida (o chamador aplica event.accepted).
    function route(event) {
        if (event.key === Qt.Key_Backspace && root.pairs.handlePairBackspace()) {
            return true;
        }
        if (root.pairs.handleTypingKey(event)) {
            return true;
        }
        if (root.surface.actionsVisible) {
            if (event.key === Qt.Key_Down) {
                root.surface.actionsMoveRequested(1);
                return true;
            }
            if (event.key === Qt.Key_Up) {
                root.surface.actionsMoveRequested(-1);
                return true;
            }
            if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter) {
                root.surface.actionsAcceptRequested();
                return true;
            }
            if (event.key === Qt.Key_Escape) {
                root.surface.actionsDismissRequested();
                return true;
            }
        }
        if (root.surface.completionVisible) {
            if (event.key === Qt.Key_Down) {
                root.surface.completionMoveRequested(1);
                return true;
            }
            if (event.key === Qt.Key_Up) {
                root.surface.completionMoveRequested(-1);
                return true;
            }
            if (event.key === Qt.Key_Return
                    || event.key === Qt.Key_Enter
                    || event.key === Qt.Key_Tab) {
                root.surface.completionAcceptRequested();
                return true;
            }
            if (event.key === Qt.Key_Escape) {
                root.surface.completionDismissRequested();
                return true;
            }
        }
        if (event.key === Qt.Key_Escape) {
            if (root.surface.usagesVisible) {
                root.surface.usagesDismissRequested();
                return true;
            }
            if (root.surface.hoverVisible) {
                root.surface.hoverDismissRequested();
                return true;
            }
        }
        if (event.key === Qt.Key_Tab
                && (event.modifiers & Qt.ShiftModifier)) {
            root.surface.unindentRequested();
            return true;
        }
        if (event.key === Qt.Key_Tab) {
            root.surface.indentRequested();
            return true;
        }
        if (event.key === Qt.Key_Backtab) {
            root.surface.unindentRequested();
            return true;
        }
        if (event.key === Qt.Key_Home
                && (event.modifiers === Qt.NoModifier
                    || event.modifiers === Qt.ShiftModifier)) {
            // E3: Home inteligente; Ctrl+Home (inicio do documento) segue
            // com o TextEdit.
            root.surface.smartHomeRequested(event.modifiers === Qt.ShiftModifier);
            return true;
        }
        if ((event.key === Qt.Key_Return || event.key === Qt.Key_Enter)
                && event.modifiers === Qt.NoModifier) {
            root.surface.newlineRequested();
            return true;
        }
        return false;
    }
}
