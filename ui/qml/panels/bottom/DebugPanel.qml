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
    property var watchesModel

    signal frameActivated(int index)
    signal variableToggled(int index)
    signal watchAdded(string expression)
    signal watchRemoved(int index)
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


        // B2 (DocsPublic/roadmaps/24): barra de rolagem. `parent: debugOutputView` é OBRIGATÓRIO — um filho
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
        // Parado, a saida divide o espaco com o inspetor; rodando, ocupa tudo.
        // Antes esta conta somava as larguras de framesBox e variablesBox, que
        // hoje moram no DebugInspector — o vizinho passou a ser ele.
        width: panel.paused
               ? Math.max(220, parent.width - inspector.width
                          - watches.width - 2 * Theme.spacingSmall)
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

    DebugInspector {
        id: inspector

        anchors.top: debugControlsRow.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.bottom: parent.bottom
        anchors.left: debugOutputView.right
        anchors.leftMargin: Theme.spacingSmall
        anchors.right: watches.left
        anchors.rightMargin: Theme.spacingSmall

        paused: panel.paused
        framesModel: panel.framesModel
        variablesModel: panel.variablesModel
        currentFrameIndex: panel.currentFrameIndex

        onFrameActivated: function(index) { panel.frameActivated(index); }
        onVariableToggled: function(index) { panel.variableToggled(index); }
    }

    DebugWatches {
        id: watches

        anchors.top: debugControlsRow.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.bottom: parent.bottom
        anchors.right: parent.right
        width: 240

        paused: panel.paused
        watchesModel: panel.watchesModel

        onWatchAdded: function(expression) { panel.watchAdded(expression); }
        onWatchRemoved: function(index) { panel.watchRemoved(index); }
    }
}
