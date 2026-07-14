pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Main Toolbar da spec: alvo, perfil e fluxo Configure → Build → Run/Debug.
Rectangle {
    id: root

    property bool workspaceOpen: false
    property bool coreConnected: false
    property bool building: false
    property bool testing: false
    property bool analyzing: false
    property bool running: false
    property bool debugging: false
    property string workspaceKind: ""
    property string activeConfigId: ""
    property string activeConfigName: ""
    property bool configMenuOpen: false

    signal openWorkspaceRequested()
    signal buildRequested()
    signal testsRequested()
    signal qualityRequested()
    signal runRequested()
    signal stopRunRequested()
    signal debugRequested()
    signal stopDebugRequested()
    signal configureRequested()
    signal configMenuRequested(real menuX, real menuY)

    height: 44
    color: Theme.background1
    border.color: Theme.borderSoft
    border.width: 1

    Row {
        id: toolbarRow

        anchors.verticalCenter: parent.verticalCenter
        anchors.left: parent.left
        anchors.leftMargin: Theme.spacingMedium
        spacing: Theme.spacingSmall

        KvButton {
            visible: !root.workspaceOpen
            text: qsTr("Abrir workspace")
            iconName: "project"
            primary: true
            onClicked: root.openWorkspaceRequested()
        }

        Rectangle {
            visible: root.workspaceOpen
            height: 32
            width: targetText.implicitWidth + 16 + Theme.spacingSmall
                   + 2 * Theme.spacingMedium
            radius: Theme.radius
            color: Theme.surface1
            border.color: Theme.borderSoft
            border.width: 1

            Row {
                anchors.centerIn: parent
                spacing: Theme.spacingSmall

                KvIcon {
                    anchors.verticalCenter: parent.verticalCenter
                    name: "tools"
                    size: 16
                }

                Text {
                    id: targetText

                    anchors.verticalCenter: parent.verticalCenter
                    text: qsTr("Target: host local")
                    color: Theme.textSecondary
                    font.pixelSize: 12
                }
            }
        }

        KvButton {
            id: configSelector

            visible: root.workspaceOpen
            selected: root.configMenuOpen
            iconName: "chevron-down"
            text: root.activeConfigId === ""
                  ? (root.workspaceKind === "rustCargo"
                     ? qsTr("Cargo: debug") : qsTr("Perfil: automático"))
                  : root.activeConfigName
            onClicked: {
                const pos = root.mapFromItem(configSelector, 0,
                                             configSelector.height + 4);
                root.configMenuRequested(pos.x, pos.y);
            }
        }

        Rectangle {
            visible: root.workspaceOpen
            width: 1
            height: 24
            color: Theme.borderStrong
            anchors.verticalCenter: parent.verticalCenter
        }

        KvIconButton {
            visible: root.workspaceOpen && root.workspaceKind === "cmake"
            enabled: root.coreConnected
            iconName: "configure"
            tooltip: qsTr("Configurar CMake")
            onClicked: root.configureRequested()
        }

        KvButton {
            visible: root.workspaceOpen
            enabled: !root.building && root.coreConnected
            text: root.building ? qsTr("Compilando...") : qsTr("Compilar")
            iconName: "build"
            primary: true
            onClicked: root.buildRequested()
        }

        KvIconButton {
            visible: root.workspaceOpen
            enabled: !root.testing && root.coreConnected
            iconName: "test"
            tooltip: root.testing ? qsTr("Testes em andamento") : qsTr("Executar testes")
            onClicked: root.testsRequested()
        }

        KvIconButton {
            visible: root.workspaceOpen
            enabled: !root.analyzing && root.coreConnected
            iconName: "problems"
            tooltip: root.analyzing ? qsTr("Análise em andamento") : qsTr("Executar análise")
            onClicked: root.qualityRequested()
        }

        KvIconButton {
            visible: root.workspaceOpen
            enabled: root.coreConnected
            active: root.debugging
            danger: root.debugging
            iconName: root.debugging ? "stop" : "debug"
            tooltip: root.debugging ? qsTr("Parar debug") : qsTr("Iniciar debug")
            onClicked: root.debugging ? root.stopDebugRequested() : root.debugRequested()
        }

        KvIconButton {
            visible: root.workspaceOpen
            enabled: root.coreConnected
            primary: true
            danger: root.running
            iconName: root.running ? "stop" : "run"
            tooltip: root.running ? qsTr("Parar execução") : qsTr("Executar configuração ativa")
            onClicked: root.running ? root.stopRunRequested() : root.runRequested()
        }
    }

    Row {
        anchors.right: parent.right
        anchors.rightMargin: Theme.spacingMedium
        anchors.verticalCenter: parent.verticalCenter
        spacing: Theme.spacingSmall
        visible: root.workspaceOpen

        Rectangle {
            width: 7
            height: 7
            radius: 3.5
            color: root.coreConnected ? Theme.successSoft : Theme.errorSoft
            anchors.verticalCenter: parent.verticalCenter
        }

        Text {
            anchors.verticalCenter: parent.verticalCenter
            text: root.workspaceKind === "" ? qsTr("projeto") : root.workspaceKind
            color: Theme.textMuted
            font.pixelSize: 11
        }
    }
}
