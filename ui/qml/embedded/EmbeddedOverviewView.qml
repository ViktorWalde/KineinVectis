pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Primeiro contato: fatos do modelo e próximos gestos nos donos existentes.
Column {
    id: root

    property var controller: null
    property bool detailsVisible: false
    readonly property bool recognized: controller !== null && controller.projectEmbedded
    readonly property var missingSdks: controller ? controller.projectSdks.filter(sdk => !sdk.found) : []
    readonly property string frameworkLabel: recognized
        ? controller.projectFrameworkLabel : ""
    readonly property string artifactLabel: controller ? controller.artifactsSummary(controller.projectArtifacts) : ""
    readonly property bool portSelected: controller !== null && controller.selectedPort !== ""
    readonly property bool microPython: controller !== null && controller.projectMicroPython
    readonly property bool platformIo: controller !== null && controller.projectPlatformIo
    readonly property bool linuxTarget: controller !== null && controller.projectLinux
    readonly property bool hasFirmware: controller !== null
        && (controller.projectArtifacts.flashRecipe !== undefined
            || ["elf", "bin", "hex", "uf2"].some(kind => (controller.projectArtifacts[kind] || []).length > 0))
    readonly property string nextStep: !recognized || missingSdks.length > 0 ? "environment"
        : !portSelected ? "board" : microPython ? "script" : "firmware"

    function activate(step) {
        if (!root.controller) return;
        if (step.command !== "") root.controller.commandRequested(step.command);
        else root.controller.tab = step.tab;
    }

    spacing: Theme.spacingMedium

    Text {
        width: parent.width
        text: root.recognized ? root.frameworkLabel : qsTr("Prepare seu projeto embarcado")
        color: Theme.textPrimary
        font.pixelSize: Theme.fontSizePanelTitle
        font.bold: true
        wrapMode: Text.WordWrap
    }

    Text {
        width: parent.width
        text: root.linuxTarget ? qsTr("SDK, aplicação e alvo Linux, em uma jornada própria.")
            : root.platformIo ? qsTr("Ambientes e tarefas do PlatformIO, junto ao projeto.")
            : root.microPython ? qsTr("Prepare a conexão e execute seus scripts na placa.")
            : qsTr("Prepare o ambiente, escolha a placa e revise o firmware antes de gravar.")
        color: Theme.textSecondary
        font.pixelSize: Theme.fontSizeSmall
        wrapMode: Text.WordWrap
    }

    Loader {
        width: parent.width
        active: root.platformIo
        sourceComponent: EmbeddedPlatformIoView { controller: root.controller }
    }
    Loader {
        width: parent.width
        active: root.linuxTarget
        sourceComponent: EmbeddedLinuxView { controller: root.controller }
    }

    Repeater {
        model: root.platformIo || root.linuxTarget ? [] : [
            { key: "environment", title: qsTr("1  Ambiente"),
              status: !root.recognized ? qsTr("Verificar") : root.missingSdks.length > 0 ? qsTr("Incompleto") : qsTr("Detectado"),
              detail: !root.recognized ? qsTr("Abra um projeto ESP-IDF, PlatformIO, Zephyr ou outro framework compatível.")
                : root.missingSdks.length > 0 ? qsTr("Ferramentas ausentes: %1").arg(root.missingSdks.map(sdk => sdk.label).join(", "))
                : qsTr("%1 reconhecido. Os compiladores e o chip podem ser revisados em Ambiente.").arg(root.frameworkLabel),
              action: !root.recognized || root.missingSdks.length > 0 ? qsTr("Preparar ambiente") : qsTr("Revisar ferramentas"),
              command: !root.recognized || root.missingSdks.length > 0 ? "setup.list" : "", tab: "kit", enabled: true },
            { key: "board", title: qsTr("2  Placa"),
              status: root.portSelected ? qsTr("Porta selecionada") : qsTr("Selecionar"),
              detail: root.portSelected ? root.controller.selectedPort
                : qsTr("Conecte a placa por USB. Escolha a porta e use Identificar para ler chip e memória."),
              action: root.portSelected ? qsTr("Ver conexão") : qsTr("Selecionar porta"), command: "", tab: "board", enabled: true },
            { key: root.microPython ? "script" : "firmware", title: root.microPython ? qsTr("3  Scripts") : qsTr("3  Firmware"),
              status: root.microPython ? qsTr("MicroPython") : root.hasFirmware ? qsTr("Build encontrado") : qsTr("Compilar"),
              detail: root.microPython ? qsTr("Com MicroPython instalado na placa, execute o arquivo .py pelo editor. O Monitor serial abre o REPL.")
                : root.hasFirmware ? (root.artifactLabel !== "" ? root.artifactLabel : qsTr("Arquivos de firmware encontrados no projeto."))
                : qsTr("A compilação produz os arquivos que serão gravados na placa."),
              action: root.microPython ? qsTr("Ir ao editor") : root.hasFirmware ? qsTr("Revisar gravação") : qsTr("Compilar firmware"),
              command: root.microPython ? "view.returnToEditor" : root.hasFirmware ? "" : "build.run", tab: "flash",
              enabled: root.recognized && root.missingSdks.length === 0 }
        ]

        Rectangle {
            id: stepCard
            required property var modelData

            width: root.width
            implicitHeight: stepContent.implicitHeight + 2 * Theme.spacingMedium
            radius: Theme.radiusLarge
            color: Theme.background0
            border.color: modelData.key === root.nextStep ? Theme.accentDim : Theme.borderSoft

            Column {
                id: stepContent
                x: Theme.spacingMedium
                y: Theme.spacingMedium
                width: parent.width - 2 * Theme.spacingMedium
                spacing: Theme.spacingSmall

                Text {
                    width: parent.width
                    text: stepCard.modelData.title + " · " + stepCard.modelData.status
                    color: Theme.textPrimary
                    font.pixelSize: Theme.fontSizeSmall
                    font.bold: true
                    wrapMode: Text.WordWrap
                }
                Text {
                    width: parent.width
                    text: stepCard.modelData.detail
                    color: Theme.textSecondary
                    font.pixelSize: Theme.fontSizeSmall
                    wrapMode: Text.WrapAnywhere
                }
                KvButton {
                    objectName: "embeddedStage_" + stepCard.modelData.key
                    text: stepCard.modelData.action
                    compact: true
                    primary: stepCard.modelData.key === root.nextStep
                    enabled: root.controller !== null && !root.controller.projectBusy && stepCard.modelData.enabled
                    onClicked: root.activate(stepCard.modelData)
                }
            }
        }
    }

    Flow {
        width: parent.width
        spacing: Theme.spacingSmall
        KvButton {
            text: qsTr("Compilar novamente")
            iconName: "build"
            visible: root.hasFirmware && !root.microPython && !root.platformIo && !root.linuxTarget
            enabled: root.recognized && root.missingSdks.length === 0 && !root.controller.projectBusy
            onClicked: root.controller.commandRequested("build.run")
        }
    }

    KvVerdict {
        width: parent.width
        busy: root.controller !== null && root.controller.projectBusy
        busyText: qsTr("Lendo o projeto e as ferramentas…")
        neutral: true
        message: busy ? qsTr("Lendo o projeto e as ferramentas…") : ""
        visible: busy
    }

    KvButton {
        text: root.detailsVisible ? qsTr("Ocultar detalhes do projeto") : qsTr("Detalhes do projeto")
        compact: true
        onClicked: root.detailsVisible = !root.detailsVisible
    }
    Loader {
        width: parent.width
        active: root.detailsVisible
        sourceComponent: Column {
            spacing: Theme.spacingMedium
            EmbeddedProjectView { width: parent.width; controller: root.controller }
            EmbeddedSizeView { width: parent.width; controller: root.controller }
        }
    }
}
