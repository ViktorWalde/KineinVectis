pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// One view for the CMake selection already owned by EmbeddedFlashController.
Column {
    id: root
    property var controller: null
    property string description: qsTr("Escolha o executável; a IDE grava o ELF que o CMake declara para ele.")
    readonly property var flash: controller ? controller.flash : null
    readonly property bool confirmed: flash !== null && flash.targetsOrigin === "fileApi"
    spacing: Theme.spacingSmall

    Text {
        width: parent.width
        text: qsTr("Alvo do CMake")
        color: Theme.textSecondary
        font.pixelSize: Theme.fontSizeSmall
        font.bold: true
    }

    Text {
        width: parent.width
        wrapMode: Text.WordWrap
        visible: text !== ""
        text: !root.flash ? ""
            : root.flash.targetsBusy ? qsTr("Lendo os alvos do build configurado…")
            : root.flash.targetsError !== "" ? root.flash.targetsError
            : root.flash.targetsOrigin === "" ? ""
            : !root.confirmed ? qsTr("O CMake ainda não confirmou os alvos deste projeto. Configure o projeto para escolher o executável que será gravado.")
            : root.flash.targets.length === 0 ? qsTr("O build configurado não tem alvo executável.")
            : root.description
        color: root.flash && root.flash.targetsError !== "" ? Theme.warningSoft : Theme.textMuted
        font.pixelSize: Theme.fontSizeCaption
    }

    Flow {
        width: parent.width
        spacing: Theme.spacingSmall

        Repeater {
            model: root.flash ? root.flash.targets : []

            KvToggleChip {
                required property var modelData

                objectName: "flashTarget_" + modelData.name
                labelText: modelData.name
                codeFont: false
                height: 28
                active: root.flash && root.flash.target === modelData.name
                tooltip: (modelData.artifacts || []).join("\n")
                onToggled: root.flash.selectTarget(modelData.name)
            }
        }

        KvButton {
            objectName: "flashConfigureCmake"
            visible: root.flash !== null && !root.flash.targetsBusy && root.flash.targetsOrigin !== "" && !root.confirmed
            text: qsTr("Configurar CMake")
            compact: true
            onClicked: root.controller.commandRequested("cmake.configure")
        }

        KvButton {
            objectName: "flashRefreshTargets"
            text: qsTr("Atualizar alvos")
            compact: true
            enabled: root.flash !== null && !root.flash.targetsBusy
            onClicked: root.flash.requestTargets()
        }
    }

}
