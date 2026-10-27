pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

Column {
    id: root
    property var controller: null
    property var flash: null
    visible: flash !== null && flash.picoDestination
    spacing: Theme.spacingSmall
    EmbeddedFlashTargetView {
        width: parent.width
        controller: root.controller
        description: qsTr("Escolha o executável cujo ELF será comparado com o UF2. Configure e compile o projeto antes; a prévia não recompila nem converte arquivos.")
    }
    KvTextField {
        objectName: "embeddedPicoImage"
        width: parent.width
        label: qsTr("Arquivo UF2 no build do alvo")
        placeholder: qsTr(".kinein/build/firmware.uf2")
        text: root.flash ? root.flash.pico.image : ""
        onEdited: value => root.flash.pico.setImage(value)
    }
    Text {
        width: parent.width
        text: qsTr("Entre em BOOTSEL, consulte e escolha o RP2040. A prévia confere UF2, ELF e destino antes de gravar. Apaga setores de 4 KiB, verifica os dados e não inicia a aplicação.")
        wrapMode: Text.WordWrap
        color: Theme.textMuted
        font.pixelSize: Theme.fontSizeCaption
    }
    EmbeddedPicoView {
        width: parent.width
        controller: root.flash ? root.flash.pico.discovery : null
        selection: root.flash ? root.flash.pico : null
    }
}
