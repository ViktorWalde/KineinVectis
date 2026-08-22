pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Cluster de toolbar da App Bar unica (F2, 2026-07-18): alvo, perfil e fluxo
// Configure -> Build -> Run/Debug. Deixou de ser uma BARRA propria — e um
// Item transparente de largura implicita, ancorado a direita da barra unica
// pelo ShellHeaderHost; o fundo e da barra (background0).
Item {
    id: root

    // Largura da BARRA (nao do cluster): decide o que cabe em tela estreita.
    property real hostWidth: 0
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
    // L3 fatia 2: o alvo de build ativo. Vazio = todos os alvos.
    property string activeTargetLabel: ""
    property bool targetMenuOpen: false
    property bool targetsAvailable: false
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
    signal targetMenuRequested(real menuX, real menuY)

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

    implicitWidth: toolbarRow.implicitWidth
    implicitHeight: 46

    Row {
        id: toolbarRow

        anchors.verticalCenter: parent.verticalCenter
        anchors.left: parent.left
        spacing: Theme.spacingSmall

        KvButton {
            visible: !root.workspaceOpen
            text: qsTr("Abrir workspace")
            iconName: "project"
            primary: true
            onClicked: root.openWorkspaceRequested()
        }

        Rectangle {
            visible: root.workspaceOpen && root.hostWidth >= 1100
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

            visible: root.workspaceOpen && root.hostWidth >= 900
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

        // Alvo de build. So aparece quando ha alvos DETECTADOS: um seletor
        // vazio ocuparia a barra prometendo uma escolha que nao existe, que e
        // a "propaganda de rail" da §12.2 em outra roupa.
        KvButton {
            id: targetSelector

            visible: root.workspaceOpen && root.targetsAvailable && root.hostWidth >= 1040
            selected: root.targetMenuOpen
            iconName: "chevron-down"
            text: root.activeTargetLabel
            onClicked: {
                const pos = root.mapFromItem(targetSelector, 0,
                                             targetSelector.height + 4);
                root.targetMenuRequested(pos.x, pos.y);
            }
        }

        Rectangle {
            visible: root.workspaceOpen && root.hostWidth >= 900
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
            // F4 (§7.2): o ambar e identidade, nao decoracao — na toolbar so
            // o Run e primario, como na JetBrains; Compilar vira botao comum.
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

        KvIconButton {
            // L2: analise cobre Cargo (clippy) e C/C++ (Cppcheck).
            visible: root.workspaceOpen
                     && (root.cargoAvailable || root.cmakeAvailable)
            enabled: !root.analyzing && root.coreConnected
            iconName: "problems"
            tooltip: root.analyzing ? qsTr("Análise em andamento") : qsTr("Executar análise estática")
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

        // Estado da conexao + build system fecham o cluster (era o Row
        // ancorado a direita da barra antiga).
        Rectangle {
            visible: root.workspaceOpen && root.hostWidth >= 1050
            width: 7
            height: 7
            radius: 3.5
            color: root.coreConnected ? Theme.successSoft : Theme.errorSoft
            anchors.verticalCenter: parent.verticalCenter
        }

        Text {
            visible: root.workspaceOpen && root.hostWidth >= 1050
            anchors.verticalCenter: parent.verticalCenter
            text: root.workspaceSystemLabel()
            color: Theme.textMuted
            font.pixelSize: 11
        }
    }
}
