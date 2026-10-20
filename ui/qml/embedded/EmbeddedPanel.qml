pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Janela integrada: navegação fixa, uma área de conteúdo rolável por aba.
Item {
    id: root

    property var controller: null
    property var toolchainController: null
    property string expandedRole: ""
    property bool advancedKitVisible: false
    readonly property string preset: root.toolchainController && root.toolchainController.preset !== ""
                                     ? root.toolchainController.preset : qsTr("padrão")
    readonly property string tab: root.controller ? root.controller.tab : "project"
    signal closeRequested()

    onTabChanged: body.contentY = 0

    KvPanelHeader {
        id: header
        anchors.top: parent.top
        width: parent.width
        title: qsTr("Embarcados")
        subtitle: root.controller ? root.controller.boardVerdict : ""
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
            { value: "board", label: qsTr("Placa") },
            { value: "flash", label: qsTr("Gravar") },
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
                width: parent.width
                message: root.controller ? root.controller.errorText : ""
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
                EmbeddedIdentityView { width: parent.width; controller: root.controller }
                EmbeddedAccessView { width: parent.width; controller: root.controller }
                EmbeddedFilesView { width: parent.width; controller: root.controller }
            }

            EmbeddedFlashView {
                width: parent.width
                visible: root.tab === "flash"
                controller: root.controller
                firmwares: root.toolchainController ? root.toolchainController.installedFirmwares() : []
            }

            Column {
                width: parent.width
                visible: root.tab === "kit"
                spacing: Theme.spacingMedium
                Repeater {
                    model: root.toolchainController ? [
                        { key: "cCompiler", label: qsTr("Compilador C") },
                        { key: "cxxCompiler", label: qsTr("Compilador C++") },
                        { key: "serialMonitor", label: qsTr("Monitor serial") }
                    ] : []
                    ToolchainRoleRow {
                        required property var modelData
                        width: parent.width
                        controller: root.toolchainController
                        roleKey: modelData.key
                        roleLabel: modelData.label
                        expanded: root.expandedRole === modelData.key
                        onToggleRequested: root.expandedRole = expanded ? "" : modelData.key
                    }
                }
                EmbeddedKitView {
                    id: kitView

                    width: parent.width
                    preset: root.preset
                    toolchainController: root.toolchainController
                    advancedVisible: root.advancedKitVisible
                    onAdvancedToggleRequested: root.advancedKitVisible = !root.advancedKitVisible
                }

                Text {
                    width: parent.width
                    wrapMode: Text.WordWrap
                    visible: text !== ""
                    text: root.toolchainController ? root.toolchainController.errorText : ""
                    color: Theme.errorSoft
                    font.pixelSize: Theme.fontSizeCaption
                }

                // O que o kit ainda nao tem, dito com o remedio (integracoes/39).
                Text {
                    width: parent.width
                    wrapMode: Text.WordWrap
                    visible: text !== ""
                    text: root.toolchainController ? root.toolchainController.sysrootHint : ""
                    color: Theme.warningSoft
                    font.pixelSize: Theme.fontSizeCaption
                }

                Text {
                    width: parent.width
                    wrapMode: Text.WordWrap
                    visible: text !== ""
                    text: root.toolchainController ? root.toolchainController.rustTargetHint() : ""
                    color: Theme.warningSoft
                    font.family: Theme.monoFont
                    font.pixelSize: Theme.fontSizeCaption
                }

                Column {
                    width: parent.width
                    spacing: Theme.spacingMedium
                    visible: root.advancedKitVisible
                    EmbeddedProbeView { width: parent.width; controller: root.controller }
                    EmbeddedAdapterView { width: parent.width; toolchainController: root.toolchainController }
                    EmbeddedInstallView { width: parent.width; toolchainController: root.toolchainController }
                    EmbeddedKitImportView { width: parent.width; toolchainController: root.toolchainController }
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
            enabled: root.toolchainController !== null
            onClicked: root.toolchainController.applyKit(kitView.sysroot, kitView.targetTriple,
                                                         kitView.chip, undefined, kitView.svdFile)
        }
    }
}
