pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Recovery is supplied by the core. Instructions remain visible; technical
// evidence is optional. A terminal command runs only through an explicit click.
Item {
    id: root
    property var controller: null
    readonly property var access: controller ? controller.access : null
    implicitHeight: content.implicitHeight
    visible: access !== null && (access.diagnosed || access.busy || access.errorText !== "")

    Column {
        id: content
        width: parent.width
        spacing: Theme.spacingMedium

        Flow {
            width: parent.width
            spacing: Theme.spacingSmall
            Text {
                text: root.access && root.access.focusedDevice !== ""
                    ? qsTr("Preparação · %1").arg(root.access.focusedDevice) : qsTr("Acesso aos dispositivos")
                color: Theme.textPrimary
                font.pixelSize: Theme.fontSizeMedium
                font.weight: Font.DemiBold
                width: Math.min(implicitWidth, content.width)
                wrapMode: Text.WrapAnywhere
            }
            KvButton {
                objectName: "embeddedRecheckAccess"
                text: qsTr("Verificar novamente")
                iconName: "refresh"
                compact: true
                enabled: root.access !== null && !root.access.busy && !root.controller.portsBusy
                onClicked: root.access.recheck()
            }
        }

        KvVerdict {
            width: parent.width
            busy: root.access !== null && root.access.busy
            busyText: qsTr("Conferindo o acesso desta sessão…")
            heading: qsTr("Não foi possível verificar o acesso")
            message: root.access ? root.access.errorText : ""
            codeFont: false
            animateChanges: true
        }

        Repeater {
            model: root.access ? root.access.visibleChannels : []
            Column {
                id: channel
                required property var modelData
                property bool detailsVisible: false
                width: content.width
                spacing: Theme.spacingSmall

                Text {
                    width: parent.width
                    wrapMode: Text.WordWrap
                    text: (channel.modelData.ok ? qsTr("Verificação concluída · ") : qsTr("Preparação necessária · "))
                          + root.access.channelTitle(channel.modelData)
                    color: channel.modelData.ok ? Theme.successSoft : Theme.warningSoft
                    font.pixelSize: Theme.fontSizeSmall
                    font.weight: Font.DemiBold
                }
                Text {
                    width: parent.width
                    wrapMode: Text.WordWrap
                    // The focused serial diagnosis is already on the port card.
                    visible: text !== "" && !(root.access.focusedDevice !== "" && channel.modelData.kind === "serial")
                    text: channel.modelData.problem || ""
                    color: Theme.textSecondary
                    font.pixelSize: Theme.fontSizeBody
                }
                Text {
                    width: parent.width
                    wrapMode: Text.WordWrap
                    visible: text !== ""
                    text: channel.modelData.distroDidIt || ""
                    color: Theme.textSecondary
                    font.pixelSize: Theme.fontSizeSmall
                }

                Repeater {
                    model: root.access.stepsOf(channel.modelData)
                    Column {
                        id: step
                        required property var modelData
                        width: channel.width
                        spacing: Theme.spacingSmall
                        Text {
                            width: parent.width
                            wrapMode: Text.WordWrap
                            text: step.modelData.explanation
                            color: Theme.textPrimary
                            font.pixelSize: Theme.fontSizeBody
                        }
                        KvButton {
                            objectName: "embeddedAccessCommand"
                            visible: step.modelData.command !== ""
                            text: qsTr("Executar no terminal")
                            iconName: "terminal"
                            compact: true
                            tooltip: step.modelData.command
                            onClicked: root.access.runStep(step.modelData.command)
                        }
                        Text {
                            width: parent.width
                            visible: channel.detailsVisible && step.modelData.command !== ""
                            text: step.modelData.command
                            wrapMode: Text.WrapAnywhere
                            color: Theme.textSecondary
                            font.family: Theme.monoFont
                            font.pixelSize: Theme.fontSizeSmall
                        }
                    }
                }

                KvButton {
                    objectName: "embeddedAccessDetails"
                    text: channel.detailsVisible ? qsTr("Ocultar detalhes técnicos") : qsTr("Detalhes técnicos")
                    compact: true
                    onClicked: channel.detailsVisible = !channel.detailsVisible
                }
                Text {
                    width: parent.width
                    visible: channel.detailsVisible
                    text: channel.modelData.detail
                    wrapMode: Text.WordWrap
                    color: Theme.textMuted
                    font.family: Theme.monoFont
                    font.pixelSize: Theme.fontSizeCaption
                }
                Text {
                    width: parent.width
                    visible: channel.detailsVisible && channel.modelData.fix !== undefined
                    text: channel.modelData.fix
                        ? qsTr("Fonte: %1 · conferida em %2").arg(channel.modelData.fix.sourceUrl).arg(channel.modelData.fix.checkedOn) : ""
                    wrapMode: Text.WrapAnywhere
                    color: Theme.textMuted
                    font.pixelSize: Theme.fontSizeCaption
                }
                Rectangle { width: parent.width; height: 1; color: Theme.borderSoft }
            }
        }
    }
}
