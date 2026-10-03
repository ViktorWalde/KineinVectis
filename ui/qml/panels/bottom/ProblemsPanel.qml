pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

ListView {
    id: panel

    FlickableScrollBar {
        id: scrollBar_panel

        view: panel
    }

    ListModel {
        id: emptyDiagnosticsModel
    }

    property var diagnosticsModel: emptyDiagnosticsModel
    // O caminho na linha e' relativo ao projeto (F5 na tela real, 2026-10-03:
    // o absoluto do compilador ocupava a linha inteira e a mensagem sumia).
    property string workspaceRoot: ""

    function displayPath(file) {
        const root = panel.workspaceRoot;
        return root !== "" && file.indexOf(root + "/") === 0 ? file.substring(root.length + 1) : file;
    }

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
        font.pixelSize: Theme.fontSizeSmall
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
                radius: height / 2
                anchors.verticalCenter: parent.verticalCenter
                color: StatusColors.severity(problemDelegate.severity)
            }

            // O caminho cabe em ate' 40% da linha, cortado no MEIO (o nome do
            // arquivo e a linha ficam a vista); a mensagem leva o resto.
            Text {
                id: pathLabel

                anchors.verticalCenter: parent.verticalCenter
                visible: problemDelegate.file !== ""
                width: Math.min(implicitWidth, problemRow.width * 0.4)
                text: panel.displayPath(problemDelegate.file) + ":" + problemDelegate.line
                color: Theme.accent
                font.family: Theme.monoFont
                font.pixelSize: Theme.fontSizeSmall
                elide: Text.ElideMiddle
            }

            Text {
                id: codeLabel

                anchors.verticalCenter: parent.verticalCenter
                visible: problemDelegate.code !== ""
                width: Math.min(implicitWidth, problemRow.width * 0.2)
                text: problemDelegate.code
                color: Theme.textMuted
                font.family: Theme.monoFont
                font.pixelSize: Theme.fontSizeCaption
                elide: Text.ElideRight
            }

            // So' a primeira linha da mensagem: as notas do compilador (a
            // segunda linha em diante) ficam na dica, com a mensagem inteira.
            Text {
                anchors.verticalCenter: parent.verticalCenter
                width: Math.max(0, parent.width - x - (passo.visible ? passo.width + Theme.spacingSmall : 0))
                text: problemDelegate.message.split("\n")[0]
                color: Theme.textPrimary
                font.pixelSize: Theme.fontSizeSmall
                maximumLineCount: 1
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
            onContainsMouseChanged: {
                if (containsMouse) {
                    TooltipController.showFor(problemDelegate,
                                              panel.displayPath(problemDelegate.file) + ":" + problemDelegate.line
                                              + "\n" + problemDelegate.message, "top");
                } else {
                    TooltipController.hideFor(problemDelegate);
                }
            }
        }
    }
}
