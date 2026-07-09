pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

ListView {
    id: panel

    ListModel {
        id: emptyJobsModel
    }

    property var jobsModel: emptyJobsModel

    function statusColor(status) {
        if (status === "success") {
            return Theme.successSoft;
        }
        if (status === "failed" || status === "cancelled") {
            return Theme.errorSoft;
        }
        if (status === "warning" || status === "cancelRequested") {
            return Theme.warningSoft;
        }
        return Theme.infoSoft;
    }

    function statusLabel(status) {
        const labels = {
            queued: qsTr("na fila"),
            running: qsTr("rodando"),
            cancelRequested: qsTr("cancelando"),
            success: qsTr("sucesso"),
            warning: qsTr("aviso"),
            failed: qsTr("falhou"),
            cancelled: qsTr("cancelado")
        };
        return labels[status] !== undefined ? labels[status] : status;
    }

    clip: true
    spacing: 2
    model: panel.jobsModel

    Text {
        anchors.centerIn: parent
        visible: panel.jobsModel.count === 0
        text: qsTr("Nenhum job recente.")
        color: Theme.textMuted
        font.pixelSize: 11
    }

    delegate: Rectangle {
        id: jobDelegate

        required property string jobId
        required property string kind
        required property string title
        required property string status
        required property real progress
        required property bool canCancel
        required property string risk
        required property string latestLine
        required property int outputCount

        width: panel.width
        height: 34
        radius: Theme.radius
        color: jobArea.containsMouse ? Theme.surface2 : "transparent"

        Row {
            id: jobRow

            anchors.left: parent.left
            anchors.right: parent.right
            anchors.leftMargin: Theme.spacingSmall
            anchors.rightMargin: Theme.spacingSmall
            anchors.verticalCenter: parent.verticalCenter
            spacing: Theme.spacingMedium

            Rectangle {
                width: 8
                height: 8
                radius: 4
                anchors.verticalCenter: parent.verticalCenter
                color: panel.statusColor(jobDelegate.status)
            }

            Column {
                anchors.verticalCenter: parent.verticalCenter
                width: Math.max(120, parent.width - statusText.width
                                - riskText.width - 42)
                spacing: 1

                Text {
                    width: parent.width
                    text: jobDelegate.title
                    color: Theme.textPrimary
                    font.pixelSize: 11
                    font.bold: true
                    elide: Text.ElideRight
                }

                Text {
                    width: parent.width
                    text: jobDelegate.latestLine !== ""
                          ? jobDelegate.latestLine
                          : jobDelegate.kind + " · " + jobDelegate.jobId
                    color: Theme.textMuted
                    font.family: Theme.monoFont
                    font.pixelSize: 10
                    elide: Text.ElideRight
                }
            }

            Text {
                id: statusText

                anchors.verticalCenter: parent.verticalCenter
                text: panel.statusLabel(jobDelegate.status)
                color: panel.statusColor(jobDelegate.status)
                font.pixelSize: 10
                font.bold: true
            }

            Text {
                id: riskText

                anchors.verticalCenter: parent.verticalCenter
                visible: jobDelegate.risk !== "low"
                text: jobDelegate.risk
                color: Theme.warningSoft
                font.pixelSize: 10
                font.family: Theme.monoFont
            }
        }

        MouseArea {
            id: jobArea

            anchors.fill: parent
            hoverEnabled: true
        }
    }
}
