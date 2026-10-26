pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

Column {
    id: root
    property var flash: null
    visible: flash !== null && flash.dfuDestination
    spacing: Theme.spacingSmall
    KvTextField {
        objectName: "embeddedDfuImage"
        width: parent.width
        label: qsTr("Arquivo DfuSe no projeto")
        placeholder: qsTr("firmware.dfu ou caminho dentro do projeto")
        text: root.flash ? root.flash.dfu.image : ""
        onEdited: value => root.flash.dfu.setImage(value)
    }
    Text {
        width: parent.width
        text: qsTr("Escolha um arquivo DfuSe com os endereços da aplicação. Entre em DFU, consulte USB e escolha a alternativa. A prévia confere o conteúdo e a memória anunciada antes de habilitar a gravação.")
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
