pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

Item {
    id: root

    property int documentCount: 0
    property string documentName: ""
    property bool saving: false
    property string errorText: ""
    property real maxAvailableWidth: 520

    signal saveRequested()
    signal discardRequested()
    signal cancelRequested()

    function detailText() {
        if (documentCount === 1) {
            return qsTr("O arquivo \"%1\" tem alterações não salvas.")
                .arg(documentName);
        }
        return qsTr("%1 arquivos têm alterações não salvas.")
            .arg(documentCount);
    }

    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        onClicked: {
            if (!root.saving) {
                root.cancelRequested();
            }
        }
    }

    Rectangle {
        anchors.centerIn: parent
        width: Math.min(500, root.maxAvailableWidth)
        height: dialogColumn.height + 2 * Theme.spacingLarge
        radius: Theme.radiusLarge
        color: Theme.background2
        border.color: Theme.borderStrong
        border.width: 1

        MouseArea {
            anchors.fill: parent
        }

        Column {
            id: dialogColumn

            anchors.centerIn: parent
            width: parent.width - 2 * Theme.spacingLarge
            spacing: Theme.spacingMedium

            Text {
                width: parent.width
                text: qsTr("Salvar alterações?")
                color: Theme.textPrimary
                font.pixelSize: 14
                font.bold: true
            }

            Text {
                width: parent.width
                text: root.detailText()
                color: Theme.textSecondary
                font.pixelSize: 12
                wrapMode: Text.WordWrap
            }

            Text {
                width: parent.width
                visible: root.saving
                text: qsTr("Salvando...")
                color: Theme.accent
                font.pixelSize: 11
            }

            Text {
                width: parent.width
                visible: root.errorText !== ""
                text: root.errorText
                color: Theme.errorSoft
                font.pixelSize: 11
                wrapMode: Text.WordWrap
            }

            Row {
                anchors.right: parent.right
                spacing: Theme.spacingSmall

                Rectangle {
                    width: cancelLabel.width + 2 * Theme.spacingMedium
                    height: 28
                    radius: Theme.radius
                    opacity: root.saving ? 0.45 : 1.0
                    color: cancelArea.containsMouse && !root.saving
                           ? Theme.surface2 : Theme.surface1
                    border.color: Theme.borderSoft
                    border.width: 1

                    Text {
                        id: cancelLabel

                        anchors.centerIn: parent
                        text: qsTr("Cancelar")
                        color: Theme.textPrimary
                        font.pixelSize: 12
                    }

                    MouseArea {
                        id: cancelArea

                        anchors.fill: parent
                        enabled: !root.saving
                        hoverEnabled: enabled
                        cursorShape: enabled ? Qt.PointingHandCursor : Qt.ArrowCursor
                        onClicked: root.cancelRequested()
                    }
                }

                Rectangle {
                    width: discardLabel.width + 2 * Theme.spacingMedium
                    height: 28
                    radius: Theme.radius
                    opacity: root.saving ? 0.45 : 1.0
                    color: discardArea.containsMouse && !root.saving
                           ? Theme.surface2 : Theme.surface1
                    border.color: Theme.errorSoft
                    border.width: 1

                    Text {
                        id: discardLabel

                        anchors.centerIn: parent
                        text: root.documentCount > 1
                              ? qsTr("Descartar tudo") : qsTr("Descartar")
                        color: Theme.errorSoft
                        font.pixelSize: 12
                    }

                    MouseArea {
                        id: discardArea

                        anchors.fill: parent
                        enabled: !root.saving
                        hoverEnabled: enabled
                        cursorShape: enabled ? Qt.PointingHandCursor : Qt.ArrowCursor
                        onClicked: root.discardRequested()
                    }
                }

                Rectangle {
                    width: saveLabel.width + 2 * Theme.spacingMedium
                    height: 28
                    radius: Theme.radius
                    opacity: root.saving ? 0.45 : 1.0
                    color: saveArea.pressed && !root.saving
                           ? Theme.accentDim : Theme.accent

                    Text {
                        id: saveLabel

                        anchors.centerIn: parent
                        text: root.documentCount > 1
                              ? qsTr("Salvar tudo") : qsTr("Salvar")
                        color: Theme.background0
                        font.pixelSize: 12
                        font.bold: true
                    }

                    MouseArea {
                        id: saveArea

                        anchors.fill: parent
                        enabled: !root.saving
                        cursorShape: enabled ? Qt.PointingHandCursor : Qt.ArrowCursor
                        onClicked: root.saveRequested()
                    }
                }
            }
        }
    }

    Keys.onEscapePressed: {
        if (!root.saving) {
            root.cancelRequested();
        }
    }

    onVisibleChanged: {
        if (visible) {
            forceActiveFocus();
        }
    }
}
