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
    property var testModel
    property string testSummary: ""
    property bool testing: false
    property var problemsModel
    property var terminalRender: ({})
    property bool terminalActive: false
    property bool workspaceAvailable: false
    property var runModel
    property bool running: false
    property var debugOutputModel
    property bool debugSessionActive: false
    property bool debugPaused: false
    property var debugFramesModel
    property var debugVariablesModel
    property int debugCurrentFrameIndex: -1
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
    signal terminalOpenRequested()
    signal terminalKeyPressed(string data)
    signal terminalResizeRequested(int cols, int rows)
    signal terminalScrollRequested(int offset)
    signal terminalSelectRequested(string id)
    signal terminalNewRequested()
    signal terminalCloseTabRequested(string id)
    signal runInputSubmitted(string text)
    signal debugContinueRequested()
    signal debugPauseRequested()
    signal debugStepOverRequested()
    signal debugStepIntoRequested()
    signal debugStepOutRequested()
    signal debugStopRequested()
    signal debugFrameActivated(int index)
    signal debugVariableToggled(int index)
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
        casesModel: root.testModel
        summary: root.testSummary
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
    }

    Row {
        id: sessionRow

        anchors.top: bottomTabs.bottom
        anchors.left: parent.left
        anchors.leftMargin: Theme.spacingSmall
        anchors.topMargin: root.activeTab === "terminal" ? Theme.spacingXSmall : 0
        height: root.activeTab === "terminal" ? 22 : 0
        visible: root.activeTab === "terminal"
        spacing: Theme.spacingSmall

        // D2.3 (docs/24): uma aba por terminal aberto. A "Execução" continua
        // sendo uma sessão à parte — ela NÃO é um PTY (é o backend run.*),
        // por isso não entra no mesmo modelo.
        Repeater {
            model: root.terminalsModel

            delegate: Rectangle {
                id: termChip

                required property string termId
                required property string title

                readonly property bool current: root.terminalSession === "shell"
                                                && root.activeTerminalId === termChip.termId

                width: termChipRow.width + 2 * Theme.spacingSmall
                height: 20
                radius: Theme.radiusXSmall
                color: termChip.current ? Theme.surfaceSelected : "transparent"
                border.color: Theme.borderSoft
                border.width: 1

                // Fica ATRÁS do conteúdo (z: -1) pra não engolir o clique do
                // ícone: selecionar e fechar são gestos diferentes na mesma aba.
                MouseArea {
                    anchors.fill: parent
                    z: -1
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.terminalSelectRequested(termChip.termId)
                }

                Row {
                    id: termChipRow

                    anchors.centerIn: parent
                    spacing: Theme.spacingXSmall

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        text: termChip.title
                        color: termChip.current ? Theme.accent : Theme.textSecondary
                        font.pixelSize: 10
                        font.bold: true
                    }

                    KvIcon {
                        id: termCloseLabel

                        anchors.verticalCenter: parent.verticalCenter
                        name: "close"
                        size: 14
                        iconColor: termCloseArea.containsMouse
                                   ? Theme.textPrimary : Theme.textMuted

                        MouseArea {
                            id: termCloseArea

                            anchors.fill: parent
                            anchors.margins: -3
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: root.terminalCloseTabRequested(termChip.termId)
                        }
                    }
                }
            }
        }

        // Novo terminal.
        Rectangle {
            width: 20
            height: 20
            radius: Theme.radiusXSmall
            color: newTerminalArea.containsMouse ? Theme.surface2 : "transparent"
            border.color: Theme.borderSoft
            border.width: 1

            Text {
                anchors.centerIn: parent
                text: "+"
                color: newTerminalArea.containsMouse ? Theme.accent : Theme.textSecondary
                font.pixelSize: 12
                font.bold: true
            }

            MouseArea {
                id: newTerminalArea

                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: root.terminalNewRequested()
            }
        }

        Rectangle {
            id: runSessionChip

            readonly property bool current: root.terminalSession === "run"

            width: runSessionRow.width + 2 * Theme.spacingSmall
            height: 20
            radius: Theme.radiusXSmall
            color: runSessionChip.current ? Theme.surfaceSelected : "transparent"
            border.color: Theme.borderSoft
            border.width: 1

            Row {
                id: runSessionRow
                anchors.centerIn: parent
                spacing: Theme.spacingXSmall

                Text {
                    id: runSessionLabel

                    anchors.verticalCenter: parent.verticalCenter
                    text: qsTr("Execução")
                    color: runSessionChip.current ? Theme.accent : Theme.textSecondary
                    font.pixelSize: 10
                    font.bold: true
                }

                Rectangle {
                    anchors.verticalCenter: parent.verticalCenter
                    visible: root.running
                    width: 6
                    height: 6
                    radius: 3
                    color: Theme.successSoft
                }
            }

            MouseArea {
                anchors.fill: parent
                cursorShape: Qt.PointingHandCursor
                onClicked: root.terminalSessionRequested("run")
            }
        }

        Rectangle {
            width: clearSessionLabel.width + 2 * Theme.spacingSmall
            height: 20
            radius: Theme.radiusXSmall
            color: clearSessionArea.containsMouse ? Theme.surface2 : "transparent"
            border.color: Theme.borderSoft
            border.width: 1

            Text {
                id: clearSessionLabel

                anchors.centerIn: parent
                text: qsTr("limpar")
                color: Theme.textMuted
                font.pixelSize: 10
            }

            MouseArea {
                id: clearSessionArea

                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: root.clearSessionRequested()
            }
        }
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
        outputModel: root.debugOutputModel
        sessionActive: root.debugSessionActive
        paused: root.debugPaused
        framesModel: root.debugFramesModel
        variablesModel: root.debugVariablesModel
        currentFrameIndex: root.debugCurrentFrameIndex
        onFrameActivated: function(index) {
            root.debugFrameActivated(index);
        }
        onVariableToggled: function(index) {
            root.debugVariableToggled(index);
        }
        onContinueRequested: root.debugContinueRequested()
        onPauseRequested: root.debugPauseRequested()
        onStepOverRequested: root.debugStepOverRequested()
        onStepIntoRequested: root.debugStepIntoRequested()
        onStepOutRequested: root.debugStepOutRequested()
        onStopRequested: root.debugStopRequested()
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
