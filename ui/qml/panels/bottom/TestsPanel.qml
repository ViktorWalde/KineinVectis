pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

Item {
    id: panel

    ListModel {
        id: emptyCasesModel
    }

    property var casesModel: emptyCasesModel
    property string summary: ""
    property bool running: false
    // Objeto de cobertura do JobsController: { measuring, summary, error,
    // filesModel, start() }. Nulo enquanto nao ha workspace.
    property var coverage: null

    readonly property bool hasCoverage: coverage !== null
                                        && (coverage.summary !== ""
                                            || coverage.error !== "")

    function coverageColor(percent) {
        if (percent >= 80) {
            return Theme.successSoft;
        }
        if (percent >= 50) {
            return Theme.textSecondary;
        }
        return Theme.errorSoft;
    }

    function statusColor(status) {
        if (status === "passed") {
            return Theme.successSoft;
        }
        if (status === "failed") {
            return Theme.errorSoft;
        }
        return Theme.textMuted;
    }

    Text {
        id: testSummaryLabel

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        visible: panel.summary !== ""
        text: panel.summary
        color: Theme.textSecondary
        font.pixelSize: 11
        font.bold: true
    }

    ListView {
        id: testCasesView


        // B2 (docs/roadmaps/24): barra de rolagem. `parent: testCasesView` é OBRIGATÓRIO — um filho
        // declarado dentro de um ListView vira filho do contentItem e ROLARIA
        // junto com a lista. O ListView segue sendo a fonte da verdade.
        VerticalScrollBar {
            id: scrollBar_testCasesView

            parent: testCasesView
            anchors.right: testCasesView.right
            anchors.top: testCasesView.top
            anchors.bottom: testCasesView.bottom

            contentSize: testCasesView.contentHeight
            viewportSize: testCasesView.height
            position: testCasesView.contentY

            onMoveRequested: function(position) {
                testCasesView.contentY = position;
            }
        }
        anchors.top: panel.summary !== "" ? testSummaryLabel.bottom : parent.top
        anchors.topMargin: panel.summary !== "" ? Theme.spacingSmall : 0
        anchors.bottom: panel.hasCoverage ? coverageBand.top : parent.bottom
        anchors.bottomMargin: panel.hasCoverage ? Theme.spacingSmall : 0
        anchors.left: parent.left
        anchors.right: parent.right
        clip: true
        spacing: 1
        model: panel.casesModel
        onCountChanged: positionViewAtEnd()

        Text {
            anchors.centerIn: parent
            visible: panel.casesModel.count === 0 && !panel.running
            text: qsTr("Nenhum teste rodado. Use Testes (Ctrl+Shift+F9).")
            color: Theme.textMuted
            font.pixelSize: 11
        }

        delegate: Row {
            id: testDelegate

            required property string name
            required property string status

            width: testCasesView.width
            height: 18
            spacing: Theme.spacingSmall

            Rectangle {
                width: 7
                height: 7
                radius: 4
                anchors.verticalCenter: parent.verticalCenter
                color: panel.statusColor(testDelegate.status)
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: testDelegate.name
                color: testDelegate.status === "failed"
                       ? Theme.textPrimary : Theme.textSecondary
                font.family: Theme.monoFont
                font.pixelSize: 11
                elide: Text.ElideRight
                width: testCasesView.width - 16
            }
        }
    }

    // Faixa de cobertura: mora com os testes porque e o mesmo gesto ("rodei os
    // testes; quanto do codigo eles tocaram?"). Ocupa altura ZERO enquanto
    // ninguem mediu — o painel de testes continua sendo o de testes.
    Column {
        id: coverageBand

        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        visible: panel.hasCoverage
        height: visible ? Math.min(implicitHeight, panel.height * 0.5) : 0
        spacing: Theme.spacingXSmall

        Text {
            width: coverageBand.width
            visible: panel.coverage !== null && panel.coverage.error !== ""
            text: panel.coverage !== null ? panel.coverage.error : ""
            color: Theme.errorSoft
            font.pixelSize: 11
            wrapMode: Text.WordWrap
        }

        Text {
            width: coverageBand.width
            visible: panel.coverage !== null && panel.coverage.summary !== ""
            text: qsTr("Cobertura: %1").arg(
                      panel.coverage !== null ? panel.coverage.summary : "")
            color: Theme.textSecondary
            font.pixelSize: 11
            font.bold: true
            elide: Text.ElideRight
        }

        ListView {
            id: coverageFilesView

            width: coverageBand.width
            height: Math.min(contentHeight,
                             panel.height * 0.5 - Theme.spacingLarge)
            clip: true
            spacing: 1
            model: panel.coverage !== null ? panel.coverage.filesModel : null

            VerticalScrollBar {
                parent: coverageFilesView
                anchors.right: coverageFilesView.right
                anchors.top: coverageFilesView.top
                anchors.bottom: coverageFilesView.bottom

                contentSize: coverageFilesView.contentHeight
                viewportSize: coverageFilesView.height
                position: coverageFilesView.contentY

                onMoveRequested: function(position) {
                    coverageFilesView.contentY = position;
                }
            }

            delegate: Row {
                id: coverageDelegate

                required property string path
                required property double percent
                required property int linesCovered
                required property int linesTotal

                width: coverageFilesView.width
                height: 18
                spacing: Theme.spacingSmall

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    width: 44
                    horizontalAlignment: Text.AlignRight
                    text: qsTr("%1%").arg(coverageDelegate.percent.toFixed(0))
                    color: panel.coverageColor(coverageDelegate.percent)
                    font.family: Theme.monoFont
                    font.pixelSize: 11
                }

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    width: coverageFilesView.width - 110
                    text: coverageDelegate.path
                    color: Theme.textSecondary
                    font.family: Theme.monoFont
                    font.pixelSize: 11
                    elide: Text.ElideLeft
                }

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    text: qsTr("%1/%2").arg(coverageDelegate.linesCovered)
                                       .arg(coverageDelegate.linesTotal)
                    color: Theme.textMuted
                    font.family: Theme.monoFont
                    font.pixelSize: 11
                }
            }
        }
    }
}
