import QtQuick

Item {
    id: root
    property real menuX: 0
    property real menuY: 0
    property bool runnableScript: false
    property bool debuggableScript: false
    property int selectionCount: 1
    property bool pasteAvailable: false
    signal dismissRequested()
    signal createFileRequested()
    signal createDirectoryRequested()
    signal runScriptRequested()
    signal debugScriptRequested()
    signal renameRequested()
    signal deleteRequested()
    signal copyRequested()
    signal cutRequested()
    signal pasteRequested()
    signal copyAbsolutePathRequested()
    signal copyRelativePathRequested()
    signal openFolderRequested()
    signal openTerminalRequested()
    focus: visible
    onVisibleChanged: {
        if (visible) {
            menuKeys.currentAction = 0;
            forceActiveFocus();
        }
    }
    Keys.onPressed: function(event) { event.accepted = menuKeys.handleKey(event); }

    ProjectEntryMenuKeyboard {
        id: menuKeys
        runnableScript: root.runnableScript
        debuggableScript: root.debuggableScript
        singleSelection: root.selectionCount <= 1
        fileSelectionAvailable: root.selectionCount >= 1 && root.selectionCount <= 128
        pasteAvailable: root.pasteAvailable
        onDismissRequested: root.dismissRequested()
        onActionRequested: function(index) {
            switch (index) {
            case 0: root.createFileRequested(); break;
            case 1: root.createDirectoryRequested(); break;
            case 2: root.runScriptRequested(); break;
            case 3: root.debugScriptRequested(); break;
            case 4: root.renameRequested(); break;
            case 5: root.deleteRequested(); break;
            case 6: root.copyRequested(); break;
            case 7: root.cutRequested(); break;
            case 8: root.pasteRequested(); break;
            case 9: root.copyAbsolutePathRequested(); break;
            case 10: root.copyRelativePathRequested(); break;
            case 11: root.openFolderRequested(); break;
            case 12: root.openTerminalRequested(); break;
            }
        }
    }

    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        onClicked: root.dismissRequested()
    }

    Rectangle {
        x: root.menuX
        y: Math.max(0, Math.min(root.menuY, root.height - height))
        width: 240
        height: entryMenuColumn.height + 2 * Theme.spacingSmall
        radius: Theme.radius
        color: Theme.background2
        border.color: Theme.borderSoft
        border.width: 1

        Column {
            id: entryMenuColumn

            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.margins: Theme.spacingSmall
            spacing: 2

            Rectangle {
                width: parent.width
                height: 26
                radius: Theme.radius
                color: entryCreateFileHover.containsMouse
                       || (root.activeFocus && menuKeys.currentAction === 0)
                       ? Theme.surface2 : "transparent"

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.left: parent.left
                    anchors.leftMargin: Theme.spacingSmall
                    text: qsTr("Adicionar arquivo")
                    color: Theme.textPrimary
                    font.pixelSize: 12
                }

                MouseArea {
                    id: entryCreateFileHover

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.createFileRequested()
                }
            }

            Rectangle {
                width: parent.width
                height: 26
                radius: Theme.radius
                color: entryCreateDirectoryHover.containsMouse
                       || (root.activeFocus && menuKeys.currentAction === 1)
                       ? Theme.surface2 : "transparent"

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.left: parent.left
                    anchors.leftMargin: Theme.spacingSmall
                    text: qsTr("Adicionar pasta")
                    color: Theme.textPrimary
                    font.pixelSize: 12
                }

                MouseArea {
                    id: entryCreateDirectoryHover

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.createDirectoryRequested()
                }
            }

            Rectangle {
                width: parent.width
                height: 1
                color: Theme.borderSoft
            }

            Rectangle {
                width: parent.width
                height: root.runnableScript ? 26 : 0
                visible: root.runnableScript
                radius: Theme.radius
                color: entryRunHover.containsMouse
                       || (root.activeFocus && menuKeys.currentAction === 2)
                       ? Theme.surface2 : "transparent"

                Row {
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.left: parent.left
                    anchors.leftMargin: Theme.spacingSmall
                    spacing: Theme.spacingSmall

                    KvIcon {
                        anchors.verticalCenter: parent.verticalCenter
                        name: "run"
                        size: 14
                    }

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        text: qsTr("Executar script")
                        color: Theme.textPrimary
                        font.pixelSize: 12
                    }
                }

                MouseArea {
                    id: entryRunHover

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.runScriptRequested()
                }
            }

            Rectangle {
                width: parent.width
                height: root.debuggableScript ? 26 : 0
                visible: root.debuggableScript
                radius: Theme.radius
                color: entryDebugHover.containsMouse
                       || (root.activeFocus && menuKeys.currentAction === 3)
                       ? Theme.surface2 : "transparent"

                Row {
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.left: parent.left
                    anchors.leftMargin: Theme.spacingSmall
                    spacing: Theme.spacingSmall

                    KvIcon {
                        anchors.verticalCenter: parent.verticalCenter
                        name: "debug"
                        size: 14
                    }

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        text: qsTr("Depurar")
                        color: Theme.textPrimary
                        font.pixelSize: 12
                    }
                }

                MouseArea {
                    id: entryDebugHover

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.debugScriptRequested()
                }
            }

            Rectangle {
                width: parent.width
                height: root.runnableScript ? 1 : 0
                visible: root.runnableScript
                color: Theme.borderSoft
            }

            Rectangle {
                width: parent.width
                height: 26
                radius: Theme.radius
                color: entryRenameHover.containsMouse
                       || (root.activeFocus && menuKeys.currentAction === 4)
                       ? Theme.surface2 : "transparent"

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.left: parent.left
                    anchors.leftMargin: Theme.spacingSmall
                    text: root.selectionCount <= 1 ? qsTr("Renomear")
                                                     : qsTr("Renomear: selecione 1 item")
                    color: root.selectionCount <= 1 ? Theme.textPrimary : Theme.textMuted
                    font.pixelSize: 12
                }

                MouseArea {
                    id: entryRenameHover

                    anchors.fill: parent
                    hoverEnabled: true
                    enabled: root.selectionCount <= 1
                    cursorShape: enabled ? Qt.PointingHandCursor : Qt.ArrowCursor
                    onClicked: root.renameRequested()
                }
            }

            Rectangle {
                width: parent.width
                height: 26
                radius: Theme.radius
                color: entryDeleteHover.containsMouse
                       || (root.activeFocus && menuKeys.currentAction === 5)
                       ? Theme.surface2 : "transparent"

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.left: parent.left
                    anchors.leftMargin: Theme.spacingSmall
                    text: root.selectionCount <= 1 ? qsTr("Excluir")
                                                     : qsTr("Excluir: selecione 1 item")
                    color: root.selectionCount <= 1 ? Theme.errorSoft : Theme.textMuted
                    font.pixelSize: 12
                }

                MouseArea {
                    id: entryDeleteHover

                    anchors.fill: parent
                    hoverEnabled: true
                    enabled: root.selectionCount <= 1
                    cursorShape: enabled ? Qt.PointingHandCursor : Qt.ArrowCursor
                    onClicked: root.deleteRequested()
                }
            }
            Rectangle { width: parent.width; height: 1; color: Theme.borderSoft }
            ProjectEntryQuickActions {
                width: parent.width
                selectionCount: root.selectionCount
                pasteAvailable: root.pasteAvailable
                menuFocused: root.activeFocus
                currentAction: menuKeys.currentAction
                onActionRequested: index => menuKeys.actionRequested(index)
            }
        }
    }
}
