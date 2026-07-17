pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

Rectangle {
    id: root

    property alias text: textEditor.text
    property alias cursorPosition: textEditor.cursorPosition
    readonly property alias language: editorHighlighter.language
    readonly property int selectionStart: textEditor.selectionStart
    readonly property int selectionEnd: textEditor.selectionEnd
    readonly property var cursorRectangle: textEditor.cursorRectangle
    readonly property bool editorActiveFocus: textEditor.activeFocus
    property bool hasOpenFile: false
    property string emptyMessage: ""
    property bool completionVisible: false
    property bool usagesVisible: false
    property bool hoverVisible: false
    property bool actionsVisible: false
    // C4 (gutter): breakpoints do arquivo atual e a linha de execucao
    // pausada do debugger (0 = nenhuma). Estado vive no DebugController.
    property var breakpointLines: []
    property int executionLine: 0
    // M3.2: linha→kind do diff git (added|modified|removed) do arquivo
    // atual; a revisão força o rebind das marcas.
    property var diffLineKinds: ({})
    property int diffRevision: 0
    // M3.4: blame por linha ("autor, idade") como coluna extra da gutter,
    // ligado/desligado pelo comando "Git: Blame do arquivo".
    property bool blameActive: false
    property var blameLineAnnotations: ({})
    property int blameRevision: 0
    readonly property int blameColumnWidth: 130
    // T6: diagnósticos do arquivo ativo. Os spans (0-based UTF-16) vão
    // para o highlighter (sublinhado ondulado); o mapa linha→{severity,
    // message} desenha a marca e o tooltip da gutter.
    property var diagnosticSpans: []
    property var diagnosticByLine: ({})
    property int diagnosticRevision: 0
    // Linhas logicas que continuam visiveis depois do folding. O renderer
    // C++ deriva isto dos QTextBlocks para a gutter nao renumerar o arquivo.
    property var visibleLineNumbers: []
    property int foldingRevision: 0

    onDiagnosticSpansChanged: {
        if (editorHighlighter !== null) {
            editorHighlighter.setDiagnostics(diagnosticSpans);
        }
    }

    signal gutterLineClicked(int line)
    signal textEdited(string text)
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

    radius: Theme.radius
    color: Theme.background0

    function remove(start, end) {
        textEditor.remove(start, end);
    }

    function insert(position, text) {
        textEditor.insert(position, text);
    }

    function select(start, end) {
        textEditor.select(start, end);
    }

    function focusEditor() {
        textEditor.forceActiveFocus();
    }

    function setFilePath(path) {
        editorHighlighter.filePath = path;
        visibleLineNumbers = editorHighlighter.visibleLineNumbers();
    }

    function setSemanticTokens(tokens) {
        editorHighlighter.setSemanticTokens(tokens);
    }

    function clearSemanticTokens() {
        editorHighlighter.clearSemanticTokens();
    }

    function setSyntaxSnapshot(tokens, foldingRanges) {
        editorHighlighter.setSyntaxTokens(tokens);
        editorHighlighter.setFoldingRanges(foldingRanges);
    }

    // D1b: ocorrências do Find realçadas no texto (a atual mais forte).
    function setSearchMatches(matches, current) {
        editorHighlighter.setSearchMatches(matches, current);
    }

    // M4.1: liga/desliga o auto-close de pares (setting autoClosePairs).
    // Continua aqui porque e' API que o ShellWorkspaceHost liga via EditorPane;
    // as REGRAS de par sao do EditorAutoClosePairs.
    property bool autoCloseEnabled: true

    EditorAutoClosePairs {
        id: autoClosePairs
        target: textEditor
        enabled: root.autoCloseEnabled
        onCloserBraceRequested: root.closerBraceRequested()
    }

    function cursorPointIn(item) {
        const rect = textEditor.cursorRectangle;
        return textEditor.mapToItem(item, rect.x, rect.y);
    }

    readonly property real editorLineHeight: Math.max(1,
        textEditor.cursorRectangle.height > 0
            ? textEditor.cursorRectangle.height
            : Theme.fontSizeEditor * 1.35)

    Text {
        anchors.centerIn: parent
        visible: !root.hasOpenFile
        width: Math.min(parent.width - 40, 480)
        horizontalAlignment: Text.AlignHCenter
        wrapMode: Text.WordWrap
        text: root.emptyMessage
        color: Theme.textMuted
        font.pixelSize: Theme.fontSizeEditor
    }

    EditorGutter {
        id: editorGutter

        anchors.left: parent.left
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.bottomMargin: Theme.spacingSmall
        anchors.leftMargin: Theme.spacingXSmall
        visible: root.hasOpenFile
        lineCount: textEditor.lineCount
        lineHeight: root.editorLineHeight
        contentY: editorFlick.contentY
        visibleLineNumbers: root.visibleLineNumbers
        breakpointLines: root.breakpointLines
        executionLine: root.executionLine
        diffLineKinds: root.diffLineKinds
        diffRevision: root.diffRevision
        blameActive: root.blameActive
        blameLineAnnotations: root.blameLineAnnotations
        blameRevision: root.blameRevision
        blameColumnWidth: root.blameColumnWidth
        diagnosticByLine: root.diagnosticByLine
        diagnosticRevision: root.diagnosticRevision
        foldingRevision: root.foldingRevision
        highlighter: editorHighlighter
        onLineClicked: line => root.gutterLineClicked(line)
        onFoldToggleRequested: line => editorHighlighter.toggleFoldAtLine(line)
    }

    Flickable {
        id: editorFlick

        anchors.left: editorGutter.right
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        anchors.margins: Theme.spacingSmall
        visible: root.hasOpenFile
        clip: true
        contentWidth: textEditor.contentWidth + 20
        contentHeight: textEditor.contentHeight + 20
        boundsBehavior: Flickable.StopAtBounds

        function ensureVisible(rect) {
            if (contentX >= rect.x) {
                contentX = rect.x;
            } else if (contentX + width <= rect.x + rect.width) {
                contentX = rect.x + rect.width - width;
            }
            if (contentY >= rect.y) {
                contentY = rect.y;
            } else if (contentY + height <= rect.y + rect.height) {
                contentY = rect.y + rect.height - height;
            }
        }

        // C4: linha do cursor destacada (some durante selecao; a linha de
        // execucao do debugger, mais forte, desenha por cima).
        Rectangle {
            visible: root.hasOpenFile
                     && textEditor.selectionStart === textEditor.selectionEnd
            y: textEditor.cursorRectangle.y
            height: textEditor.cursorRectangle.height
            width: Math.max(editorFlick.contentWidth, editorFlick.width)
            color: Theme.surfaceSelected
        }

        Rectangle {
            visible: root.executionLine > 0
            y: (root.executionLine - 1) * root.editorLineHeight
            width: Math.max(editorFlick.contentWidth, editorFlick.width)
            height: root.editorLineHeight
            color: Theme.accentDim
            opacity: 0.35
        }

        TextEdit {
            id: textEditor

            EditorHighlighter {
                id: editorHighlighter

                document: textEditor.textDocument
            }

            Connections {
                target: editorHighlighter

                function onFoldingChanged() {
                    root.visibleLineNumbers = editorHighlighter.visibleLineNumbers();
                    root.foldingRevision++;
                }
            }

            width: Math.max(editorFlick.width, contentWidth)
            height: Math.max(editorFlick.height, contentHeight)
            color: Theme.textPrimary
            selectionColor: Theme.accentDim
            selectedTextColor: Theme.textPrimary
            font.family: Theme.monoFont
            font.pixelSize: Theme.fontSizeEditor
            wrapMode: TextEdit.NoWrap
            selectByMouse: true
            tabStopDistance: 4 * 8
            onCursorRectangleChanged: editorFlick.ensureVisible(cursorRectangle)
            // O EditorHighlighter acima e um QSyntaxHighlighter no documento
            // deste TextEdit, e rehighlight() marca o documento como alterado
            // mesmo quando so o FORMATO mudou (medido em Qt 6.11.1: texto
            // identico, contentsChanged=1, onTextChanged=1). Sem esta barreira
            // cada passada de realce se apresenta como edicao do usuario, e o
            // consumidor refiltra o autocomplete: a selecao do usuario volta ao
            // primeiro item sozinha. `textEdited` significa texto EDITADO.
            property string lastNotifiedText: ""
            onTextChanged: {
                if (text === lastNotifiedText) {
                    return;
                }
                lastNotifiedText = text;
                root.textEdited(text);
            }
            Keys.onPressed: function(event) {
                if (event.key === Qt.Key_Backspace
                        && autoClosePairs.handlePairBackspace()) {
                    event.accepted = true;
                    return;
                }
                if (autoClosePairs.handleTypingKey(event)) {
                    event.accepted = true;
                    return;
                }
                if (root.actionsVisible) {
                    if (event.key === Qt.Key_Down) {
                        root.actionsMoveRequested(1);
                        event.accepted = true;
                        return;
                    }
                    if (event.key === Qt.Key_Up) {
                        root.actionsMoveRequested(-1);
                        event.accepted = true;
                        return;
                    }
                    if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter) {
                        root.actionsAcceptRequested();
                        event.accepted = true;
                        return;
                    }
                    if (event.key === Qt.Key_Escape) {
                        root.actionsDismissRequested();
                        event.accepted = true;
                        return;
                    }
                }
                if (root.completionVisible) {
                    if (event.key === Qt.Key_Down) {
                        root.completionMoveRequested(1);
                        event.accepted = true;
                        return;
                    }
                    if (event.key === Qt.Key_Up) {
                        root.completionMoveRequested(-1);
                        event.accepted = true;
                        return;
                    }
                    if (event.key === Qt.Key_Return
                            || event.key === Qt.Key_Enter
                            || event.key === Qt.Key_Tab) {
                        root.completionAcceptRequested();
                        event.accepted = true;
                        return;
                    }
                    if (event.key === Qt.Key_Escape) {
                        root.completionDismissRequested();
                        event.accepted = true;
                        return;
                    }
                }
                if (event.key === Qt.Key_Escape) {
                    if (root.usagesVisible) {
                        root.usagesDismissRequested();
                        event.accepted = true;
                        return;
                    }
                    if (root.hoverVisible) {
                        root.hoverDismissRequested();
                        event.accepted = true;
                    }
                }
                if (event.key === Qt.Key_Tab
                        && (event.modifiers & Qt.ShiftModifier)) {
                    root.unindentRequested();
                    event.accepted = true;
                    return;
                }
                if (event.key === Qt.Key_Tab) {
                    root.indentRequested();
                    event.accepted = true;
                    return;
                }
                if (event.key === Qt.Key_Backtab) {
                    root.unindentRequested();
                    event.accepted = true;
                    return;
                }
                if (event.key === Qt.Key_Home
                        && (event.modifiers === Qt.NoModifier
                            || event.modifiers === Qt.ShiftModifier)) {
                    // E3: Home inteligente; Ctrl+Home (início do
                    // documento) segue com o TextEdit.
                    root.smartHomeRequested(
                        event.modifiers === Qt.ShiftModifier);
                    event.accepted = true;
                    return;
                }
                if ((event.key === Qt.Key_Return || event.key === Qt.Key_Enter)
                        && event.modifiers === Qt.NoModifier) {
                    root.newlineRequested();
                    event.accepted = true;
                }
            }
        }
    }

    // B2 (docs/roadmaps/24): barra de rolagem do editor. Até 2026-07-12 o editor era um
    // Flickable SEM indicador nenhum — não dava pra saber o tamanho do arquivo
    // nem onde se estava nele. O Flickable segue sendo a fonte da verdade: a
    // barra só reflete `contentY` e PEDE mudança por `moveRequested`.
    VerticalScrollBar {
        id: editorScrollBar

        anchors.right: editorFlick.right
        anchors.top: editorFlick.top
        anchors.bottom: editorFlick.bottom
        visible: root.hasOpenFile && scrollable

        contentSize: editorFlick.contentHeight
        viewportSize: editorFlick.height
        position: editorFlick.contentY

        onMoveRequested: function(position) {
            editorFlick.contentY = position;
        }
    }

    // T6: tooltip da mensagem do diagnóstico ao passar o mouse na marca
    // da gutter (a UI não usa QtQuick.Controls; é um popup próprio leve).
    Rectangle {
        id: diagnosticTooltip

        readonly property real lineTop: editorGutter.y
            + (editorGutter.hoveredDiagnosticLine - 1) * root.editorLineHeight
            - editorFlick.contentY

        readonly property real maxTextWidth: Math.min(420,
            root.width - x - 3 * Theme.spacingSmall)

        visible: editorGutter.hoveredDiagnosticText !== ""
        z: 30
        x: editorGutter.x + editorGutter.width + Theme.spacingSmall
        y: Math.max(Theme.spacingSmall,
                    lineTop + root.editorLineHeight)
        width: diagnosticTooltipText.width + 2 * Theme.spacingSmall
        height: diagnosticTooltipText.height + 2 * Theme.spacingSmall
        radius: Theme.radius
        color: Theme.background2
        border.color: Theme.borderStrong
        border.width: 1

        Text {
            id: diagnosticTooltipText

            x: Theme.spacingSmall
            y: Theme.spacingSmall
            // implicitWidth (não embrulhado) é constante para o texto;
            // cortar no máximo evita o binding circular do wrap.
            width: Math.min(implicitWidth, diagnosticTooltip.maxTextWidth)
            text: editorGutter.hoveredDiagnosticText
            color: Theme.textPrimary
            font.pixelSize: Theme.fontSizeEditor - 2
            wrapMode: Text.WordWrap
        }
    }
}
