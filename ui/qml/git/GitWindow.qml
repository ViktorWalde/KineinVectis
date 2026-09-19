pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A janela do Git EM PE', a esquerda (Etapa 3, E3-3 — roadmaps/44 §4.1;
// pedido do autor: "o posicionamento nao ta' memoria muscular JetBrains").
// Alterna com o explorer no mesmo slot. Abas Commit | Log em cima; a
// linha do branch (pull/push/stash/pop) quebra em Flow para caber a 220
// px; Commit = a lista de mudancas por pasta + a caixa de commit no pe';
// Log = o filtro + o grafo. O que esta' selecionado abre NO EDITOR
// (GitViewerPane), nao aqui. Fala com o GitController; abrir arquivo e
// fechar a janela saem por sinal.
Rectangle {
    id: root

    property var gitController: null

    signal openRequested(string absPath)
    signal closeRequested()

    readonly property bool historyVisible: gitController ? gitController.historyVisible : false

    radius: Theme.radiusLarge
    color: Theme.background1
    border.color: Theme.borderSoft
    border.width: 1

    function clearMessage() {
        commitRow.clearMessage();
    }

    // ---- o cabecalho: titulo, as duas abas, atualizar, fechar -------------

    Row {
        id: cabecalho

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.spacingSmall
        height: 24
        spacing: Theme.spacingSmall

        // O titulo cede a vez as abas a 220 px (a foto a 1024: o x sumia).
        Text {
            id: titulo

            anchors.verticalCenter: parent.verticalCenter
            visible: root.width >= 260
            width: visible ? implicitWidth : -parent.spacing
            text: qsTr("Git")
            color: Theme.textPrimary
            font.pixelSize: 12
            font.weight: Font.DemiBold
        }

        KvToggleChip {
            id: abaCommit

            anchors.verticalCenter: parent.verticalCenter
            labelText: root.gitController && root.gitController.changeCount > 0
                       ? qsTr("Commit (%1)").arg(root.gitController.changeCount) : qsTr("Commit")
            active: !root.historyVisible
            onToggled: root.gitController.showChanges()
        }

        KvToggleChip {
            id: abaLog

            anchors.verticalCenter: parent.verticalCenter
            labelText: qsTr("Log")
            active: root.historyVisible
            onToggled: root.gitController.openHistory()
        }

        Item {
            width: Math.max(0, parent.width - titulo.width - abaCommit.width - abaLog.width
                            - atualizar.width - fechar.width - 5 * parent.spacing)
            height: 1
        }

        KvIconButton {
            id: atualizar

            anchors.verticalCenter: parent.verticalCenter
            compact: true
            iconName: "refresh"
            tooltip: root.historyVisible ? qsTr("Atualizar o histórico") : qsTr("Atualizar as mudanças")
            onClicked: root.historyVisible ? root.gitController.refreshHistory() : root.gitController.refresh()
        }

        KvIconButton {
            id: fechar

            anchors.verticalCenter: parent.verticalCenter
            compact: true
            iconName: "close"
            tooltip: qsTr("Fechar a janela do Git")
            onClicked: root.closeRequested()
        }
    }

    // ---- a linha do branch: quebra quando a janela e' estreita ------------

    Flow {
        id: gitActions

        anchors.top: cabecalho.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.spacingSmall
        anchors.topMargin: Theme.spacingXSmall
        spacing: Theme.spacingXSmall

        Repeater {
            model: [
                { label: root.gitController && root.gitController.branchLabel !== ""
                         ? root.gitController.branchLabel : qsTr("branch"), action: "branch" },
                { label: qsTr("pull"), action: "pull" },
                { label: qsTr("push"), action: "push" },
                { label: qsTr("stash"), action: "stash" },
                { label: qsTr("pop"), action: "pop" }
            ]

            delegate: Rectangle {
                id: gitActionChip

                required property var modelData

                readonly property bool remoteBusy: root.gitController
                    && root.gitController.remoteOperationRunning
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
                        if (action === "branch") root.gitController.openBranchMenu();
                        else if (action === "pull" || action === "push") root.gitController.startRemote(action);
                        else root.gitController.stashRequested(action === "pop" ? "pop" : "push", "");
                    }
                }
            }
        }
    }

    GitBranchMenu {
        id: branchMenu

        anchors.top: gitActions.bottom
        anchors.left: parent.left
        anchors.leftMargin: Theme.spacingSmall
        anchors.topMargin: Theme.spacingXSmall
        z: 5
        visible: root.gitController ? root.gitController.branchMenuVisible : false
        branchesModel: root.gitController ? root.gitController.branchesModel : null
        onBranchCheckoutRequested: function(branch) { root.gitController.checkoutBranch(branch); }
        onBranchCreateRequested: function(name) { root.gitController.createBranch(name); }
    }

    // ---- Log: o filtro e o grafo ------------------------------------------

    GitHistoryFilter {
        id: historyFilter

        anchors.top: gitActions.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.leftMargin: Theme.spacingSmall
        anchors.rightMargin: Theme.spacingSmall
        height: visible ? implicitHeight : 0
        visible: root.historyVisible
        z: 4
        filterText: root.gitController ? root.gitController.historyFilterText : ""
        logRef: root.gitController ? root.gitController.historyLogRef : ""
        branchesModel: root.gitController ? root.gitController.branchesModel : null
        onFilterTextEdited: function(text) { root.gitController.setHistoryFilter(text); }
        onLogRefChosen: function(ref) { root.gitController.setHistoryRef(ref); }
        onRefsOpenChanged: if (refsOpen && root.gitController) root.gitController.branchesRequested()
    }

    GitHistoryList {
        id: historyView

        anchors.top: historyFilter.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.bottom: parent.bottom
        anchors.bottomMargin: Theme.spacingSmall
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.leftMargin: Theme.spacingSmall
        anchors.rightMargin: Theme.spacingSmall
        visible: root.historyVisible
        historyModel: root.gitController ? root.gitController.historyModel : null
        historyLoading: root.gitController ? root.gitController.historyLoading : false
        repo: root.gitController ? root.gitController.repo : false
        laneCount: root.gitController ? root.gitController.historyLaneCount : 1
        selectedSha: root.gitController ? root.gitController.inspector.sha : ""
        onCommitActivated: function(sha, shortSha, summary) {
            root.gitController.inspector.showCommit(root.gitController.historyEntry(sha));
        }
    }

    // ---- Commit: as mudancas por pasta e a caixa no pe' -------------------

    GitChangesList {
        id: changesView

        anchors.top: gitActions.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.bottom: commitRow.top
        anchors.bottomMargin: Theme.spacingSmall
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.leftMargin: Theme.spacingSmall
        anchors.rightMargin: Theme.spacingSmall
        visible: !root.historyVisible
        changesModel: root.gitController ? root.gitController.changesModel : null
        repo: root.gitController ? root.gitController.repo : false
        revision: root.gitController ? root.gitController.revision : 0
        selectedAbsPath: root.gitController ? root.gitController.inspector.path : ""
        onStageToggleRequested: function(index) { root.gitController.toggleStaged(index); }
        onFolderStageRequested: function(absPaths, stageAll) { root.gitController.stageFolder(absPaths, stageAll); }
        onSelectRequested: function(absPath, path) { root.gitController.inspector.showChange(absPath, path); }
        onDiffRequested: function(absPath) { root.gitController.showDiffOf(absPath); }
        onDiscardRequested: function(index) { root.gitController.openDiscardDialog(index); }
        onOpenRequested: function(absPath) { root.openRequested(absPath); }
    }

    GitCommitBox {
        id: commitRow

        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.spacingSmall
        height: implicitHeight
        historyVisible: root.historyVisible
        errorText: root.gitController ? root.gitController.lastMutationError : ""
        stagedCount: root.gitController ? root.gitController.stagedCount : 0
        amend: root.gitController ? root.gitController.amend : false
        remoteRunning: root.gitController ? root.gitController.remoteOperationRunning : false
        headPushed: root.gitController ? root.gitController.aheadCount === 0 : false
        branchLabel: root.gitController ? root.gitController.branchLabel : ""
        onCommitRequested: function(message) { root.gitController.commit(message); }
        onCommitAndPushRequested: function(message) { root.gitController.commitAndPush(message); }
        onAmendToggled: root.gitController.amend = !root.gitController.amend
    }
}
