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
    readonly property var adapterOptions: root.toolchainController
        ? root.toolchainController.candidatesFor("debugAdapter") : []

    signal closeRequested()

    Column {
        id: coluna

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: rodape.top
        anchors.bottomMargin: Theme.spacingSmall
        spacing: Theme.spacingSmall

        Text {
            width: parent.width
            text: qsTr("Embarcados")
            color: Theme.textPrimary
            font.pixelSize: 13
            font.bold: true
        }

        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            // O QUE A TELA PROMETE E' O QUE ELA FAZ: detectar pelo probe-rs,
            // guardar o kit, subir o depurador escolhido. Nada roda como root.
            text: qsTr("O projeto é lido pelos seus marcadores, a sonda pelo probe-rs e as portas pelo sysfs, sem abrir nenhuma; o chip, o alvo e o depurador ficam no kit. Nada roda como root.")
            color: Theme.textMuted
            font.pixelSize: 10
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

        Text {
            id: semSonda

            width: parent.width
            wrapMode: Text.WordWrap
            visible: root.controller && !root.controller.probeFound
            // A ferramenta ausente e' outro estado: a dica do core ja' diz como
            // instalar, e a tela nao repete.
            text: root.controller
                  ? (root.controller.busy ? qsTr("procurando…")
                     : (root.controller.toolAvailable ? qsTr("nenhuma sonda reconhecida")
                                                      : qsTr("probe-rs não encontrado nesta máquina")))
                  : ""
            color: Theme.textSecondary
            font.pixelSize: 11
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

        // --- Alvo do kit ---------------------------------------------------
        Text {
            text: qsTr("Alvo do kit %1").arg(root.preset)
            color: Theme.textSecondary
            font.pixelSize: 11
            font.bold: true
        }

        EmbeddedKitField {
            id: campoChip

            labelText: qsTr("Chip")
            placeholder: qsTr("como no `probe-rs chip list`, ex.: STM32F401CC")
            value: root.toolchainController ? root.toolchainController.chip : ""
        }

        EmbeddedKitField {
            id: campoAlvo

            labelText: qsTr("Alvo")
            placeholder: qsTr("triple, ex.: thumbv7em-none-eabihf")
            value: root.toolchainController ? root.toolchainController.targetTriple : ""
        }

        EmbeddedKitField {
            id: campoSysroot

            labelText: qsTr("Sysroot")
            placeholder: qsTr("raiz do sistema alvo (CMAKE_SYSROOT)")
            value: root.toolchainController ? root.toolchainController.sysroot : ""
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

        // --- Depurador -----------------------------------------------------
        Text {
            text: qsTr("Depurador do kit: %1").arg(
                      root.toolchainController ? root.toolchainController.labelFor("debugAdapter") : "")
            color: Theme.textSecondary
            font.pixelSize: 11
            font.bold: true
        }

        Flow {
            width: parent.width
            spacing: Theme.spacingXSmall

            Repeater {
                model: root.adapterOptions

                KvToggleChip {
                    id: chipAdaptador

                    required property var modelData

                    labelText: String(chipAdaptador.modelData.label)
                    active: {
                        const selecao = root.toolchainController.selectionFor("debugAdapter");
                        return selecao !== null && selecao.id === chipAdaptador.modelData.id;
                    }
                    onToggled: root.toolchainController.choose("debugAdapter",
                                                               chipAdaptador.modelData.id)
                }
            }
        }
    }

    Row {
        id: rodape

        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        spacing: Theme.spacingSmall
        layoutDirection: Qt.RightToLeft

        KvButton {
            text: qsTr("Fechar")
            compact: true
            onClicked: root.closeRequested()
        }

        KvButton {
            text: qsTr("Aplicar ao kit")
            primary: true
            compact: true
            enabled: root.toolchainController !== null
            // Os tres campos viajam juntos: string vazia LIMPA no core, e e'
            // isso que a tela mostra — campo vazio e' kit sem aquele valor.
            onClicked: root.toolchainController.applyKit(campoSysroot.text, campoAlvo.text,
                                                         campoChip.text)
        }

        KvButton {
            text: qsTr("Procurar sonda e portas")
            compact: true
            enabled: root.controller !== null && !root.controller.busy
            onClicked: root.controller.refresh()
        }
    }
}
