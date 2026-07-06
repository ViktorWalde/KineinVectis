import QtQuick

Rectangle {
    id: root

    property alias text: textEditor.text
    property alias cursorPosition: textEditor.cursorPosition
    readonly property int selectionStart: textEditor.selectionStart
    readonly property int selectionEnd: textEditor.selectionEnd
    readonly property var cursorRectangle: textEditor.cursorRectangle
    readonly property bool editorActiveFocus: textEditor.activeFocus
    property bool hasOpenFile: false
    property string emptyMessage: ""
    property bool completionVisible: false
    property bool usagesVisible: false
    property bool hoverVisible: false

    signal textEdited(string text)
    signal completionMoveRequested(int delta)
    signal completionAcceptRequested()
    signal completionDismissRequested()
    signal usagesDismissRequested()
    signal hoverDismissRequested()
    signal indentRequested()
    signal unindentRequested()
    signal newlineRequested()

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
    }

    function setSemanticTokens(tokens) {
        editorHighlighter.setSemanticTokens(tokens);
    }

    function cursorPointIn(item) {
        const rect = textEditor.cursorRectangle;
        return textEditor.mapToItem(item, rect.x, rect.y);
    }

    Text {
        anchors.centerIn: parent
        visible: !root.hasOpenFile
        width: Math.min(parent.width - 40, 480)
        horizontalAlignment: Text.AlignHCenter
        wrapMode: Text.WordWrap
        text: root.emptyMessage
        color: Theme.textMuted
        font.pixelSize: 14
    }

    Flickable {
        id: editorFlick

        anchors.fill: parent
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

        TextEdit {
            id: textEditor

            EditorHighlighter {
                id: editorHighlighter

                document: textEditor.textDocument
            }

            width: Math.max(editorFlick.width, contentWidth)
            height: Math.max(editorFlick.height, contentHeight)
            color: Theme.textPrimary
            selectionColor: Theme.accentDim
            selectedTextColor: Theme.textPrimary
            font.family: Theme.monoFont
            font.pixelSize: 13
            wrapMode: TextEdit.NoWrap
            selectByMouse: true
            tabStopDistance: 4 * 8
            onCursorRectangleChanged: editorFlick.ensureVisible(cursorRectangle)
            onTextChanged: root.textEdited(text)
            Keys.onPressed: function(event) {
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
                if ((event.key === Qt.Key_Return || event.key === Qt.Key_Enter)
                        && event.modifiers === Qt.NoModifier) {
                    root.newlineRequested();
                    event.accepted = true;
                }
            }
        }
    }
}
