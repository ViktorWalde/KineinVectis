pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Os campos do ALVO do kit: chip, triple, sysroot e — desde o P3
// (2026-09-17) — o CMSIS-SVD do chip, que vai no launch do probe-rs e vira
// o escopo "Peripherals" no depurador. Dono proprio pela regra do painel
// (bateu em 300 linhas). Burro: mostra o que o ToolchainController tem e
// expoe o que foi digitado; quem grava e' o "Aplicar ao kit" do painel.
Column {
    id: root

    property string preset: ""
    property var toolchainController: null

    readonly property string chip: campoChip.text
    readonly property string targetTriple: campoAlvo.text
    readonly property string sysroot: campoSysroot.text
    readonly property string svdFile: campoSvd.text

    spacing: Theme.spacingSmall

    Text {
        text: qsTr("Alvo do kit %1").arg(root.preset)
        color: Theme.textSecondary
        font.pixelSize: 11
        font.bold: true
    }

    // O seletor de preset (pente-fino 2026-09-18): os configure presets do
    // CMakePresets/CMakeUserPresets do projeto; escolher um torna o kit
    // dele o ativo, e o configure automatico passa a usa-lo. So' aparece
    // quando o projeto tem presets.
    Flow {
        width: parent.width
        spacing: Theme.spacingXSmall
        visible: root.toolchainController && root.toolchainController.presets.length > 0

        KvToggleChip {
            labelText: qsTr("padrão")
            active: root.toolchainController ? root.toolchainController.preset === "" : true
            onToggled: root.toolchainController.selectPreset("")
        }

        Repeater {
            model: root.toolchainController ? root.toolchainController.presets : []

            delegate: KvToggleChip {
                required property var modelData

                labelText: modelData.displayName ? modelData.displayName : modelData.name
                active: root.toolchainController ? root.toolchainController.preset === modelData.name : false
                onToggled: root.toolchainController.selectPreset(modelData.name)
            }
        }
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

    // O SVD: o arquivo CMSIS-SVD do chip (do pack do fabricante ou do
    // cmsis-svd, cada um com a sua licenca — 35 §5.7); vazio = sem
    // periféricos no depurador.
    EmbeddedKitField {
        id: campoSvd

        labelText: qsTr("SVD")
        placeholder: qsTr("CMSIS-SVD do chip, ex.: ~/svd/esp32c3.svd (registradores de periférico no depurador)")
        value: root.toolchainController ? root.toolchainController.svdFile : ""
    }
}
