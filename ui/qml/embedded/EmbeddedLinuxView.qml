pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Acesso aos donos de SDK, terminal e SSH; nenhuma implantação implícita.
Column {
    id: root
    property var controller: null
    spacing: Theme.spacingMedium
    Text {
        width: parent.width
        text: qsTr("Linux embarcado")
        color: Theme.textPrimary
        font.pixelSize: Theme.fontSizeLarge
        font.bold: true
    }
    Text {
        width: parent.width
        text: qsTr("Prepare o SDK para compilar sua aplicação e use um alvo SSH para enviar, executar e depurar. A imagem do sistema segue o fluxo do seu projeto Yocto ou Buildroot.")
        color: Theme.textSecondary
        font.pixelSize: Theme.fontSizeSmall
        wrapMode: Text.WordWrap
    }
    Flow {
        width: parent.width
        spacing: Theme.spacingSmall
        KvButton {
            objectName: "embeddedLinuxRemote"
            text: qsTr("Abrir alvos SSH")
            primary: true
            enabled: root.controller !== null
            onClicked: root.controller.commandRequested("remote.list")
        }
        KvButton {
            text: qsTr("Terminal do projeto")
            enabled: root.controller !== null
            onClicked: root.controller.commandRequested("terminal.open")
        }
        KvButton {
            text: qsTr("Revisar SDK e sysroot")
            enabled: root.controller !== null
            onClicked: root.controller.tab = "kit"
        }
    }
    Text {
        width: parent.width
        text: qsTr("O console UART é opcional e fica em Conexões. Configure o destino e as tarefas de implantação no painel Remoto.")
        color: Theme.textMuted
        font.pixelSize: Theme.fontSizeCaption
        wrapMode: Text.WordWrap
    }
}
