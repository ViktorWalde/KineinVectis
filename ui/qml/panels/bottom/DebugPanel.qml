pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Aba Debug do painel inferior (fatia M2.5b): controles da sessao DAP e a
// saida do processo depurado. Componente burro: estado entra por property,
// intencao sai por signal (o DebugController decide).
Item {
    id: panel

    ListModel {
        id: emptyOutputModel
    }

    ListModel {
        id: emptyInspectionModel
    }

    property var outputModel: emptyOutputModel
    property bool sessionActive: false
    property bool paused: false
    property var framesModel: emptyInspectionModel
    property var variablesModel: emptyInspectionModel
    property int currentFrameIndex: -1

    signal frameActivated(int index)
    signal variableToggled(int index)
    signal continueRequested()
    signal pauseRequested()
    signal stepOverRequested()
    signal stepIntoRequested()
    signal stepOutRequested()
    signal stopRequested()

    function lineColor(kind) {
        if (kind === "command") {
            return Theme.accent;
        }
        if (kind === "stderr") {
            return Theme.errorSoft;
        }
        if (kind === "info") {
            return Theme.textMuted;
        }
        return Theme.textSecondary;
    }

    Row {
        id: debugControlsRow

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        height: 26
        spacing: Theme.spacingSmall

        Repeater {
            model: [
                { key: "continue", label: qsTr("Continuar (Ctrl+Alt+C)"),
                  needsPaused: true },
                { key: "pause", label: qsTr("Pausar"), needsPaused: false },
                { key: "stepOver", label: qsTr("Step Over (Ctrl+Alt+N)"),
                  needsPaused: true },
                { key: "stepInto", label: qsTr("Step Into (Ctrl+Alt+I)"),
                  needsPaused: true },
                { key: "stepOut", label: qsTr("Step Out (Ctrl+Alt+U)"),
                  needsPaused: true },
                { key: "stop", label: qsTr("Parar"), needsPaused: false }
            ]

            delegate: Rectangle {
                id: debugControl

                required property var modelData

                readonly property bool actionEnabled: panel.sessionActive
                    && (modelData.needsPaused
                        ? panel.paused
                        : (modelData.key !== "pause" || !panel.paused))

                width: debugControlLabel.width + 2 * Theme.spacingSmall
                height: 24
                anchors.verticalCenter: parent.verticalCenter
                radius: Theme.radiusXSmall
                opacity: actionEnabled ? 1.0 : 0.45
                color: debugControlArea.containsMouse && actionEnabled
                       ? Theme.surface2 : Theme.surface1
                border.color: Theme.borderSoft
                border.width: 1

                Text {
                    id: debugControlLabel

                    anchors.centerIn: parent
                    text: debugControl.modelData.label
                    color: debugControl.modelData.key === "stop"
                           ? Theme.errorSoft : Theme.textPrimary
                    font.pixelSize: 11
                }

                MouseArea {
                    id: debugControlArea

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: debugControl.actionEnabled
                                 ? Qt.PointingHandCursor : Qt.ArrowCursor
                    onClicked: {
                        if (!debugControl.actionEnabled) {
                            return;
                        }
                        if (debugControl.modelData.key === "continue") {
                            panel.continueRequested();
                        } else if (debugControl.modelData.key === "pause") {
                            panel.pauseRequested();
                        } else if (debugControl.modelData.key === "stepOver") {
                            panel.stepOverRequested();
                        } else if (debugControl.modelData.key === "stepInto") {
                            panel.stepIntoRequested();
                        } else if (debugControl.modelData.key === "stepOut") {
                            panel.stepOutRequested();
                        } else {
                            panel.stopRequested();
                        }
                    }
                }
            }
        }
    }

    ListView {
        id: debugOutputView


        // B2 (docs/24): barra de rolagem. `parent: debugOutputView` é OBRIGATÓRIO — um filho
        // declarado dentro de um ListView vira filho do contentItem e ROLARIA
        // junto com a lista. O ListView segue sendo a fonte da verdade.
        VerticalScrollBar {
            id: scrollBar_debugOutputView

            parent: debugOutputView
            anchors.right: debugOutputView.right
            anchors.top: debugOutputView.top
            anchors.bottom: debugOutputView.bottom

            contentSize: debugOutputView.contentHeight
            viewportSize: debugOutputView.height
            position: debugOutputView.contentY

            onMoveRequested: function(position) {
                debugOutputView.contentY = position;
            }
        }
        anchors.top: debugControlsRow.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        width: panel.paused
               ? Math.max(220, parent.width - framesBox.width
                          - variablesBox.width - 2 * Theme.spacingSmall)
               : parent.width
        clip: true
        model: panel.outputModel
        onCountChanged: positionViewAtEnd()

        Text {
            anchors.centerIn: parent
            visible: panel.outputModel.count === 0
            text: qsTr("Clique na gutter para marcar breakpoints e use o"
                       + " botão Debug na barra superior (Ctrl+Alt+D).")
            color: Theme.textMuted
            font.pixelSize: 11
        }

        delegate: Text {
            id: debugLineDelegate

            required property string line
            required property string kind

            width: debugOutputView.width
            text: line
            color: panel.lineColor(debugLineDelegate.kind)
            font.family: Theme.monoFont
            font.pixelSize: 11
            wrapMode: Text.WrapAnywhere
        }
    }

    Rectangle {
        id: framesBox

        anchors.top: debugControlsRow.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.bottom: parent.bottom
        anchors.left: debugOutputView.right
        anchors.leftMargin: Theme.spacingSmall
        width: 230
        visible: panel.paused
        radius: Theme.radius
        color: Theme.background1
        border.color: Theme.borderSoft
        border.width: 1

        Text {
            id: framesTitle

            anchors.top: parent.top
            anchors.left: parent.left
            anchors.margins: Theme.spacingSmall
            text: qsTr("Frames")
            color: Theme.textMuted
            font.pixelSize: 10
            font.bold: true
        }

        ListView {
            anchors.top: framesTitle.bottom
            anchors.topMargin: Theme.spacingXSmall
            anchors.bottom: parent.bottom
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.margins: Theme.spacingSmall
            clip: true
            model: panel.framesModel

            delegate: Rectangle {
                id: frameRow

                required property int index
                required property string name
                required property string file
                required property int line

                width: ListView.view.width
                height: 22
                radius: Theme.radiusXSmall
                color: panel.currentFrameIndex === frameRow.index
                       ? Theme.surfaceSelected
                       : (frameRowArea.containsMouse ? Theme.surface2
                                                     : "transparent")

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.leftMargin: Theme.spacingSmall
                    anchors.rightMargin: Theme.spacingSmall
                    text: frameRow.line > 0
                          ? qsTr("%1  ·  :%2").arg(frameRow.name).arg(frameRow.line)
                          : frameRow.name
                    color: panel.currentFrameIndex === frameRow.index
                           ? Theme.textPrimary : Theme.textSecondary
                    font.pixelSize: 11
                    elide: Text.ElideRight
                }

                MouseArea {
                    id: frameRowArea

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: panel.frameActivated(frameRow.index)
                }
            }
        }
    }

    Rectangle {
        id: variablesBox

        anchors.top: debugControlsRow.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.bottom: parent.bottom
        anchors.left: framesBox.right
        anchors.leftMargin: Theme.spacingSmall
        anchors.right: parent.right
        visible: panel.paused
        radius: Theme.radius
        color: Theme.background1
        border.color: Theme.borderSoft
        border.width: 1

        Text {
            id: variablesTitle

            anchors.top: parent.top
            anchors.left: parent.left
            anchors.margins: Theme.spacingSmall
            text: qsTr("Variáveis")
            color: Theme.textMuted
            font.pixelSize: 10
            font.bold: true
        }

        ListView {
            anchors.top: variablesTitle.bottom
            anchors.topMargin: Theme.spacingXSmall
            anchors.bottom: parent.bottom
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.margins: Theme.spacingSmall
            clip: true
            model: panel.variablesModel

            delegate: Rectangle {
                id: variableRowDelegate

                required property int index
                required property string name
                required property string value
                required property string typeName
                required property real reference
                required property int depth
                required property bool expanded

                width: ListView.view.width
                height: 20
                radius: Theme.radiusXSmall
                color: variableRowArea.containsMouse
                       && variableRowDelegate.reference > 0
                       ? Theme.surface2 : "transparent"

                Text {
                    id: variableArrow

                    anchors.verticalCenter: parent.verticalCenter
                    anchors.left: parent.left
                    anchors.leftMargin: Theme.spacingSmall
                                        + variableRowDelegate.depth * 14
                    width: 12
                    text: variableRowDelegate.reference > 0
                          ? (variableRowDelegate.expanded ? "▾" : "▸") : ""
                    color: Theme.textMuted
                    font.pixelSize: 10
                }

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.left: variableArrow.right
                    anchors.right: parent.right
                    anchors.rightMargin: Theme.spacingSmall
                    text: variableRowDelegate.typeName !== ""
                          ? qsTr("%1 = %2  (%3)")
                                .arg(variableRowDelegate.name)
                                .arg(variableRowDelegate.value)
                                .arg(variableRowDelegate.typeName)
                          : qsTr("%1 = %2")
                                .arg(variableRowDelegate.name)
                                .arg(variableRowDelegate.value)
                    color: Theme.textSecondary
                    font.family: Theme.monoFont
                    font.pixelSize: 11
                    elide: Text.ElideRight
                }

                MouseArea {
                    id: variableRowArea

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: variableRowDelegate.reference > 0
                                 ? Qt.PointingHandCursor : Qt.ArrowCursor
                    onClicked: panel.variableToggled(variableRowDelegate.index)
                }
            }
        }
    }
}
