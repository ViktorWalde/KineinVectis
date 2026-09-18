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
    property var workspaceBuildSystems: []
    property string activeConfigId: ""
    property string activeConfigName: ""
    property bool configMenuOpen: false

    signal openWorkspaceRequested()
    signal buildRequested(string buildSystem)
    signal testsRequested(string buildSystem)
    signal qualityRequested()
    signal runRequested()
    signal stopRunRequested()
    signal debugRequested()
    signal stopDebugRequested()
    signal configureRequested()
    signal configMenuRequested(real menuX, real menuY)

    readonly property bool cargoAvailable: hasBuildSystem("cargo")
    readonly property bool cmakeAvailable: hasBuildSystem("cmake")
    readonly property bool hybridNativeWorkspace: cargoAvailable && cmakeAvailable

    function hasBuildSystem(buildSystem) {
        const systems = workspaceBuildSystems !== undefined
                && workspaceBuildSystems !== null ? workspaceBuildSystems : [];
        return systems.indexOf(buildSystem) >= 0;
    }

    function primaryBuildSystem() {
        if (workspaceKind === "rustCargo") return "cargo";
        if (workspaceKind === "cmake") return "cmake";
        return "";
    }

    function workspaceSystemLabel() {
        if (hybridNativeWorkspace) return "Cargo + CMake";
        if (cargoAvailable) return "Cargo";
        if (cmakeAvailable) return "CMake";
        return workspaceKind === "" ? qsTr("projeto") : workspaceKind;
    }

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
            visible: root.workspaceOpen && root.width >= 1100
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

            visible: root.workspaceOpen && root.width >= 900
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
            visible: root.workspaceOpen && root.width >= 900
            width: 1
            height: 24
            color: Theme.borderStrong
            anchors.verticalCenter: parent.verticalCenter
        }

        KvIconButton {
            visible: root.workspaceOpen && root.cmakeAvailable
            enabled: root.coreConnected
            iconName: "configure"
            tooltip: qsTr("Configurar CMake")
            onClicked: root.configureRequested()
        }

        KvButton {
            visible: root.workspaceOpen
            enabled: !root.building && root.coreConnected
            text: root.building ? qsTr("Compilando...")
                  : (root.hybridNativeWorkspace ? "Cargo" : qsTr("Compilar"))
            iconName: "build"
            primary: true
            onClicked: root.buildRequested(root.hybridNativeWorkspace
                                           ? "cargo"
                                           : root.primaryBuildSystem())
        }

        KvIconButton {
            visible: root.workspaceOpen
            enabled: !root.testing && root.coreConnected
            iconName: "test"
            tooltip: root.testing ? qsTr("Testes em andamento")
                                  : (root.hybridNativeWorkspace
                                     ? qsTr("Executar testes Cargo")
                                     : qsTr("Executar testes"))
            onClicked: root.testsRequested(root.hybridNativeWorkspace
                                           ? "cargo"
                                           : root.primaryBuildSystem())
        }

        KvButton {
            visible: root.workspaceOpen && root.hybridNativeWorkspace
            enabled: !root.building && root.coreConnected
            text: "CMake"
            iconName: "build"
            onClicked: root.buildRequested("cmake")
        }

        KvIconButton {
            visible: root.workspaceOpen && root.hybridNativeWorkspace
            enabled: !root.testing && root.coreConnected
            iconName: "test"
            tooltip: qsTr("Executar testes CMake")
            onClicked: root.testsRequested("cmake")
        }

        // A analise: clippy (Cargo), ruff (Python), clang-tidy pela CDB
        // (CMake e Makefile, D6 de 2026-09-17). O que nao tem linter nao
        // ganha botao — a paleta continua dizendo o motivo.
        KvIconButton {
            visible: root.workspaceOpen
                     && (root.cargoAvailable || root.cmakeAvailable
                         || root.hasBuildSystem("python") || root.hasBuildSystem("make"))
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
        visible: root.workspaceOpen && root.width >= 1050

        Rectangle {
            width: 7
            height: 7
            radius: 3.5
            color: root.coreConnected ? Theme.successSoft : Theme.errorSoft
            anchors.verticalCenter: parent.verticalCenter
        }

        Text {
            anchors.verticalCenter: parent.verticalCenter
            text: root.workspaceSystemLabel()
            color: Theme.textMuted
            font.pixelSize: 11
        }
    }
}
