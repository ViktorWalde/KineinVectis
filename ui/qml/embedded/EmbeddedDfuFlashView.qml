pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

Column {
    id: root
    property var controller: null
    property var flash: null
    visible: flash !== null && flash.dfuDestination
    spacing: Theme.spacingSmall
    KvToggleChip {
        objectName: "embeddedDfuBinaryMode"
        labelText: qsTr("BIN conferido com alvo CMake")
        active: root.flash !== null && root.flash.dfu.binaryMode
        onToggled: root.flash.dfu.selectBinary(!active)
    }
    EmbeddedFlashTargetView {
        width: parent.width
        visible: root.flash !== null && root.flash.dfu.binaryMode
        controller: root.controller
        description: qsTr("Escolha o executável cujo ELF será comparado com o BIN. A prévia mostra o endereço de carga conferido.")
    }
    KvTextField {
        objectName: "embeddedDfuImage"
        width: parent.width
        label: root.flash && root.flash.dfu.binaryMode ? qsTr("Arquivo BIN no build do alvo") : qsTr("Arquivo DfuSe no projeto")
        placeholder: root.flash && root.flash.dfu.binaryMode ? qsTr(".kinein/build/firmware.bin") : qsTr("firmware.dfu ou caminho dentro do projeto")
        text: root.flash ? root.flash.dfu.image : ""
        onEdited: value => root.flash.dfu.setImage(value)
    }
    Text {
        width: parent.width
        text: root.flash && root.flash.dfu.binaryMode
            ? qsTr("O BIN deve corresponder ao ELF do alvo escolhido. Configure e compile o projeto antes; gerar a prévia não recompila nem converte arquivos. Entre em DFU e escolha a alternativa USB.")
            : qsTr("Escolha um arquivo DfuSe com os endereços da aplicação. Entre em DFU, consulte USB e escolha a alternativa. A prévia confere o conteúdo e a memória anunciada antes de habilitar a gravação.")
        wrapMode: Text.WordWrap
        color: Theme.textMuted
        font.pixelSize: Theme.fontSizeCaption
    }
    EmbeddedDfuView {
        width: parent.width
        controller: root.flash ? root.flash.dfu.discovery : null
        selection: root.flash ? root.flash.dfu : null
    }
}
