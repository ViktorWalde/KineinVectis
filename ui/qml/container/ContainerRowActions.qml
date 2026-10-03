pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// AS ACOES de uma linha de container (2026-10-03): a porta do host como
// etiqueta (rodando, abre no navegador) e os botoes — sempre a vista,
// discretos, acesos com o mouse na linha (`lit`); verde inicia, vermelho
// para/remove. Remover arma no primeiro clique e so' age no segundo, dentro
// do prazo. Ocupado (`busy`), nada se clica ate' o motor responder.
Row {
    id: root

    property var row: null
    property var controller: null
    property bool running: false
    property bool busy: false
    property bool lit: false
    property bool confirmRemove: false

    readonly property var hostPort: root.row !== null ? root.row.ports.find(p => p.host !== "") : undefined

    spacing: 2

    // O segundo clique do Remover tem prazo.
    Timer {
        interval: Theme.motionProgress * 4
        running: root.confirmRemove
        onTriggered: root.confirmRemove = false
    }


    // A porta do host: etiqueta; rodando, abre no navegador.
    Rectangle {
        anchors.verticalCenter: parent.verticalCenter
        visible: root.hostPort !== undefined
        width: portText.implicitWidth + 2 * Theme.spacingSmall
        height: 20
        radius: height / 2
        color: portArea.containsMouse && root.running ? Theme.surface2 : "transparent"
        border.width: 1
        border.color: root.running ? Theme.accentDim : Theme.borderSoft

        Text {
            id: portText

            anchors.centerIn: parent
            text: root.hostPort !== undefined ? ":" + root.hostPort.host + (root.running ? " ↗" : "") : ""
            color: root.running ? Theme.accent : Theme.textMuted
            font.family: Theme.monoFont
            font.pixelSize: Theme.fontSizeCaption
        }

        MouseArea {
            id: portArea

            anchors.fill: parent
            enabled: root.running && root.hostPort !== undefined && root.hostPort.url !== ""
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: Qt.openUrlExternally(root.hostPort.url)
            onContainsMouseChanged: {
                if (containsMouse) TooltipController.showFor(parent, qsTr("Abrir %1").arg(root.hostPort.url), "bottom");
                else TooltipController.hideFor(parent);
            }
        }
    }

    // As acoes: sempre a vista, discretas; acendem com o mouse na linha.
    Row {
        anchors.verticalCenter: parent.verticalCenter
        spacing: 0
        opacity: root.lit ? 1 : 0.55
        enabled: !root.busy

        Behavior on opacity {
            NumberAnimation { duration: Theme.motionFast }
        }

        KvIconButton {
            compact: true
            iconName: root.running ? "stop" : "run"
            iconSize: 14
            danger: root.running
            success: !root.running
            tooltip: root.running ? qsTr("Parar") : qsTr("Iniciar")
            onClicked: root.controller.act(root.row.target, root.running ? "stop" : "start")
        }

        KvIconButton {
            compact: true
            iconName: "documents"
            iconSize: 14
            enabled: root.controller !== null && root.controller.canOpenTerminals
            tooltip: enabled ? qsTr("Logs (numa aba do terminal)") : qsTr("Logs: abra um projeto — a aba de terminal é do projeto")
            onClicked: root.controller.openLogs(root.row.target)
        }

        KvIconButton {
            compact: true
            visible: root.running
            iconName: "terminal"
            iconSize: 14
            enabled: root.controller !== null && root.controller.canOpenTerminals
            tooltip: qsTr("Shell dentro do container")
            onClicked: root.controller.openShell(root.row.target)
        }

        KvIconButton {
            compact: true
            visible: !root.running
            iconName: root.confirmRemove ? "check" : "trash"
            iconSize: 14
            danger: true
            active: root.confirmRemove
            tooltip: root.confirmRemove ? qsTr("Clique de novo para remover %1").arg(root.row ? root.row.name : "")
                                        : qsTr("Remover (pede confirmação)")
            onClicked: {
                if (!root.confirmRemove) {
                    root.confirmRemove = true;
                    return;
                }
                root.confirmRemove = false;
                root.controller.act(root.row.target, "remove");
            }
        }
    }
}
