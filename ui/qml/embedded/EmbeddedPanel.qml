pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Janela integrada: navegação fixa, uma área de conteúdo rolável por aba.
Item {
    id: root

    property var controller: null
    property var toolchainController: null
    property bool ready: false
    readonly property bool linuxTarget: controller !== null && controller.projectLinux
    property bool environmentLoaded: false
    readonly property string tab: root.controller ? root.controller.tab : "project"
    signal closeRequested()

    Component.onCompleted: root.ready = true
    onTabChanged: {
        body.contentY = 0;
        if (root.tab === "kit") root.environmentLoaded = true;
        if (root.ready) contentTransition.restart();
    }
    Connections {
        target: root.controller
        function onWorkspaceRootChanged() { root.environmentLoaded = false; }
    }
    Connections {
        target: root.controller ? root.controller.access : null
        function onChannelsChanged() {
            if (root.tab === "board" && root.controller.access.focusedDevice !== ""
                && root.controller.access.diagnosed) {
                Qt.callLater(function() {
                    body.contentY = Math.min(accessView.mapToItem(content, 0, 0).y,
                                            Math.max(0, body.contentHeight - body.height));
                });
            }
        }
    }
    NumberAnimation {
        id: contentTransition
        target: content
        property: "opacity"
        from: 0
        to: 1
        duration: Theme.motionFast
        easing.type: Theme.easingStandard
    }

    KvPanelHeader {
        id: header
        anchors.top: parent.top
        width: parent.width
        title: qsTr("Embarcados")
        subtitle: root.linuxTarget ? qsTr("SDK e alvo Linux por SSH") : root.controller ? root.controller.boardVerdict : ""
        primaryLabel: qsTr("Atualizar")
        primaryIcon: "refresh"
        primaryAccent: false
        primaryEnabled: root.controller !== null
        primaryBusy: root.controller !== null && root.controller.busy
        onPrimaryRequested: root.controller.refresh()
        onCloseRequested: root.closeRequested()
    }

    KvSegmentedControl {
        id: navigation
        objectName: "embeddedNavigation"
        anchors.top: header.bottom
        anchors.topMargin: Theme.spacingMedium
        width: parent.width
        current: root.tab
        options: [
            { value: "project", label: qsTr("Visão geral") },
            { value: "board", label: root.linuxTarget ? qsTr("Conexões") : qsTr("Placa") },
            { value: "flash", label: root.linuxTarget ? qsTr("Implantar") : qsTr("Gravar") },
            { value: "kit", label: qsTr("Ambiente") }
        ]
        onSelected: value => root.controller.tab = value
    }

    Flickable {
        id: body
        objectName: "embeddedBody"
        anchors.top: navigation.bottom
        anchors.topMargin: Theme.spacingMedium
        anchors.bottom: footer.top
        anchors.bottomMargin: Theme.spacingSmall
        width: parent.width
        contentWidth: width
        contentHeight: content.implicitHeight
        clip: true
        boundsBehavior: Flickable.StopAtBounds

        Column {
            id: content
            width: body.width - Theme.spacingSmall
            spacing: Theme.spacingMedium

            KvVerdict {
                objectName: "embeddedOperationError"
                width: parent.width
                heading: qsTr("Não foi possível concluir a operação")
                message: root.controller ? root.controller.errorText : ""
                codeFont: false
                animateChanges: true
            }

            EmbeddedOverviewView {
                width: parent.width
                visible: root.tab === "project"
                controller: root.controller
            }

            Column {
                width: parent.width
                visible: root.tab === "board"
                spacing: Theme.spacingMedium
                EmbeddedSerialView { width: parent.width; controller: root.controller }
                EmbeddedIdentityView {
                    width: parent.width
                    controller: root.controller
                    kitChip: root.toolchainController ? root.toolchainController.chip : ""
                    visible: !root.linuxTarget && controller !== null && controller.identity.device !== ""
                }
                EmbeddedAccessView { id: accessView; width: parent.width; controller: root.controller }
                EmbeddedFilesView { width: parent.width; controller: root.controller }
            }

            EmbeddedFlashView {
                width: parent.width
                visible: root.tab === "flash" && !root.linuxTarget
                controller: root.controller
                firmwares: root.toolchainController ? root.toolchainController.installedFirmwares() : []
            }

            Loader {
                width: parent.width
                active: root.tab === "flash" && root.linuxTarget
                sourceComponent: EmbeddedLinuxView { controller: root.controller }
            }

            Loader {
                id: environmentLoader
                objectName: "embeddedEnvironmentLoader"
                width: parent.width
                visible: root.tab === "kit"
                active: root.environmentLoaded || root.tab === "kit"
                sourceComponent: EmbeddedEnvironmentView {
                    controller: root.controller
                    toolchainController: root.toolchainController
                }
            }
        }
    }

    FlickableScrollBar { view: body }

    Item {
        id: footer
        anchors.bottom: parent.bottom
        width: parent.width
        height: visible ? applyButton.implicitHeight : 0
        visible: root.tab === "kit"

        KvButton {
            id: applyButton
            anchors.right: parent.right
            text: qsTr("Salvar kit")
            primary: true
            compact: true
            enabled: root.toolchainController !== null && environmentLoader.item !== null
            onClicked: (environmentLoader.item as EmbeddedEnvironmentView).applyKit()
        }
    }
}
