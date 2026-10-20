pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// As tarefas usam os donos de build, configuração, prévia e serial existentes.
Column {
    id: root
    property var controller: null
    readonly property var info: controller ? controller.projectPlatformIoInfo || ({}) : ({})
    readonly property bool available: controller !== null
        && controller.projectSdks.some(sdk => sdk.id === "pio" && sdk.found)
    spacing: Theme.spacingMedium

    KvInsetSurface {
        width: parent.width
        implicitHeight: tasks.implicitHeight + 2 * Theme.spacingMedium
        Column {
            id: tasks
            x: Theme.spacingMedium
            y: Theme.spacingMedium
            width: parent.width - 2 * Theme.spacingMedium
            spacing: Theme.spacingMedium
            Text {
                width: parent.width
                text: qsTr("PlatformIO · Tarefas do projeto")
                font.pixelSize: Theme.fontSizeLarge
                font.bold: true
                color: Theme.textPrimary
                wrapMode: Text.WordWrap
            }
            Text {
                width: parent.width
                text: root.info.detail ? qsTr("Ambientes declarados: %1").arg(root.info.detail)
                    : qsTr("Os ambientes e as opções vêm do platformio.ini.")
                font.pixelSize: Theme.fontSizeSmall
                color: Theme.textSecondary
                wrapMode: Text.WordWrap
            }
            Flow {
                width: parent.width
                spacing: Theme.spacingSmall
                KvButton {
                    objectName: "embeddedPlatformIoBuild"
                    text: root.available ? qsTr("Compilar projeto") : qsTr("Preparar PlatformIO")
                    iconName: "build"
                    primary: true
                    enabled: root.controller !== null && !root.controller.projectBusy
                    onClicked: root.controller.commandRequested(root.available ? "build.run" : "setup.list")
                }
                KvButton {
                    objectName: "embeddedPlatformIoUpload"
                    text: qsTr("Revisar upload")
                    enabled: root.available && !root.controller.projectBusy
                    onClicked: {
                        if (root.controller.flash.engine !== "platformio") root.controller.flash.selectEngine("platformio");
                        root.controller.tab = "flash";
                    }
                }
                KvButton {
                    text: qsTr("Conexão e monitor")
                    onClicked: root.controller.tab = "board"
                }
                KvButton {
                    text: qsTr("Configurar índice C/C++")
                    enabled: root.available
                    onClicked: root.controller.commandRequested("configAction.list")
                }
            }
            Text {
                width: parent.width
                text: qsTr("Compilar usa os ambientes padrão do projeto. O upload abre uma prévia para conferir o destino antes de executar.")
                font.pixelSize: Theme.fontSizeCaption
                color: Theme.textMuted
                wrapMode: Text.WordWrap
            }
        }
    }
}
