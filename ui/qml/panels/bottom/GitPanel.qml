pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A HUD do Git (2026-09-18, pedido do autor: "uma HUD única para o Git,
// como as IDEs JetBrains"): a linha do branch com pull/push/stash; duas
// vistas — Mudanças (a lista agrupada por pasta, o commit embaixo) e
// Histórico (o grafo, os refs) — e, à direita, o que está selecionado:
// o diff da mudança ou o commit inteiro (autor, refs, arquivos, patch).
// Fala com o GitController (dono do estado); só abrir arquivo sai por sinal.
Item {
    id: panel

    property var gitController: null

    signal openRequested(string absPath)

    readonly property bool historyVisible: gitController ? gitController.historyVisible : false
    readonly property real leftWidth: Math.round(width * 0.44)

    function clearMessage() {
        commitRow.clearMessage();
    }

    Row {
        id: gitActions

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        height: 22
        spacing: Theme.spacingSmall

        Repeater {
            model: [
                { label: panel.gitController && panel.gitController.branchLabel !== ""
                         ? panel.gitController.branchLabel : qsTr("branch"), action: "branch" },
                { label: qsTr("pull"), action: "pull" },
                { label: qsTr("push"), action: "push" },
                { label: qsTr("stash"), action: "stash" },
                { label: qsTr("pop"), action: "pop" }
            ]

            delegate: Rectangle {
                id: gitActionChip

                required property var modelData

                readonly property bool remoteBusy: panel.gitController
                    && panel.gitController.remoteOperationRunning
                    && (modelData.action === "pull" || modelData.action === "push")

                width: actionLabel.width + 2 * Theme.spacingSmall
                height: 22
                radius: Theme.radiusXSmall
                opacity: remoteBusy ? 0.5 : 1.0
                color: actionArea.containsMouse ? Theme.surface2 : Theme.surface1
                border.color: modelData.action === "branch" ? Theme.accent : Theme.borderSoft
                border.width: 1

                Text {
                    id: actionLabel

                    anchors.centerIn: parent
                    text: gitActionChip.modelData.label
                    color: gitActionChip.modelData.action === "branch" ? Theme.accent : Theme.textSecondary
                    font.pixelSize: 10
                    font.family: gitActionChip.modelData.action === "branch" ? Theme.monoFont : Theme.uiFont
                }

                MouseArea {
                    id: actionArea

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        const action = gitActionChip.modelData.action;
                        if (action === "branch") panel.gitController.openBranchMenu();
                        else if (action === "pull" || action === "push") panel.gitController.startRemote(action);
                        else panel.gitController.stashRequested(action === "pop" ? "pop" : "push", "");
                    }
                }
            }
        }

        Item { width: Theme.spacingSmall; height: 1 }

        KvToggleChip {
            anchors.verticalCenter: parent.verticalCenter
            labelText: panel.gitController && panel.gitController.changeCount > 0
                       ? qsTr("Mudanças (%1)").arg(panel.gitController.changeCount) : qsTr("Mudanças")
            active: !panel.historyVisible
            onToggled: panel.gitController.showChanges()
        }

        KvToggleChip {
            anchors.verticalCenter: parent.verticalCenter
            labelText: qsTr("Histórico")
            active: panel.historyVisible
            onToggled: panel.gitController.openHistory()
        }

        KvIconButton {
            anchors.verticalCenter: parent.verticalCenter
            visible: panel.historyVisible
            compact: true
            iconName: "refresh"
            tooltip: qsTr("Atualizar o histórico")
            onClicked: panel.gitController.refreshHistory()
        }
    }

    GitBranchMenu {
        id: branchMenu

        anchors.top: gitActions.bottom
        anchors.left: parent.left
        anchors.topMargin: Theme.spacingXSmall
        z: 5
        visible: panel.gitController ? panel.gitController.branchMenuVisible : false
        branchesModel: panel.gitController ? panel.gitController.branchesModel : null
        onBranchCheckoutRequested: function(branch) { panel.gitController.checkoutBranch(branch); }
        onBranchCreateRequested: function(name) { panel.gitController.createBranch(name); }
    }

    // ---- a esquerda: a lista (mudancas ou historico) e o commit ----------

    // O filtro do historico (fatia 2b): texto local + branch pelo core.
    GitHistoryFilter {
        id: historyFilter

        anchors.top: gitActions.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.left: parent.left
        width: panel.leftWidth
        height: visible ? implicitHeight : 0
        visible: panel.historyVisible
        z: 4
        filterText: panel.gitController ? panel.gitController.historyFilterText : ""
        logRef: panel.gitController ? panel.gitController.historyLogRef : ""
        branchesModel: panel.gitController ? panel.gitController.branchesModel : null
        onFilterTextEdited: function(text) { panel.gitController.setHistoryFilter(text); }
        onLogRefChosen: function(ref) { panel.gitController.setHistoryRef(ref); }
        onRefsOpenChanged: if (refsOpen && panel.gitController) panel.gitController.branchesRequested()
    }

    GitHistoryList {
        id: historyView

        anchors.top: historyFilter.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        width: panel.leftWidth
        visible: panel.historyVisible
        historyModel: panel.gitController ? panel.gitController.historyModel : null
        historyLoading: panel.gitController ? panel.gitController.historyLoading : false
        repo: panel.gitController ? panel.gitController.repo : false
        laneCount: panel.gitController ? panel.gitController.historyLaneCount : 1
        selectedSha: panel.gitController ? panel.gitController.inspector.sha : ""
        onCommitActivated: function(sha, shortSha, summary) {
            panel.gitController.inspector.showCommit(panel.gitController.historyEntry(sha));
        }
    }

    GitChangesList {
        id: changesView

        anchors.top: gitActions.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.bottom: commitRow.top
        anchors.bottomMargin: Theme.spacingSmall
        anchors.left: parent.left
        width: panel.leftWidth
        visible: !panel.historyVisible
        changesModel: panel.gitController ? panel.gitController.changesModel : null
        repo: panel.gitController ? panel.gitController.repo : false
        revision: panel.gitController ? panel.gitController.revision : 0
        selectedAbsPath: panel.gitController ? panel.gitController.inspector.path : ""
        onStageToggleRequested: function(index) { panel.gitController.toggleStaged(index); }
        onFolderStageRequested: function(absPaths, stageAll) { panel.gitController.stageFolder(absPaths, stageAll); }
        onSelectRequested: function(absPath, path) { panel.gitController.inspector.showChange(absPath, path); }
        onDiffRequested: function(absPath) { panel.gitController.openDiffDialog(absPath); }
        onDiscardRequested: function(index) { panel.gitController.openDiscardDialog(index); }
        onOpenRequested: function(absPath) { panel.openRequested(absPath); }
    }

    GitCommitBox {
        id: commitRow

        anchors.bottom: parent.bottom
        anchors.left: parent.left
        width: panel.leftWidth
        height: implicitHeight
        historyVisible: panel.historyVisible
        errorText: panel.gitController ? panel.gitController.lastMutationError : ""
        stagedCount: panel.gitController ? panel.gitController.stagedCount : 0
        amend: panel.gitController ? panel.gitController.amend : false
        remoteRunning: panel.gitController ? panel.gitController.remoteOperationRunning : false
        onCommitRequested: function(message) { panel.gitController.commit(message); }
        onCommitAndPushRequested: function(message) { panel.gitController.commitAndPush(message); }
        onAmendToggled: panel.gitController.amend = !panel.gitController.amend
    }

    // ---- a direita: o que esta' selecionado -------------------------------

    Rectangle {
        anchors.top: gitActions.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.bottom: parent.bottom
        anchors.left: changesView.right
        anchors.leftMargin: Theme.spacingMedium
        anchors.right: parent.right
        radius: Theme.radius
        color: Theme.background0
        border.width: 1
        border.color: Theme.borderSoft

        GitInspectorPane {
            anchors.fill: parent
            anchors.margins: Theme.spacingSmall
            inspector: panel.gitController ? panel.gitController.inspector : null
        }
    }
}
