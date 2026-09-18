pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O painel de embarcados: a sonda que esta' no USB, o alvo do kit e o
// depurador que o kit vai subir.
//
// Burro de proposito, como o `GrafanaPanel`: recebe estado e emite pedidos.
// A sonda vem do `EmbeddedController`; chip, alvo, sysroot e depurador sao
// KIT e vem do `ToolchainController` — existiam no protocolo desde
// 2026-09-03 e nenhuma tela os mostrava (roadmaps/35 §5.7, fatia 1).
Item {
    id: root

    property var controller: null
    property var toolchainController: null

    readonly property string preset: root.toolchainController && root.toolchainController.preset !== ""
                                    ? root.toolchainController.preset : qsTr("padrão")
    signal closeRequested()

    // A altura e' a do conteudo: a moldura comum (KvPanelFrame) ROLA o que
    // nao couber — antes o painel vazava da moldura a 800 px (foto 12c).
    implicitHeight: coluna.implicitHeight + Theme.spacingSmall + rodape.implicitHeight

    Column {
        id: coluna

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        spacing: Theme.spacingSmall

        // A primeira linha comum dos paineis de ambiente (F8): titulo, uma
        // linha, a acao primaria — "Procurar sonda e portas" — e o x.
        KvPanelHeader {
            width: parent.width
            title: qsTr("Embarcados")
            // O QUE A TELA PROMETE E' O QUE ELA FAZ: detectar pelo probe-rs,
            // guardar o kit, subir o depurador escolhido. Nada roda como root.
            subtitle: qsTr("O projeto é lido pelos seus marcadores, a sonda pelo probe-rs e as portas pelo sysfs, sem abrir nenhuma; o chip, o alvo e o depurador ficam no kit. Nada roda como root.")
            primaryLabel: qsTr("Procurar sonda e portas")
            primaryIcon: "refresh"
            primaryEnabled: root.controller !== null
            primaryBusy: root.controller !== null && root.controller.busy
            onPrimaryRequested: root.controller.refresh()
            onCloseRequested: root.closeRequested()
        }

        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            visible: text !== ""
            text: root.controller ? root.controller.errorText : ""
            color: Theme.errorSoft
            font.pixelSize: 10
        }

        // O MODELO do projeto: dono proprio (pilar 0 do roadmaps/42).
        EmbeddedProjectView {
            width: parent.width
            controller: root.controller
        }

        // --- Sonda ---------------------------------------------------------
        Text {
            text: qsTr("Sonda")
            color: Theme.textSecondary
            font.pixelSize: 11
            font.bold: true
        }

        Repeater {
            model: root.controller ? root.controller.probes : []

            Text {
                id: linhaSonda

                required property var modelData

                width: coluna.width
                text: "● " + root.controller.probeSummary(linhaSonda.modelData)
                color: Theme.textPrimary
                font.family: Theme.monoFont
                font.pixelSize: 11
                elide: Text.ElideMiddle
            }
        }

        // O veredito comum (F8): a busca em andamento, ou o que ela achou.
        // A ferramenta ausente e' outro estado: a dica do core ja' diz como
        // instalar, e a tela nao repete.
        KvVerdict {
            id: vereditoSonda

            width: parent.width
            visible: root.controller && !root.controller.probeFound
                     && (vereditoSonda.busy || vereditoSonda.showsBand)
            busy: root.controller !== null && root.controller.busy
            busyText: qsTr("procurando…")
            neutral: root.controller !== null && root.controller.toolAvailable
            message: root.controller
                     ? (root.controller.toolAvailable ? qsTr("nenhuma sonda reconhecida")
                                                      : qsTr("probe-rs não encontrado nesta máquina"))
                     : ""
        }

        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            visible: root.controller && root.controller.hint !== "" && !root.controller.busy
            text: root.controller ? root.controller.hint : ""
            color: Theme.textMuted
            font.pixelSize: 10
        }

        // A saida CRUA aparece quando nada foi reconhecido: e' o que deixa o
        // usuario ver se ha' uma sonda ali e o formato mudou.
        Rectangle {
            id: caixaCrua

            width: parent.width
            height: Math.min(96, saidaCrua.contentHeight + 2 * Theme.spacingSmall)
            visible: root.controller && !root.controller.probeFound
                     && root.controller.rawOutput.trim() !== "" && !root.controller.busy
            radius: Theme.radius
            color: Theme.background0
            border.width: 1
            border.color: Theme.borderSoft
            clip: true

            Flickable {
                anchors.fill: parent
                anchors.margins: Theme.spacingSmall
                contentHeight: saidaCrua.contentHeight
                clip: true

                Text {
                    id: saidaCrua

                    width: parent.width
                    wrapMode: Text.WrapAnywhere
                    text: root.controller ? root.controller.rawOutput.trim() : ""
                    color: Theme.textMuted
                    font.family: Theme.monoFont
                    font.pixelSize: 10
                }
            }
        }

        // As portas seriais: dono proprio (E1 do integracoes/38 §6).
        EmbeddedSerialView {
            width: parent.width
            controller: root.controller
        }

        EmbeddedAccessView {
            width: parent.width
            controller: root.controller
        }

        EmbeddedIdentityView {
            width: parent.width
            controller: root.controller
        }

        EmbeddedFlashView {
            width: parent.width
            controller: root.controller
            firmwares: root.toolchainController ? root.toolchainController.installedFirmwares() : []
        }

        EmbeddedFilesView {
            width: parent.width
            controller: root.controller
        }

        // --- Alvo do kit: dono proprio (saiu daqui em 2026-09-17, P3, quando
        // o SVD entrou e o painel bateu em 300) --------------------------------
        EmbeddedKitView {
            id: kit

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

        // O que o kit ainda nao tem, dito com o remedio (integracoes/39): o
        // sysroot do compilador cross de distro, o alvo Rust por instalar.
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

        // O tamanho do binario: dono proprio (a catraca cobrou em 2026-09-11).
        EmbeddedSizeView {
            width: parent.width
            controller: root.controller
        }

        // O depurador do kit: dono proprio (saiu daqui em 2026-09-13).
        EmbeddedAdapterView {
            width: parent.width
            toolchainController: root.toolchainController
        }

        // O provedor de instalacao de toolchain (integracoes/39 §5).
        EmbeddedInstallView {
            width: parent.width
            toolchainController: root.toolchainController
        }

        // Ler o sysroot e importar kit de SDK (42 §8 itens b e d).
        EmbeddedKitImportView {
            width: parent.width
            toolchainController: root.toolchainController
        }
    }

    Row {
        id: rodape

        anchors.top: coluna.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.left: parent.left
        anchors.right: parent.right
        spacing: Theme.spacingSmall
        layoutDirection: Qt.RightToLeft

        KvButton {
            text: qsTr("Aplicar ao kit")
            primary: true
            compact: true
            enabled: root.toolchainController !== null
            // Os campos viajam juntos: string vazia LIMPA no core, e e' isso
            // que a tela mostra — campo vazio e' kit sem aquele valor.
            onClicked: root.toolchainController.applyKit(kit.sysroot, kit.targetTriple,
                                                         kit.chip, undefined, kit.svdFile)
        }

    }
}
