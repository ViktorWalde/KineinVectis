pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Revisa nomes e inclusao antes de enviar um lote ao core. O resultado por item
// volta ao mesmo dialogo para corrigir uma falha parcial sem repetir sucessos.
Item {
    id: root

    property var entries: []
    property var transferController: null
    property var draftEntries: []
    property string destinationDisplayPath: ""
    property string errorText: ""
    property bool cut: false
    property bool importing: false
    property bool pending: false
    property real maxAvailableWidth: 560

    signal confirmRequested(var entries)
    signal cancelRequested()
    signal pendingDismissRequested()

    focus: visible
    onVisibleChanged: {
        if (visible) forceActiveFocus();
    }

    onEntriesChanged: {
        draftEntries = entries.map(entry => ({
            from: entry.from, name: entry.name, included: entry.included,
            status: entry.status, error: entry.error
        }));
    }

    Keys.onEscapePressed: {
        if (root.pending) root.pendingDismissRequested();
        else root.cancelRequested();
    }

    KvBackdrop { anchors.fill: parent }

    Rectangle {
        anchors.centerIn: parent
        width: Math.min(root.maxAvailableWidth, 560)
        height: contentColumn.height + 2 * Theme.spacingMedium
        radius: Theme.radius
        color: Theme.background2
        border.color: Theme.accent
        border.width: 1

        Column {
            id: contentColumn
            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.margins: Theme.spacingMedium
            spacing: Theme.spacingSmall

            Text {
                text: root.importing ? qsTr("Importar itens")
                      : root.cut ? qsTr("Mover itens") : qsTr("Copiar itens")
                color: Theme.textPrimary
                font.pixelSize: Theme.fontSizeMedium
                font.bold: true
            }

            Text {
                width: parent.width
                text: qsTr("Destino: ") + root.destinationDisplayPath
                color: Theme.textSecondary
                font.pixelSize: Theme.fontSizeSmall
                elide: Text.ElideMiddle
            }

            Text {
                width: parent.width
                text: qsTr("Ajuste o nome ou desmarque um item para pulá-lo. "
                           + "O lote não desfaz itens já concluídos.")
                color: Theme.textMuted
                font.pixelSize: Theme.fontSizeCaption
                wrapMode: Text.WordWrap
            }

            Flickable {
                width: parent.width
                height: Math.min(280, entryColumn.height)
                contentHeight: entryColumn.height
                clip: true

                Column {
                    id: entryColumn
                    width: parent.width
                    spacing: 2

                    Repeater {
                        model: root.draftEntries

                        delegate: Rectangle {
                            id: entryRow
                            required property int index
                            required property var modelData
                            property bool included: modelData.included

                            width: entryColumn.width
                            height: 45
                            radius: Theme.radius
                            color: Theme.background1

                            Rectangle {
                                id: inclusionBox
                                anchors.left: parent.left
                                anchors.leftMargin: Theme.spacingSmall
                                anchors.verticalCenter: parent.verticalCenter
                                width: 17
                                height: 17
                                radius: Theme.radiusXSmall
                                color: entryRow.included ? Theme.accent : Theme.background0
                                border.color: Theme.accent
                                border.width: 1

                                Text {
                                    anchors.centerIn: parent
                                    text: entryRow.included ? "✓" : ""
                                    color: Theme.background0
                                    font.pixelSize: Theme.fontSizeBody
                                }
                                MouseArea {
                                    anchors.fill: parent
                                    enabled: !root.pending
                                             && !root.transferController.batchItemSucceeded(entryRow.modelData.status)
                                    onClicked: {
                                        entryRow.included = !entryRow.included;
                                        root.draftEntries[entryRow.index].included = entryRow.included;
                                    }
                                }
                            }

                            Column {
                                anchors.left: inclusionBox.right
                                anchors.leftMargin: Theme.spacingSmall
                                anchors.right: parent.right
                                anchors.rightMargin: Theme.spacingSmall
                                anchors.verticalCenter: parent.verticalCenter
                                spacing: 1

                                KvTextField {
                                    width: parent.width
                                    height: 22
                                    text: entryRow.modelData.name
                                    enabled: !root.pending
                                             && !root.transferController.batchItemSucceeded(entryRow.modelData.status)
                                    pixelSize: Theme.fontSizeSmall
                                    onEdited: (text) => { root.draftEntries[entryRow.index].name = text; }
                                }
                                Text {
                                    width: parent.width
                                    text: entryRow.modelData.error !== ""
                                          ? entryRow.modelData.error
                                          : root.transferController.batchItemSucceeded(entryRow.modelData.status)
                                            ? qsTr("Concluído") : entryRow.modelData.from
                                    color: entryRow.modelData.error !== ""
                                           ? Theme.errorSoft : Theme.textMuted
                                    font.pixelSize: Theme.fontSizeMicro
                                    elide: Text.ElideMiddle
                                }
                            }
                        }
                    }
                }
            }

            Text {
                width: parent.width
                visible: root.pending
                text: qsTr("Transferindo… acompanhe ou cancele em Jobs.")
                color: Theme.textSecondary
                font.pixelSize: Theme.fontSizeCaption
            }

            Text {
                width: parent.width
                visible: root.errorText !== ""
                text: root.errorText
                color: Theme.errorSoft
                font.pixelSize: Theme.fontSizeCaption
                wrapMode: Text.WordWrap
            }

            Row {
                anchors.right: parent.right
                spacing: Theme.spacingSmall

                KvButton {
                    text: root.pending ? qsTr("Ver em Jobs") : qsTr("Cancelar")
                    onClicked: {
                        if (root.pending) root.pendingDismissRequested();
                        else root.cancelRequested();
                    }
                }
                KvButton {
                    text: root.importing ? qsTr("Importar")
                          : root.cut ? qsTr("Mover") : qsTr("Copiar")
                    enabled: !root.pending
                    primary: true
                    onClicked: root.confirmRequested(root.draftEntries)
                }
            }
        }
    }
}
