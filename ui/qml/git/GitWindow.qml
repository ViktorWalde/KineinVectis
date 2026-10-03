pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A janela do Git EM PE', a esquerda (Etapa 3, E3-3 — roadmaps/44 §4.1;
// pedido do autor: "o posicionamento nao ta' memoria muscular JetBrains").
// Alterna com o explorer no mesmo slot. Abas Commit | Log em cima; a
// linha do branch (pull/push/stash/pop) inteira, porque o minimo da janela a
// comporta (2026-10-03); Commit = a lista de mudancas por pasta + a caixa de commit no pe';
// Log = o filtro + o grafo. O que esta' selecionado abre NO EDITOR
// (GitViewerPane), nao aqui. Fala com o GitController; abrir arquivo e
// fechar a janela saem por sinal.
Rectangle {
    id: root

    property var gitController: null

    signal openRequested(string absPath)
    signal closeRequested()

    readonly property bool historyVisible: gitController ? gitController.historyVisible : false

    // Dentro da ilha unica (0.3.9): sem borda propria.
    radius: Theme.radiusLarge
    color: Theme.background1

    // O minimo que o slot da esquerda precisa com o Git nele (53 §4.4): a
    // MAIOR das tres linhas, cada uma inteira numa linha so' — o cabecalho,
    // o branch com pull/push/stash/pop e o rodape do commit (2026-10-03,
    // pedido do autor, com a foto de referencia: nada quebra, nada some).
    readonly property real headerMinimumWidth: titleLabel.implicitWidth + commitTab.width + logTab.width
                                              + refreshButton.width + closeButton.width + 5 * headerRow.spacing
    readonly property real minimumWidth: Math.max(headerMinimumWidth, branchRowWidth(),
                                                  commitRow.minimumWidth) + 2 * Theme.spacingSmall

    // A linha do branch inteira: as fichas e os espacos entre elas.
    function branchRowWidth() {
        let total = 0;
        let count = 0;
        for (let i = 0; i < gitActions.children.length; i++) {
            const chip = gitActions.children[i];
            if (chip.modelData === undefined) continue;
            total += chip.width;
            count += 1;
        }
        return total + Math.max(0, count - 1) * gitActions.spacing;
    }

    function clearMessage() {
        commitRow.clearMessage();
    }

    // ---- o cabecalho: titulo, as duas abas, atualizar, fechar -------------

    Row {
        id: headerRow

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.spacingSmall
        height: 24
        spacing: Theme.spacingSmall

        // O titulo fica sempre: o minimo da janela ja' o inclui (2026-10-03).
        // Sem `width` explicito, para nao amarrar a largura do Text ao
        // proprio implicitWidth ("Binding loop detected", 0.3.6, 53 §5.2).
        Text {
            id: titleLabel

            anchors.verticalCenter: parent.verticalCenter
            text: qsTr("Git")
            color: Theme.textPrimary
            font.pixelSize: Theme.fontSizeBody
            font.weight: Font.DemiBold
        }

        KvToggleChip {
            id: commitTab

            anchors.verticalCenter: parent.verticalCenter
            labelText: root.gitController && root.gitController.changeCount > 0
                       ? qsTr("Commit (%1)").arg(root.gitController.changeCount) : qsTr("Commit")
            active: !root.historyVisible
            onToggled: root.gitController.showChanges()
        }

        KvToggleChip {
            id: logTab

            anchors.verticalCenter: parent.verticalCenter
            labelText: qsTr("Log")
            active: root.historyVisible
            onToggled: root.gitController.openHistory()
        }

        Item {
            // O titulo so' ocupa largura e um espacamento quando visivel.
            width: Math.max(0, parent.width - titleLabel.implicitWidth - parent.spacing
                            - commitTab.width - logTab.width
                            - refreshButton.width - closeButton.width - 4 * parent.spacing)
            height: 1
        }

        KvIconButton {
            id: refreshButton

            anchors.verticalCenter: parent.verticalCenter
            compact: true
            iconName: "refresh"
            tooltip: root.historyVisible ? qsTr("Atualizar o histórico") : qsTr("Atualizar as mudanças")
            onClicked: root.historyVisible ? root.gitController.refreshHistory() : root.gitController.refresh()
        }

        KvIconButton {
            id: closeButton

            anchors.verticalCenter: parent.verticalCenter
            compact: true
            iconName: "close"
            tooltip: qsTr("Fechar a janela do Git")
            onClicked: root.closeRequested()
        }
    }

    // ---- a linha do branch, inteira (o minimo da janela a comporta) --------

    Flow {
        id: gitActions

        anchors.top: headerRow.bottom
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

                // Um branch de nome comprido fica em ate' 140 px, com "…": a
                // linha nao alarga a janela sem limite (o nome inteiro aparece
                // no menu de branches, que este chip abre).
                Text {
                    id: actionLabel

                    anchors.centerIn: parent
                    width: Math.min(implicitWidth, 140)
                    elide: Text.ElideRight
                    text: gitActionChip.modelData.label
                    color: gitActionChip.modelData.action === "branch" ? Theme.accent : Theme.textSecondary
                    font.pixelSize: Theme.fontSizeCaption
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
