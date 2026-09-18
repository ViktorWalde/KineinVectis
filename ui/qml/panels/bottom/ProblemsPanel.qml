pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

ListView {
    id: panel


    // B2 (DocsPublic/roadmaps/24): barra de rolagem. `parent: panel` é OBRIGATÓRIO — um filho
    // declarado dentro de um ListView vira filho do contentItem e ROLARIA
    // junto com a lista. O ListView segue sendo a fonte da verdade.
    VerticalScrollBar {
        id: scrollBar_panel

        parent: panel
        anchors.right: panel.right
        anchors.top: panel.top
        anchors.bottom: panel.bottom

        contentSize: panel.contentHeight
        viewportSize: panel.height
        position: panel.contentY

        onMoveRequested: function(position) {
            panel.contentY = position;
        }
    }
    ListModel {
        id: emptyDiagnosticsModel
    }

    property var diagnosticsModel: emptyDiagnosticsModel

    signal openRequested(string file, int line, int column)
    // F5: o proximo passo do problema (codeActions | health, com o alvo).
    signal nextStepRequested(string kind, string target, string file, int line, int column)

    // A regra do proximo passo mora fora da tela (ProblemRules).
    ProblemRules {
        id: nextStep
    }


    clip: true
    spacing: 2
    model: diagnosticsModel

    Text {
        anchors.centerIn: parent
        visible: panel.diagnosticsModel.count === 0
        text: qsTr("Nenhum problema. Rode um build (Ctrl+F9) ou "
                   + "uma análise (Ctrl+Shift+L).")
        color: Theme.textMuted
        font.pixelSize: 11
    }

    delegate: Rectangle {
        id: problemDelegate

        required property string severity
        required property string message
        required property string code
        required property string file
        required property int line
        required property int column
        required property string source

        readonly property var step: nextStep.stepFor({ source: source, message: message })

        width: panel.width
        height: problemRow.height + Theme.spacingSmall
        radius: Theme.radius
        color: problemArea.containsMouse ? Theme.surface2 : "transparent"

        Row {
            id: problemRow

            anchors.verticalCenter: parent.verticalCenter
            anchors.left: parent.left
            anchors.leftMargin: Theme.spacingSmall
            spacing: Theme.spacingSmall
            width: parent.width - 2 * Theme.spacingSmall

            Rectangle {
                width: 8
                height: 8
                radius: 4
                anchors.verticalCenter: parent.verticalCenter
                color: StatusColors.severity(problemDelegate.severity)
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                visible: problemDelegate.file !== ""
                text: problemDelegate.file + ":" + problemDelegate.line
                color: Theme.accent
                font.family: Theme.monoFont
                font.pixelSize: 11
            }

            Text {
                id: codeLabel

                anchors.verticalCenter: parent.verticalCenter
                visible: problemDelegate.code !== ""
                text: problemDelegate.code
                color: Theme.textMuted
                font.family: Theme.monoFont
                font.pixelSize: 10
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                width: parent.width - x - (passo.visible ? passo.width + Theme.spacingSmall : 0)
                text: problemDelegate.message
                color: Theme.textPrimary
                font.pixelSize: 11
                elide: Text.ElideRight
            }
        }

        // O proximo passo, a direita: sempre visivel quando existe — e' o
        // que a referencia faz; esconder ate' o hover e' esconder a ajuda.
        KvButton {
            id: passo

            anchors.right: parent.right
            anchors.rightMargin: Theme.spacingSmall
            anchors.verticalCenter: parent.verticalCenter
            z: 2
            visible: problemDelegate.step.label !== ""
            compact: true
            text: problemDelegate.step.label
            onClicked: panel.nextStepRequested(problemDelegate.step.kind, problemDelegate.step.target,
                                               problemDelegate.file, problemDelegate.line,
                                               problemDelegate.column)
        }

        MouseArea {
            id: problemArea

            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: panel.openRequested(problemDelegate.file, problemDelegate.line,
                                           problemDelegate.column)
        }
    }
}
