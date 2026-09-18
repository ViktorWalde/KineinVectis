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
    // M3.2: linha→kind do diff git (added|modified|removed); a revisão força
    // o rebind. A cobertura (P5) segue a mesma forma.
    property var diffLineKinds: ({})
    property int diffRevision: 0
    property var coverageLineKinds: ({})
    property int coverageRevision: 0
    // M3.4: blame por linha ("autor, idade") como coluna extra da gutter,
    // ligado/desligado pelo comando "Git: Blame do arquivo".
    property bool blameActive: false
    property var blameLineAnnotations: ({})
    property int blameRevision: 0
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
        if (editorHighlighter !== null) editorHighlighter.setDiagnostics(diagnosticSpans);
    }

    signal gutterLineClicked(int line)
    signal codeActionsRequested(int line)
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
    property bool autoCloseEnabled: true

    function cursorPointIn(item) {
        const rect = textEditor.cursorRectangle;
        return textEditor.mapToItem(item, rect.x, rect.y);
    }

    readonly property real editorLineHeight: Math.max(1,
        textEditor.cursorRectangle.height > 0
            ? textEditor.cursorRectangle.height
            : Theme.fontSizeEditor * 1.35)

    // Toda a REGRA de tecla mora aqui (auto-close de pares e roteamento);
    // esta superficie so' desenha e repassa. Ver EditorTypingController.qml.
    EditorTypingController {
        id: typingController

        surface: root
        autoCloseEnabled: root.autoCloseEnabled
        completionVisible: root.completionVisible
        actionsVisible: root.actionsVisible
        usagesVisible: root.usagesVisible
        hoverVisible: root.hoverVisible

        onCompletionMoveRequested: delta => root.completionMoveRequested(delta)
        onCompletionAcceptRequested: root.completionAcceptRequested()
        onCompletionDismissRequested: root.completionDismissRequested()
        onActionsMoveRequested: delta => root.actionsMoveRequested(delta)
        onActionsAcceptRequested: root.actionsAcceptRequested()
        onActionsDismissRequested: root.actionsDismissRequested()
        onUsagesDismissRequested: root.usagesDismissRequested()
        onHoverDismissRequested: root.hoverDismissRequested()
        onIndentRequested: root.indentRequested()
        onUnindentRequested: root.unindentRequested()
        onNewlineRequested: root.newlineRequested()
        onCloserBraceRequested: root.closerBraceRequested()
        onSmartHomeRequested: extendSelection =>
            root.smartHomeRequested(extendSelection)
    }

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
        coverageLineKinds: root.coverageLineKinds
        coverageRevision: root.coverageRevision
        blameActive: root.blameActive
        blameLineAnnotations: root.blameLineAnnotations
        blameRevision: root.blameRevision
        diagnosticByLine: root.diagnosticByLine
        diagnosticRevision: root.diagnosticRevision
        foldingRevision: root.foldingRevision
        highlighter: editorHighlighter
        cursorY: root.hasOpenFile ? textEditor.cursorRectangle.y : -1
        onLineClicked: line => root.gutterLineClicked(line)
        onActionsRequested: line => root.codeActionsRequested(line)
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

        EditorLineHighlights {
            width: Math.max(editorFlick.contentWidth, editorFlick.width)
            active: root.hasOpenFile
            cursorY: textEditor.cursorRectangle.y
            cursorHeight: textEditor.cursorRectangle.height
            hasSelection: textEditor.selectionStart !== textEditor.selectionEnd
            executionLine: root.executionLine
            lineHeight: root.editorLineHeight
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
            onTextChanged: root.textEdited(text)
            Keys.onPressed: function(event) {
                event.accepted = typingController.route(event);
            }
        }
    }

    // B2 (DocsPublic/roadmaps/24): barra de rolagem do editor. Até 2026-07-12 o editor era um
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

    EditorDiagnosticTooltip {
        id: diagnosticTooltip

        message: editorGutter.hoveredDiagnosticText
        lineHeight: root.editorLineHeight
        lineTop: editorGutter.y - editorFlick.contentY
                 + (editorGutter.hoveredDiagnosticLine - 1) * diagnosticTooltip.lineHeight
        x: editorGutter.x + editorGutter.width + Theme.spacingSmall
        maxAvailableWidth: root.width - x - 3 * Theme.spacingSmall
    }
}
