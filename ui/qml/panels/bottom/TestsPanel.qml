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


        // B2 (docs/24): barra de rolagem. `parent: testCasesView` é OBRIGATÓRIO — um filho
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
        anchors.bottom: parent.bottom
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
}
