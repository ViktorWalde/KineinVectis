pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Formulário do kit: criado no primeiro acesso, conservado entre abas.
Column {
    id: root
    property var controller: null
    property var toolchainController: null
    property string expandedRole: ""
    property bool advancedKitVisible: false
    readonly property bool linuxTarget: controller !== null && controller.projectLinux
    readonly property bool managedToolchain: controller !== null
        && (controller.projectPlatformIo || controller.projectManagedLinux)
    readonly property string preset: toolchainController && toolchainController.preset !== ""
                                     ? toolchainController.preset : qsTr("padrão")
    function applyKit() {
        if (root.toolchainController) root.toolchainController.applyKit(kitView.sysroot,
            kitView.targetTriple, kitView.chip, undefined, kitView.svdFile);
    }
    width: parent.width
    spacing: Theme.spacingMedium
    Text {
        width: parent.width
        visible: root.managedToolchain
        text: root.controller && root.controller.projectPlatformIo
            ? qsTr("O PlatformIO gerencia os compiladores e SDKs pelos ambientes do platformio.ini. As opções avançadas abaixo pertencem ao kit da IDE.")
            : qsTr("%1 gerencia a toolchain da imagem. O kit abaixo atende às aplicações do alvo Linux; importe o SDK correspondente para configurar compiladores, alvo e sysroot.").arg(root.controller ? root.controller.projectFrameworkLabel : "")
        color: Theme.textSecondary
        font.pixelSize: Theme.fontSizeSmall
        wrapMode: Text.WordWrap
    }
    Repeater {
        model: root.toolchainController ? [
            { key: "cCompiler", label: qsTr("Compilador C") },
            { key: "cxxCompiler", label: qsTr("Compilador C++") },
            { key: "serialMonitor", label: qsTr("Monitor serial") }
        ].filter(role => !root.managedToolchain || role.key === "serialMonitor") : []
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
        linuxTarget: root.linuxTarget
        platformIo: root.controller !== null && root.controller.projectPlatformIo
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
        objectName: "embeddedRustTargetHint"
        width: parent.width
        wrapMode: Text.WordWrap
        visible: text !== ""
        text: root.toolchainController && root.controller && root.controller.projectFrameworks.length > 0
            && root.controller.projectFrameworks[0].framework === "cargoEmbedded"
                ? root.toolchainController.rustTargetHint() : ""
        color: Theme.warningSoft
        font.family: Theme.monoFont
        font.pixelSize: Theme.fontSizeCaption
    }

    EmbeddedKitImportView {
        width: parent.width
        visible: root.linuxTarget || root.advancedKitVisible
        toolchainController: root.toolchainController
    }
    Column {
        width: parent.width
        spacing: Theme.spacingMedium
        visible: root.advancedKitVisible
        EmbeddedProbeView { width: parent.width; visible: !root.linuxTarget; controller: root.controller }
        EmbeddedAdapterView { width: parent.width; toolchainController: root.toolchainController }
        EmbeddedInstallView { width: parent.width; toolchainController: root.toolchainController }
    }

}
