pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

Item {
    id: root

    property int lineCount: 1
    property real lineHeight: 1
    property real contentY: 0
    property var visibleLineNumbers: []
    property var breakpointLines: []
    property int executionLine: 0
    property var diffLineKinds: ({})
    property int diffRevision: 0
    property bool blameActive: false
    property var blameLineAnnotations: ({})
    property int blameRevision: 0
    property int blameColumnWidth: 130
    property var diagnosticByLine: ({})
    property int diagnosticRevision: 0
    property int foldingRevision: 0
    property var highlighter: null

    property int hoveredDiagnosticLine: 0
    property string hoveredDiagnosticText: ""

    readonly property int digitCount: Math.max(2, String(root.lineCount).length)
    readonly property int markerLaneWidth: 28
    readonly property int diagnosticLaneWidth: 14
    readonly property int diagnosticSize: 8
    readonly property int numberRightPadding: 6
    readonly property int breakpointSize: 8
    readonly property int breakpointLeft:
        root.markerLaneWidth - root.breakpointSize - 6
    readonly property real breakpointMarkerRight:
        root.breakpointLeft + root.breakpointSize
    readonly property int lineNumberColumnWidth:
        Math.ceil(lineNumberMetrics.advanceWidth("8".repeat(root.digitCount)))
        + root.numberRightPadding
    readonly property real lineNumberLaneStart:
        root.width - root.lineNumberColumnWidth
    readonly property real diagnosticLaneStart:
        root.lineNumberLaneStart - root.diagnosticLaneWidth
    readonly property real diagnosticLeft:
        root.diagnosticLaneStart
        + (root.diagnosticLaneWidth - root.diagnosticSize) / 2
    readonly property real diagnosticMarkerRight:
        root.diagnosticLeft + root.diagnosticSize
    readonly property int firstVisibleIndex:
        Math.max(0, Math.floor(root.contentY / root.lineHeight))
    readonly property int visibleLineCount: Math.max(0,
        Math.min(root.visibleLineNumbers.length - root.firstVisibleIndex,
                 Math.ceil(root.height / root.lineHeight) + 1))

    signal lineClicked(int line)
    signal foldToggleRequested(int line)

    clip: true
    width: root.markerLaneWidth
           + (root.blameActive ? root.blameColumnWidth : 0)
           + root.diagnosticLaneWidth
           + root.lineNumberColumnWidth

    FontMetrics {
        id: lineNumberMetrics

        font.family: Theme.monoFont
        font.pixelSize: Math.max(1, Theme.fontSizeEditor - 2)
    }

    function diffKindFor(line, revision) {
        const kind = root.diffLineKinds[line];
        return kind === undefined ? "" : kind;
    }

    function blameTextFor(line, revision) {
        const text = root.blameLineAnnotations[line];
        return text === undefined ? "" : text;
    }

    function diagnosticSeverityFor(line, revision) {
        const info = root.diagnosticByLine[line];
        return info === undefined ? "" : info.severity;
    }

    function diagnosticMessageFor(line, revision) {
        const info = root.diagnosticByLine[line];
        return info === undefined ? "" : info.message;
    }


    function isFoldableLine(line, revision) {
        return root.highlighter !== null
            && root.highlighter.isFoldableLine(line);
    }

    function isFoldedLine(line, revision) {
        return root.highlighter !== null
            && root.highlighter.isFoldedLine(line);
    }

    Repeater {
        model: root.visibleLineCount

        delegate: Item {
            id: gutterLine

            required property int index
            readonly property int lineNumber:
                Number(root.visibleLineNumbers[
                    root.firstVisibleIndex + index])
            readonly property int visibleIndex:
                root.firstVisibleIndex + index
            readonly property bool hasBreakpoint:
                root.breakpointLines.indexOf(lineNumber) >= 0

            y: visibleIndex * root.lineHeight - root.contentY
            width: root.width
            height: root.lineHeight

            Rectangle {
                readonly property string diffKind:
                    root.diffKindFor(gutterLine.lineNumber,
                                     root.diffRevision)

                anchors.left: parent.left
                anchors.top: parent.top
                width: 3
                height: diffKind === "removed" ? 3 : parent.height
                visible: diffKind !== ""
                color: diffKind === "added" ? Theme.successSoft
                       : (diffKind === "modified" ? Theme.infoSoft
                                                  : Theme.errorSoft)
            }

            Text {
                anchors.left: parent.left
                anchors.leftMargin: 5
                anchors.verticalCenter: parent.verticalCenter
                visible: root.isFoldableLine(gutterLine.lineNumber,
                                             root.foldingRevision)
                text: root.isFoldedLine(gutterLine.lineNumber,
                                        root.foldingRevision) ? "▸" : "▾"
                color: Theme.textMuted
                font.pixelSize: Theme.fontSizeEditor - 3

                MouseArea {
                    anchors.fill: parent
                    anchors.margins: -4
                    cursorShape: Qt.PointingHandCursor
                    onClicked: function(mouse) {
                        root.foldToggleRequested(gutterLine.lineNumber);
                        mouse.accepted = true;
                    }
                }
            }

            Rectangle {
                anchors.verticalCenter: parent.verticalCenter
                anchors.left: parent.left
                anchors.leftMargin: root.breakpointLeft
                width: root.breakpointSize
                height: root.breakpointSize
                radius: root.breakpointSize / 2
                visible: gutterLine.hasBreakpoint
                color: Theme.errorSoft
                border.width: 1
                border.color: Theme.background0
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                anchors.left: parent.left
                anchors.leftMargin: root.markerLaneWidth
                width: root.blameColumnWidth
                visible: root.blameActive
                text: root.blameTextFor(gutterLine.lineNumber,
                                        root.blameRevision)
                color: Theme.textMuted
                font.family: Theme.monoFont
                font.pixelSize: Theme.fontSizeEditor - 3
                elide: Text.ElideRight
            }

            Text {
                id: lineNumberText

                readonly property string diagnosticSeverity:
                    root.diagnosticSeverityFor(gutterLine.lineNumber,
                                               root.diagnosticRevision)

                anchors.right: parent.right
                anchors.rightMargin: root.numberRightPadding
                anchors.verticalCenter: parent.verticalCenter
                text: gutterLine.lineNumber
                color: root.executionLine === gutterLine.lineNumber
                       ? Theme.accent
                       : (diagnosticSeverity !== ""
                          ? StatusColors.severity(diagnosticSeverity)
                          : Theme.textMuted)
                font.family: Theme.monoFont
                font.pixelSize: Theme.fontSizeEditor - 2
                font.bold: diagnosticSeverity === "error"
            }

            Rectangle {
                id: diagnosticDot

                readonly property string severity:
                    root.diagnosticSeverityFor(gutterLine.lineNumber,
                                               root.diagnosticRevision)

                // Diagnostico e numero ocupam faixas geometricas distintas.
                // A posicao nao depende da largura dos digitos nem do bold.
                x: root.diagnosticLeft
                anchors.verticalCenter: parent.verticalCenter
                width: root.diagnosticSize
                height: root.diagnosticSize
                radius: root.diagnosticSize / 2
                visible: severity !== ""
                color: StatusColors.severity(severity)
                border.width: 1
                border.color: Theme.background0

                MouseArea {
                    anchors.fill: parent
                    anchors.margins: -3
                    hoverEnabled: true
                    acceptedButtons: Qt.NoButton
                    onEntered: {
                        root.hoveredDiagnosticLine = gutterLine.lineNumber;
                        root.hoveredDiagnosticText =
                            root.diagnosticMessageFor(gutterLine.lineNumber,
                                                      root.diagnosticRevision);
                    }
                    onExited: {
                        root.hoveredDiagnosticLine = 0;
                        root.hoveredDiagnosticText = "";
                    }
                }
            }

            MouseArea {
                anchors.fill: parent
                cursorShape: Qt.PointingHandCursor
                onClicked: root.lineClicked(gutterLine.lineNumber)
            }
        }
    }
}
