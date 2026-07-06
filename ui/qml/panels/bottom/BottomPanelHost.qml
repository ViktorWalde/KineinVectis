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
    property string terminalText: ""
    property bool terminalActive: false
    property bool workspaceAvailable: false
    property var runModel
    property bool running: false
    property var searchModel
    property bool searchCaseSensitive: false
    property bool searching: false
    property bool searchTruncated: false
    property var logLinesModel
    property var toolsList

    signal tabRequested(string tab)
    signal refreshToolsRequested()
    signal problemOpenRequested(string file, int line, int column)
    signal terminalOpenRequested()
    signal terminalInputSubmitted(string text)
    signal runInputSubmitted(string text)
    signal searchRequested(string query)
    signal searchCaseSensitivityToggleRequested(string query)
    signal searchResultOpenRequested(string path, int line, int column)

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

    function focusTerminalInput() {
        terminalView.focusInput();
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

    TerminalPanel {
        id: terminalView

        anchors.top: bottomTabs.bottom
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.spacingSmall
        visible: root.activeTab === "terminal"
        terminalText: root.terminalText
        terminalActive: root.terminalActive
        workspaceAvailable: root.workspaceAvailable
        onOpenRequested: root.terminalOpenRequested()
        onInputSubmitted: function(text) {
            root.terminalInputSubmitted(text);
        }
    }

    RunPanel {
        id: runView

        anchors.top: bottomTabs.bottom
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.spacingSmall
        visible: root.activeTab === "run"
        outputModel: root.runModel
        running: root.running
        onInputSubmitted: function(text) {
            root.runInputSubmitted(text);
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
        onSearchRequested: function(query) {
            root.searchRequested(query);
        }
        onCaseSensitivityToggleRequested: function(query) {
            root.searchCaseSensitivityToggleRequested(query);
        }
        onResultOpenRequested: function(path, line, column) {
            root.searchResultOpenRequested(path, line, column);
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
