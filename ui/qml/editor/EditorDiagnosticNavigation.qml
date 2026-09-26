import QtQuick

// IR PARA O PROXIMO (ou anterior) DIAGNOSTICO, a partir do cursor.
//
// Saiu do `EditorController` em 2026-09-25 para pagar a entrada da correcao
// estrutural de indentacao (E1). A V6 previa exatamente isto: "o corte por
// responsabilidade acompanha a primeira fatia que realmente exigir novo dono".
//
// E' uma responsabilidade inteira, e nao um pedaco cortado no tamanho: ela sabe
// perguntar ao dono dos diagnosticos qual e' o proximo a partir de uma posicao,
// levar o cursor ate' la' e devolver o foco — tres coisas que so' fazem sentido
// juntas.
QtObject {
    id: root

    property var diagnosticsController: null
    property var textController: null

    // Ir ao diagnostico e ficar olhando para outro lugar seria meio gesto.
    signal focusRequested()

    function goToNext() {
        return root.jump(true);
    }

    function goToPrevious() {
        return root.jump(false);
    }

    function jump(forward) {
        if (root.diagnosticsController === null || root.textController === null) {
            return false;
        }
        const position = root.textController.cursorLineColumn();
        const target = forward
                ? root.diagnosticsController.nextDiagnostic(position.line, position.column)
                : root.diagnosticsController.prevDiagnostic(position.line, position.column);
        if (target === null) {
            return false;
        }
        root.textController.goToLine(target.line, target.column);
        root.focusRequested();
        return true;
    }
}
