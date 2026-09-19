pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O painel de embarcados em ABAS (Etapa 3, E3-5 — roadmaps/44 §4): o
// cabecalho comum com o veredito da placa, e quatro abas que cabem sem
// rolar a 800 px — Placa (portas, sonda, identidade, permissoes, arquivos
// na placa) · Projeto (framework, modelo, tamanho) · Gravar (motor,
// previa, firmware) · Kit (chip/alvo/sysroot/SVD, depurador, instalar,
// importar). Antes eram nove secoes numa coluna que passava da janela.
//
// Burro de proposito: recebe estado e emite pedidos. A sonda e a placa vem
// do EmbeddedController; chip, alvo, sysroot e depurador sao KIT e vem do
// ToolchainController. Nada roda como root.
Item {
    id: root

    property var controller: null
    property var toolchainController: null

    readonly property string preset: root.toolchainController && root.toolchainController.preset !== ""
                                    ? root.toolchainController.preset : qsTr("padrão")
    readonly property string tab: root.controller ? root.controller.tab : "board"

    signal closeRequested()

    implicitHeight: cabecalho.implicitHeight + abas.height + conteudo.implicitHeight
                    + rodape.height + 3 * Theme.spacingSmall

    KvPanelHeader {
        id: cabecalho

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        title: qsTr("Embarcados")
        subtitle: root.controller ? root.controller.boardVerdict : ""
        primaryLabel: qsTr("Procurar sonda e portas")
        primaryIcon: "refresh"
        primaryEnabled: root.controller !== null
        primaryBusy: root.controller !== null && root.controller.busy
        onPrimaryRequested: root.controller.refresh()
        onCloseRequested: root.closeRequested()
    }

    // As quatro abas.
    Row {
        id: abas

        anchors.top: cabecalho.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.left: parent.left
        height: 24
        spacing: Theme.spacingXSmall

        Repeater {
            model: [
                { key: "board", label: qsTr("Placa") },
                { key: "project", label: qsTr("Projeto") },
                { key: "flash", label: qsTr("Gravar") },
                { key: "kit", label: qsTr("Kit") }
            ]

            KvToggleChip {
                id: aba

                required property var modelData

                anchors.verticalCenter: parent.verticalCenter
                labelText: aba.modelData.label
                active: root.tab === aba.modelData.key
                onToggled: root.controller.tab = aba.modelData.key
            }
        }
    }

    Text {
        id: erro

        anchors.top: abas.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        height: visible ? implicitHeight : 0
        wrapMode: Text.WordWrap
        visible: text !== ""
        text: root.controller ? root.controller.errorText : ""
        color: Theme.errorSoft
        font.pixelSize: 10
    }

    Item {
        id: conteudo

        anchors.top: erro.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.left: parent.left
        anchors.right: parent.right
        implicitHeight: Math.max(placa.visible ? placa.implicitHeight : 0,
                                 projeto.visible ? projeto.implicitHeight : 0,
                                 gravar.visible ? gravar.implicitHeight : 0,
                                 kit.visible ? kit.implicitHeight : 0)

        // --- Placa: o que esta' no USB e o que ela e' -----------------------
        Column {
            id: placa

            width: parent.width
            visible: root.tab === "board"
            spacing: Theme.spacingSmall

            EmbeddedSerialView { width: parent.width; controller: root.controller }
            EmbeddedIdentityView { width: parent.width; controller: root.controller }
            EmbeddedAccessView { width: parent.width; controller: root.controller }
            EmbeddedProbeView { width: parent.width; controller: root.controller }
            EmbeddedFilesView { width: parent.width; controller: root.controller }
        }

        // --- Projeto: o modelo e o tamanho do binario -----------------------
        Column {
            id: projeto

            width: parent.width
            visible: root.tab === "project"
            spacing: Theme.spacingSmall

            EmbeddedProjectView { width: parent.width; controller: root.controller }
            EmbeddedSizeView { width: parent.width; controller: root.controller }
        }

        // --- Gravar: o motor, a previa e o firmware -------------------------
        Column {
            id: gravar

            width: parent.width
            visible: root.tab === "flash"
            spacing: Theme.spacingSmall

            EmbeddedFlashView {
                width: parent.width
                controller: root.controller
                firmwares: root.toolchainController ? root.toolchainController.installedFirmwares() : []
            }
        }

        // --- Kit: o alvo, o depurador, instalar e importar ------------------
        Column {
            id: kit

            width: parent.width
            visible: root.tab === "kit"
            spacing: Theme.spacingSmall

            EmbeddedKitView {
                id: kitView

                width: parent.width
                preset: root.preset
                toolchainController: root.toolchainController
            }

            Text {
                width: parent.width
                wrapMode: Text.WordWrap
                visible: text !== ""
                text: root.toolchainController ? root.toolchainController.errorText : ""
                color: Theme.errorSoft
                font.pixelSize: 10
            }

            // O que o kit ainda nao tem, dito com o remedio (integracoes/39).
            Text {
                width: parent.width
                wrapMode: Text.WordWrap
                visible: text !== ""
                text: root.toolchainController ? root.toolchainController.sysrootHint : ""
                color: Theme.warningSoft
                font.pixelSize: 10
            }

            Text {
                width: parent.width
                wrapMode: Text.WordWrap
                visible: text !== ""
                text: root.toolchainController ? root.toolchainController.rustTargetHint() : ""
                color: Theme.warningSoft
                font.family: Theme.monoFont
                font.pixelSize: 10
            }

            EmbeddedAdapterView { width: parent.width; toolchainController: root.toolchainController }
            EmbeddedInstallView { width: parent.width; toolchainController: root.toolchainController }
            EmbeddedKitImportView { width: parent.width; toolchainController: root.toolchainController }
        }
    }

    // O rodape so' da aba Kit: aplicar os campos.
    Row {
        id: rodape

        anchors.top: conteudo.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.left: parent.left
        anchors.right: parent.right
        height: visible ? 28 : 0
        visible: root.tab === "kit"
        spacing: Theme.spacingSmall
        layoutDirection: Qt.RightToLeft

        KvButton {
            text: qsTr("Aplicar ao kit")
            primary: true
            compact: true
            enabled: root.toolchainController !== null
            // Os campos viajam juntos: string vazia LIMPA no core, e e' isso
            // que a tela mostra — campo vazio e' kit sem aquele valor.
            onClicked: root.toolchainController.applyKit(kitView.sysroot, kitView.targetTriple,
                                                         kitView.chip, undefined, kitView.svdFile)
        }
    }
}
