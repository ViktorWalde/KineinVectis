import QtQuick
import KineinVectis

// O widget de EXECUTAR (Etapa 2, F1): UMA configuracao ativa, Rodar,
// Depurar, e o menu "⋯" com Compilar/Testar/Analise/Cobertura/Configurar
// de cada sistema presente. E' o Run widget da referencia: os doze
// controles da barra antiga (dois sistemas de build lado a lado, icones
// sem rotulo) viram um seletor e tres botoes. O que esta' rodando
// (build, testes, analise) aparece como um ponto pulsando no "⋯", com o
// nome no tooltip — a barra de status (F2) diz o resto.
Row {
    id: root

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
    property bool actionsMenuOpen: false

    signal configMenuRequested(real menuX, real menuY)
    signal actionsMenuRequested(real menuX, real menuY)
    signal runRequested()
    signal stopRunRequested()
    signal debugRequested()
    signal stopDebugRequested()

    readonly property bool busy: building || testing || analyzing
    readonly property string busyLabel: building ? qsTr("Compilando…")
                                       : (testing ? qsTr("Testando…")
                                       : (analyzing ? qsTr("Analisando…") : ""))

    spacing: Theme.spacingXSmall

    KvButton {
        id: configSelector

        anchors.verticalCenter: parent.verticalCenter
        selected: root.configMenuOpen
        iconName: "chevron-down"
        text: root.activeConfigId === ""
              ? (root.workspaceKind === "rustCargo" ? qsTr("Cargo: debug") : qsTr("Perfil: automático"))
              : root.activeConfigName
        onClicked: {
            const pos = root.mapFromItem(configSelector, 0, configSelector.height + 4);
            root.configMenuRequested(pos.x, pos.y);
        }
    }

    KvIconButton {
        anchors.verticalCenter: parent.verticalCenter
        enabled: root.coreConnected
        primary: !root.running
        danger: root.running
        iconName: root.running ? "stop" : "run"
        tooltip: root.running ? qsTr("Parar execução") : qsTr("Rodar a configuração ativa")
        onClicked: root.running ? root.stopRunRequested() : root.runRequested()
    }

    KvIconButton {
        anchors.verticalCenter: parent.verticalCenter
        enabled: root.coreConnected
        active: root.debugging
        danger: root.debugging
        iconName: root.debugging ? "stop" : "debug"
        tooltip: root.debugging ? qsTr("Parar depuração") : qsTr("Depurar a configuração ativa")
        onClicked: root.debugging ? root.stopDebugRequested() : root.debugRequested()
    }

    // Compilar, testar, analisar, cobertura, configurar: um menu, com
    // rotulo por sistema (o que os icones iguais nao diziam).
    Item {
        anchors.verticalCenter: parent.verticalCenter
        width: 32
        height: 32

        KvIconButton {
            id: acoes

            anchors.fill: parent
            enabled: root.coreConnected
            active: root.actionsMenuOpen
            iconName: "build"
            tooltip: root.busy ? root.busyLabel : qsTr("Compilar, testar, analisar…")
            onClicked: {
                const pos = root.mapFromItem(acoes, 0, acoes.height + 4);
                root.actionsMenuRequested(pos.x, pos.y);
            }
        }

        // O ponto que pulsa enquanto um job de build/teste/analise roda.
        Rectangle {
            anchors.right: parent.right
            anchors.top: parent.top
            anchors.margins: 3
            visible: root.busy
            width: 8
            height: 8
            radius: 4
            color: Theme.accent

            SequentialAnimation on opacity {
                running: root.busy
                loops: Animation.Infinite
                NumberAnimation { from: 1.0; to: 0.3; duration: 600 }
                NumberAnimation { from: 0.3; to: 1.0; duration: 600 }
            }
        }
    }
}
