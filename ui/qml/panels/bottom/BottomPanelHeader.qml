pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Cabecalho de LINHA UNICA do painel inferior (F3, 2026-07-18): titulo da
// ferramenta ativa + acoes DELA (abas de sessao no terminal, redetectar nas
// ferramentas) + esconder. A fileira de 10 abas morreu: a ferramenta e
// escolhida no rail esquerdo, na status bar (IDE log) ou no menu Exibir
// (Jobs) — como no IntelliJ, o painel mostra UMA ferramenta e o seu nome.
Item {
    id: root

    property string activeTab: ""
    property var problemsModel: null
    property bool processRunning: false
    property var terminalsModel: null
    property string activeTerminalId: ""
    property string terminalSession: "shell"
    property bool running: false

    property bool measuringCoverage: false

    signal hideRequested()
    signal refreshToolsRequested()
    signal coverageRequested()
    signal terminalSelectRequested(string id)
    signal terminalCloseTabRequested(string id)
    signal terminalNewRequested()
    signal terminalSessionRequested(string session)
    signal clearSessionRequested()

    function tabIcon(tab) {
        const icons = {
            build: "build", jobs: "run", problems: "problems", tests: "test",
            terminal: "terminal", debug: "debug", git: "git", search: "search",
            logs: "file", tools: "tools", compileContext: "build"
        };
        return icons[tab] !== undefined ? icons[tab] : "file";
    }

    function tabLabel(tab) {
        const labels = {
            build: qsTr("Build"), jobs: qsTr("Jobs"),
            problems: qsTr("Problemas"), tests: qsTr("Testes"),
            terminal: qsTr("Terminal"), debug: qsTr("Debug"),
            git: qsTr("Git"), search: qsTr("Busca"),
            logs: qsTr("Log da IDE"), tools: qsTr("Ferramentas"),
            compileContext: qsTr("Contexto de compilação")
        };
        const label = labels[tab] !== undefined ? labels[tab] : tab;
        // Contagem DERIVADA do modelo que ja desce. Antes ela vinha por uma
        // propriedade propria, atravessando ShellWorkspaceHost e
        // BottomPanelHost em paralelo ao proprio modelo — dois caminhos para
        // o mesmo fato, e o numerico podia mentir se um deles ficasse para tras.
        const problemas = problemsModel ? problemsModel.count : 0;
        if (tab === "problems" && problemas > 0) {
            return qsTr("Problemas (%1)").arg(problemas);
        }
        if (tab === "terminal" && processRunning) {
            return label + " ·";
        }
        return label;
    }

    height: 34

    Row {
        id: titleRow

        anchors.left: parent.left
        anchors.verticalCenter: parent.verticalCenter
        spacing: Theme.spacingSmall

        KvIcon {
            anchors.verticalCenter: parent.verticalCenter
            name: root.tabIcon(root.activeTab)
            size: 15
            active: true
        }

        Text {
            anchors.verticalCenter: parent.verticalCenter
            text: root.tabLabel(root.activeTab)
            color: Theme.textPrimary
            font.pixelSize: Theme.fontSizePanelTitle
            font.bold: true
        }
    }

    // Acoes da ferramenta ativa, na MESMA linha do titulo.
    TerminalSessionTabs {
        anchors.left: titleRow.right
        anchors.leftMargin: Theme.spacingMedium
        anchors.verticalCenter: parent.verticalCenter
        height: 22
        visible: root.activeTab === "terminal"
        sessionsModel: root.terminalsModel
        activeTerminalId: root.activeTerminalId
        terminalSession: root.terminalSession
        running: root.running
        onSelectRequested: function(id) { root.terminalSelectRequested(id); }
        onCloseRequested: function(id) { root.terminalCloseTabRequested(id); }
        onNewRequested: root.terminalNewRequested()
        onSessionRequested: function(s) { root.terminalSessionRequested(s); }
        onClearRequested: root.clearSessionRequested()
    }

    KvButton {
        anchors.left: titleRow.right
        anchors.leftMargin: Theme.spacingMedium
        anchors.verticalCenter: parent.verticalCenter
        height: 24
        visible: root.activeTab === "tools"
        compact: true
        iconName: "refresh"
        text: qsTr("Redetectar")
        onClicked: root.refreshToolsRequested()
    }

    // Cobertura mora com Testes, como o "Run with Coverage" do IntelliJ — nao
    // ganha icone proprio no rail (a §12.2 proibe rail-propaganda) nem aba
    // propria (a F3 matou a fileira de abas).
    KvButton {
        anchors.left: titleRow.right
        anchors.leftMargin: Theme.spacingMedium
        anchors.verticalCenter: parent.verticalCenter
        height: 24
        visible: root.activeTab === "tests"
        enabled: !root.measuringCoverage
        compact: true
        iconName: "test"
        text: root.measuringCoverage ? qsTr("Medindo...") : qsTr("Cobertura")
        onClicked: root.coverageRequested()
    }

    KvIconButton {
        anchors.right: parent.right
        anchors.verticalCenter: parent.verticalCenter
        width: 24
        height: 24
        iconName: "minimize"
        iconSize: 13
        tooltip: qsTr("Ocultar painel")
        onClicked: root.hideRequested()
    }
}
