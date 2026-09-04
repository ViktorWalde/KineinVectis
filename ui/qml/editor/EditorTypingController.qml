pragma ComponentBehavior: Bound
import QtQuick

// O QUE UMA TECLA SIGNIFICA dentro do editor de texto.
//
// POR QUE ESTE ARQUIVO EXISTE (2026-09-03). Ate' esta data as duas
// responsabilidades abaixo moravam dentro do EditorTextSurface.qml, que e' um
// componente VISUAL (Rectangle com gutter, Flickable, TextEdit, scrollbar e
// tooltip). Sao 234 linhas de REGRA convivendo com pintura:
//
//   1. auto-close de pares: fechar o par, type-over do fechador, envolver a
//      selecao, `#include <` fechando em `<>`, backspace apagando par vazio;
//   2. roteamento de tecla: qual intencao (indentar, aceitar completacao,
//      fechar popup, Home inteligente, nova linha) a tecla representa DADO o
//      que esta aberto na tela.
//
// Nenhuma das duas e' desenho. O corte e' por RESPONSABILIDADE (ARCHITECTURE.md
// §4 regra 9), nao por tamanho: depois dele `grep -c "Qt.Key_"` no
// EditorTextSurface.qml devolve 0, e `grep -ci "pair"` tambem.
//
// COMO CONVERSA COM A SUPERFICIE. Este controller NAO conhece o TextEdit; ele
// fala com a API publica do EditorTextSurface (`text`, `cursorPosition`,
// `selectionStart`, `selectionEnd`, `remove`, `insert`, `select`). Por isso o
// harness QML consegue exercita-lo com um objeto falso de cinco funcoes.
//
// AS TABELAS DE PAR NAO ESTAO AQUI. Elas sao regra pura de texto e vivem no
// singleton TextRules.qml, junto com `isWordChar` — que antes estava copiado
// aqui e no EditorTextController.
QtObject {
    id: root

    // Regras puras de texto — uma unica DEFINICAO, ver TextRules.qml.
    readonly property TextRules rules: TextRules {}

    // A superficie de texto (EditorTextSurface, ou um duble no harness).
    property var surface: null

    // M4.1: liga/desliga o auto-close de pares (setting autoClosePairs).
    property bool autoCloseEnabled: true

    // O que esta aberto na tela muda o significado de Enter, Esc e setas.
    property bool completionVisible: false
    property bool actionsVisible: false
    property bool usagesVisible: false
    property bool hoverVisible: false

    signal completionMoveRequested(int delta)
    signal completionAcceptRequested()
    signal completionDismissRequested()
    signal actionsMoveRequested(int delta)
    signal actionsAcceptRequested()
    signal actionsDismissRequested()
    signal usagesDismissRequested()
    signal hoverDismissRequested()
    signal indentRequested()
    signal unindentRequested()
    signal newlineRequested()
    signal closerBraceRequested()
    signal smartHomeRequested(bool extendSelection)

    // Unico ponto de entrada. true = tecla CONSUMIDA (o chamador marca
    // event.accepted). A ordem dos testes e' significativa e esta' descrita
    // em docs/roadmaps/39, §2.1.
    function route(event) {
        if (surface === null) {
            return false;
        }
        if (event.key === Qt.Key_Backspace && handleBackspace()) {
            return true;
        }
        if (handleTypingKey(event)) {
            return true;
        }
        if (root.actionsVisible) {
            if (event.key === Qt.Key_Down) {
                root.actionsMoveRequested(1);
                return true;
            }
            if (event.key === Qt.Key_Up) {
                root.actionsMoveRequested(-1);
                return true;
            }
            if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter) {
                root.actionsAcceptRequested();
                return true;
            }
            if (event.key === Qt.Key_Escape) {
                root.actionsDismissRequested();
                return true;
            }
        }
        if (root.completionVisible) {
            if (event.key === Qt.Key_Down) {
                root.completionMoveRequested(1);
                return true;
            }
            if (event.key === Qt.Key_Up) {
                root.completionMoveRequested(-1);
                return true;
            }
            if (event.key === Qt.Key_Return
                    || event.key === Qt.Key_Enter
                    || event.key === Qt.Key_Tab) {
                root.completionAcceptRequested();
                return true;
            }
            if (event.key === Qt.Key_Escape) {
                root.completionDismissRequested();
                return true;
            }
        }
        if (event.key === Qt.Key_Escape) {
            if (root.usagesVisible) {
                root.usagesDismissRequested();
                return true;
            }
            if (root.hoverVisible) {
                root.hoverDismissRequested();
                return true;
            }
        }
        if (event.key === Qt.Key_Tab
                && (event.modifiers & Qt.ShiftModifier)) {
            root.unindentRequested();
            return true;
        }
        if (event.key === Qt.Key_Tab) {
            root.indentRequested();
            return true;
        }
        if (event.key === Qt.Key_Backtab) {
            root.unindentRequested();
            return true;
        }
        if (event.key === Qt.Key_Home
                && (event.modifiers === Qt.NoModifier
                    || event.modifiers === Qt.ShiftModifier)) {
            // E3: Home inteligente; Ctrl+Home (início do documento) segue
            // com o TextEdit.
            root.smartHomeRequested(event.modifiers === Qt.ShiftModifier);
            return true;
        }
        if ((event.key === Qt.Key_Return || event.key === Qt.Key_Enter)
                && event.modifiers === Qt.NoModifier) {
            root.newlineRequested();
            return true;
        }
        return false;
    }

    // E1 (docs-privada/diario/18, trilha E): auto-close de pares, type-over do
    // fechador, surround da seleção e `#include <`.
    // true = tecla consumida.
    function handleTypingKey(event) {
        if (event.text === "" || !autoCloseEnabled) {
            return false;
        }
        // Ctrl puro é atalho; Ctrl+Alt (AltGr em layouts europeus) produz
        // caractere legítimo e passa.
        if ((event.modifiers & Qt.ControlModifier)
                && !(event.modifiers & Qt.AltModifier)) {
            return false;
        }
        const character = event.text;
        const closer = root.rules.pairOpeners[character];
        const position = surface.cursorPosition;
        const content = surface.text;
        const hasSelection = surface.selectionStart !== surface.selectionEnd;

        // CR1: "<" logo após `#include ` fecha em "<>" (contexto seguro;
        // "<" genérico é comparação/template/shift e NÃO auto-fecha).
        if (character === "<" && !hasSelection && position > 0) {
            const lineStart = content.lastIndexOf("\n", position - 1) + 1;
            const beforeCursor = content.substring(lineStart, position);
            if (/^\s*#\s*include\s+$/.test(beforeCursor)) {
                surface.insert(position, "<>");
                surface.cursorPosition = position + 1;
                return true;
            }
        }

        if (hasSelection && closer !== undefined) {
            // Abridor com seleção ativa ENVOLVE em vez de substituir.
            const start = surface.selectionStart;
            const end = surface.selectionEnd;
            const selected = content.substring(start, end);
            surface.remove(start, end);
            surface.insert(start, character + selected + closer);
            surface.select(start + 1, end + 1);
            return true;
        }
        if (root.rules.pairClosers[character] !== undefined && !hasSelection
                && content.charAt(position) === character) {
            // type-over: pula o fechador já presente em vez de duplicar.
            surface.cursorPosition = position + 1;
            return true;
        }
        if (closer !== undefined) {
            const previous = position > 0 ? content.charAt(position - 1) : "";
            const next = content.charAt(position);
            const quote = character === "\"" || character === "'";
            // Aspas coladas em palavra não duplicam (don't → don''t);
            // colchetes/parênteses antes de palavra ou aspas também não.
            if (quote && (root.rules.isWordChar(previous)
                          || root.rules.isWordChar(next))) {
                return false;
            }
            if (!quote && (root.rules.isWordChar(next)
                           || next === "\"" || next === "'")) {
                return false;
            }
            surface.insert(position, character + closer);
            surface.cursorPosition = position + 1;
            return true;
        }
        if (character === "}" && !hasSelection) {
            // E3: a inserção (com dedent quando a linha é só whitespace) vive
            // no EditorTextController; o type-over acima tem precedência e não
            // re-indenta.
            root.closerBraceRequested();
            return true;
        }
        return false;
    }

    // Backspace entre um par VAZIO apaga os dois caracteres.
    function handleBackspace() {
        if (!autoCloseEnabled
                || surface.selectionStart !== surface.selectionEnd) {
            return false;
        }
        const position = surface.cursorPosition;
        if (position <= 0) {
            return false;
        }
        const content = surface.text;
        const previous = content.charAt(position - 1);
        const closer = root.rules.pairOpeners[previous];
        if (closer !== undefined && content.charAt(position) === closer) {
            surface.remove(position - 1, position + 1);
            return true;
        }
        return false;
    }
}
