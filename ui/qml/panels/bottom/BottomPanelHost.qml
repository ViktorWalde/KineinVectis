pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

Rectangle {
    id: root

    property bool open: false
    property string activeTab: "logs"
    property int problemCount: 0
    property var buildOutputModel
    property var jobsModel
    // O painel de testes recebe o DONO (JobsController), nao copias: a arvore,
    // os casos, a saida e o resumo moram nele, e "listar"/"rodar um" sao
    // pedidos a ele (mesmo padrao do debugController abaixo, 2026-09-13).
    property var jobsController
    property bool testing: false
    property var problemsModel
    property var terminalRender: ({})
    property bool terminalActive: false
    property bool workspaceAvailable: false
    property var runModel
    property bool running: false
    // O dominio de debug entra como CONTROLLER, nao como 7 escalares e 10
    // sinais de passagem. Mesmo padrao do ShellHeaderHost e do ShellEditorHost:
    // host recebe dono, nao copia de estado. Foi o que fez o ShellWorkspaceHost
    // caber de novo quando os watches entraram (etapa 15).
    property var debugController
    property var gitChangesModel
    property bool gitRepo: false
    property int gitStagedCount: 0
    property string gitErrorText: ""
    property var gitHistoryModel
    property bool gitHistoryVisible: false
    property bool gitHistoryLoading: false
    property string gitBranchLabel: ""
    property var gitBranchesModel
    property bool gitBranchMenuVisible: false
    property bool gitRemoteOperationRunning: false
    property string terminalSession: "shell"
    // D2.3: abas de terminal.
    property var terminalsModel: null
    property string activeTerminalId: ""
    property var searchModel
    property bool searchCaseSensitive: false
    property bool searching: false
    property bool searchTruncated: false
    property bool searchReplaceMode: false
    property bool searchReplacing: false
    property string searchReplaceError: ""
    property string searchReplaceSummary: ""
    property var logLinesModel
    property var toolsList

    signal tabRequested(string tab)
    signal terminalSessionRequested(string session)
    signal clearSessionRequested()
    signal refreshToolsRequested()
    signal problemOpenRequested(string file, int line, int column)
    signal problemNextStepRequested(string kind, string target, string file, int line, int column)
    signal terminalOpenRequested()
    signal terminalKeyPressed(string data)
    signal terminalResizeRequested(int cols, int rows)
    signal terminalScrollRequested(int offset)
    signal terminalWheelRequested(int col, int row, int lines, int modifiers)
    signal terminalSelectRequested(string id)
    signal terminalNewRequested()
    signal terminalCloseTabRequested(string id)
    signal runInputSubmitted(string text)
    signal gitStageToggleRequested(int index)
    signal gitDiffRequested(string absPath)
    signal gitDiscardRequested(int index)
    signal gitOpenRequested(string absPath)
    signal gitCommitRequested(string message)
    signal gitChangesViewRequested()
    signal gitHistoryViewRequested()
    signal gitHistoryRefreshRequested()
    signal gitCommitActivated(string sha, string shortSha, string summary)
    signal gitBranchMenuRequested()
    signal gitBranchCheckoutRequested(string branch)
    signal gitBranchCreateRequested(string name)
    signal gitRemoteRequested(string operation)
    signal gitStashRequested(string action)
    signal searchRequested(string query)
    signal searchCaseSensitivityToggleRequested(string query)
    signal searchResultOpenRequested(string path, int line, int column)
    signal searchReplaceRequested(string query, string replacement)

    visible: open
    radius: Theme.radiusLarge
    color: Theme.background2
    border.color: Theme.borderSoft
    border.width: 1

    function focusSearchInput() {
        searchView.focusInput();
    }

    function clearSearchInput() {
        searchView.clearInput();
    }

    function focusSearchReplaceInput() {
        searchView.focusReplaceInput();
    }

    function focusTerminalInput() {
        if (root.terminalSession === "run") {
            runView.focusInput();
        } else {
            terminalView.focusInput();
        }
    }

    function clearTerminalInput() {
        terminalView.clearInput();
    }

    function clearRunInput() {
        runView.clearInput();
    }

    BottomTabBar {
        id: bottomTabs

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.margins: Theme.spacingSmall
        activeTab: root.activeTab
        problemCount: root.problemCount
        processRunning: root.running
        onTabRequested: function(tab) {
            root.tabRequested(tab);
        }
        onRefreshToolsRequested: root.refreshToolsRequested()
    }

    BuildPanel {
        anchors.top: bottomTabs.bottom
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.spacingSmall
        visible: root.activeTab === "build"
        outputModel: root.buildOutputModel
    }

    TestsPanel {
        anchors.top: bottomTabs.bottom
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.spacingSmall
        visible: root.activeTab === "tests"
        controller: root.jobsController
        running: root.testing
    }

    JobsPanel {
        anchors.top: bottomTabs.bottom
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.spacingSmall
        visible: root.activeTab === "jobs"
        jobsModel: root.jobsModel
    }

    ProblemsPanel {
        anchors.top: bottomTabs.bottom
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.spacingSmall
        visible: root.activeTab === "problems"
        diagnosticsModel: root.problemsModel
        onOpenRequested: function(file, line, column) {
            root.problemOpenRequested(file, line, column);
        }
        onNextStepRequested: (kind, target, file, line, column) =>
            root.problemNextStepRequested(kind, target, file, line, column)
    }

    TerminalSessionTabs {
        id: sessionRow

        anchors.top: bottomTabs.bottom
        anchors.left: parent.left
        anchors.leftMargin: Theme.spacingSmall
        anchors.topMargin: root.activeTab === "terminal" ? Theme.spacingXSmall : 0
        height: root.activeTab === "terminal" ? 22 : 0
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

    TerminalPanel {
        id: terminalView

        anchors.top: sessionRow.bottom
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.spacingSmall
        visible: root.activeTab === "terminal" && root.terminalSession === "shell"
        render: root.terminalRender
        terminalActive: root.terminalActive
        workspaceAvailable: root.workspaceAvailable
        onOpenRequested: root.terminalOpenRequested()
        onKeyPressed: function(data) {
            root.terminalKeyPressed(data);
        }
        onResizeRequested: function(cols, rows) {
            root.terminalResizeRequested(cols, rows);
        }
        onScrollRequested: function(offset) {
            root.terminalScrollRequested(offset);
        }
        onWheelRequested: function(col, row, lines, modifiers) {
            root.terminalWheelRequested(col, row, lines, modifiers);
        }
    }

    RunPanel {
        id: runView

        anchors.top: sessionRow.bottom
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.spacingSmall
        visible: root.activeTab === "terminal" && root.terminalSession === "run"
        outputModel: root.runModel
        running: root.running
        onInputSubmitted: function(text) {
            root.runInputSubmitted(text);
        }
    }

    DebugPanel {
        anchors.top: bottomTabs.bottom
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.spacingSmall
        visible: root.activeTab === "debug"

        outputModel: root.debugController.outputModel
        sessionActive: root.debugController.sessionActive
        attached: root.debugController.attached
        canStart: root.debugController.workspaceRoot !== ""
                  && !root.debugController.sessionActive && !root.debugController.starting
        onAttachRequested: function(host, port) { root.debugController.startDebug("", { host: host, port: port }); }
        paused: root.debugController.paused
        framesModel: root.debugController.framesModel
        variablesModel: root.debugController.variablesModel
        watchesModel: root.debugController.watchesModel
        inspect: root.debugController.inspect
        currentFrameIndex: root.debugController.currentFrameIndex

        onFrameActivated: function(index) { root.debugController.selectFrame(index, true); }
        onVariableToggled: function(index) { root.debugController.toggleVariable(index); }
        onWatchAdded: function(expression) { root.debugController.addWatch(expression); }
        onWatchRemoved: function(index) { root.debugController.removeWatch(index); }
        onContinueRequested: root.debugController.continueDebug()
        onPauseRequested: root.debugController.pauseDebug()
        onStepOverRequested: root.debugController.stepOver()
        onStepIntoRequested: root.debugController.stepInto()
        onStepOutRequested: root.debugController.stepOutOf()
        onStopRequested: root.debugController.stopDebug()
    }

    GitPanel {
        anchors.top: bottomTabs.bottom
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.spacingSmall
        visible: root.activeTab === "git"
        changesModel: root.gitChangesModel
        repo: root.gitRepo
        stagedCount: root.gitStagedCount
        errorText: root.gitErrorText
        historyModel: root.gitHistoryModel
        historyVisible: root.gitHistoryVisible
        historyLoading: root.gitHistoryLoading
        branchLabel: root.gitBranchLabel
        branchesModel: root.gitBranchesModel
        branchMenuVisible: root.gitBranchMenuVisible
        remoteOperationRunning: root.gitRemoteOperationRunning
        onStageToggleRequested: function(index) {
            root.gitStageToggleRequested(index);
        }
        onDiffRequested: function(absPath) {
            root.gitDiffRequested(absPath);
        }
        onDiscardRequested: function(index) {
            root.gitDiscardRequested(index);
        }
        onOpenRequested: function(absPath) {
            root.gitOpenRequested(absPath);
        }
        onCommitRequested: function(message) {
            root.gitCommitRequested(message);
        }
        onChangesViewRequested: root.gitChangesViewRequested()
        onHistoryViewRequested: root.gitHistoryViewRequested()
        onHistoryRefreshRequested: root.gitHistoryRefreshRequested()
        onCommitActivated: function(sha, shortSha, summary) {
            root.gitCommitActivated(sha, shortSha, summary);
        }
        onBranchMenuRequested: root.gitBranchMenuRequested()
        onBranchCheckoutRequested: function(branch) {
            root.gitBranchCheckoutRequested(branch);
        }
        onBranchCreateRequested: function(name) {
            root.gitBranchCreateRequested(name);
        }
        onRemoteRequested: function(operation) {
            root.gitRemoteRequested(operation);
        }
        onStashRequested: function(action) {
            root.gitStashRequested(action);
        }
    }

    SearchPanel {
        id: searchView

        anchors.top: bottomTabs.bottom
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.spacingSmall
        visible: root.activeTab === "search"
        resultsModel: root.searchModel
        caseSensitive: root.searchCaseSensitive
        searching: root.searching
        truncated: root.searchTruncated
        replaceMode: root.searchReplaceMode
        replacing: root.searchReplacing
        replaceError: root.searchReplaceError
        replaceSummary: root.searchReplaceSummary
        onSearchRequested: function(query) {
            root.searchRequested(query);
        }
        onCaseSensitivityToggleRequested: function(query) {
            root.searchCaseSensitivityToggleRequested(query);
        }
        onResultOpenRequested: function(path, line, column) {
            root.searchResultOpenRequested(path, line, column);
        }
        onReplaceRequested: function(query, replacement) {
            root.searchReplaceRequested(query, replacement);
        }
    }

    IdeLogPanel {
        anchors.top: bottomTabs.bottom
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.spacingSmall
        visible: root.activeTab === "logs"
        logLinesModel: root.logLinesModel
    }

    ToolsPanel {
        anchors.top: bottomTabs.bottom
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.spacingSmall
        visible: root.activeTab === "tools"
        tools: root.toolsList
    }
}
