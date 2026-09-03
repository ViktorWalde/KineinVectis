pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Aba Git do painel inferior (fatia M3.3): lista de mudanças com stage
// por clique, diff/descartar por linha e commit do que está staged.
// M3.4 soma a vista Histórico (commits; diff por clique) na mesma aba.
// Componente burro: estado entra por property, intenção sai por signal.
Item {
    id: panel

    ListModel {
        id: emptyChangesModel
    }

    ListModel {
        id: emptyHistoryModel
    }

    property var changesModel: emptyChangesModel
    property bool repo: false
    property int stagedCount: 0
    property string errorText: ""
    property var historyModel: emptyHistoryModel
    property bool historyVisible: false
    property bool historyLoading: false
    property string branchLabel: ""
    property var branchesModel
    property bool branchMenuVisible: false
    property bool remoteOperationRunning: false

    signal stageToggleRequested(int index)
    signal diffRequested(string absPath)
    signal discardRequested(int index)
    signal openRequested(string absPath)
    signal commitRequested(string message)
    signal changesViewRequested()
    signal historyViewRequested()
    signal historyRefreshRequested()
    signal commitActivated(string sha, string shortSha, string summary)
    signal branchMenuRequested()
    signal branchMenuDismissRequested()
    signal branchCheckoutRequested(string branch)
    signal branchCreateRequested(string name)
    signal remoteRequested(string operation)
    signal stashRequested(string action)

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
                { label: panel.branchLabel === "" ? qsTr("branch") : panel.branchLabel,
                  action: "branch" },
                { label: qsTr("pull"), action: "pull" },
                { label: qsTr("push"), action: "push" },
                { label: qsTr("stash"), action: "stash" },
                { label: qsTr("pop"), action: "pop" }
            ]

            delegate: Rectangle {
                id: gitActionChip

                required property var modelData

                width: actionLabel.width + 2 * Theme.spacingSmall
                height: 22
                radius: Theme.radiusXSmall
                opacity: panel.remoteOperationRunning
                         && (modelData.action === "pull" || modelData.action === "push")
                         ? 0.5 : 1.0
                color: actionArea.containsMouse ? Theme.surface2 : Theme.surface1
                border.color: modelData.action === "branch" ? Theme.accent : Theme.borderSoft
                border.width: 1

                Text {
                    id: actionLabel

                    anchors.centerIn: parent
                    text: gitActionChip.modelData.label
                    color: gitActionChip.modelData.action === "branch"
                           ? Theme.accent : Theme.textSecondary
                    font.pixelSize: 10
                    font.family: gitActionChip.modelData.action === "branch"
                                 ? Theme.monoFont : Theme.uiFont
                }

                MouseArea {
                    id: actionArea

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        const action = gitActionChip.modelData.action;
                        if (action === "branch") {
                            panel.branchMenuRequested();
                        } else if (action === "pull" || action === "push") {
                            panel.remoteRequested(action);
                        } else {
                            panel.stashRequested(action === "pop" ? "pop" : "push");
                        }
                    }
                }
            }
        }
    }

    GitBranchMenu {
        id: branchMenu

        anchors.top: gitActions.bottom
        anchors.left: parent.left
        anchors.topMargin: Theme.spacingXSmall
        visible: panel.branchMenuVisible

        branchesModel: panel.branchesModel

        onBranchCheckoutRequested: function(branch) { panel.branchCheckoutRequested(branch); }
        onBranchCreateRequested: function(name) { panel.branchCreateRequested(name); }
    }
    Row {
        id: viewsHeader

        anchors.top: gitActions.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.left: parent.left
        height: 20
        spacing: Theme.spacingSmall

        Rectangle {
            id: changesChip

            width: changesChipLabel.width + 2 * Theme.spacingSmall
            height: 20
            radius: Theme.radiusXSmall
            color: !panel.historyVisible ? Theme.surfaceSelected
                                         : (changesChipArea.containsMouse
                                            ? Theme.surface2 : Theme.surface1)
            border.color: Theme.borderSoft
            border.width: 1

            Text {
                id: changesChipLabel

                anchors.centerIn: parent
                text: qsTr("Mudanças")
                color: !panel.historyVisible ? Theme.textPrimary
                                             : Theme.textSecondary
                font.pixelSize: 10
            }

            MouseArea {
                id: changesChipArea

                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: panel.changesViewRequested()
            }
        }

        Rectangle {
            id: historyChip

            width: historyChipLabel.width + 2 * Theme.spacingSmall
            height: 20
            radius: Theme.radiusXSmall
            color: panel.historyVisible ? Theme.surfaceSelected
                                        : (historyChipArea.containsMouse
                                           ? Theme.surface2 : Theme.surface1)
            border.color: Theme.borderSoft
            border.width: 1

            Text {
                id: historyChipLabel

                anchors.centerIn: parent
                text: qsTr("Histórico")
                color: panel.historyVisible ? Theme.textPrimary
                                            : Theme.textSecondary
                font.pixelSize: 10
            }

            MouseArea {
                id: historyChipArea

                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: panel.historyViewRequested()
            }
        }

        Rectangle {
            id: historyRefreshChip

            width: historyRefreshLabel.width + 2 * Theme.spacingSmall
            height: 20
            radius: Theme.radiusXSmall
            visible: panel.historyVisible
            color: historyRefreshArea.containsMouse ? Theme.surface2
                                                    : "transparent"
            border.color: Theme.borderSoft
            border.width: 1

            Text {
                id: historyRefreshLabel

                anchors.centerIn: parent
                text: qsTr("atualizar")
                color: Theme.textSecondary
                font.pixelSize: 10
            }

            MouseArea {
                id: historyRefreshArea

                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: panel.historyRefreshRequested()
            }
        }
    }

    GitHistoryList {
        id: historyView

        anchors.top: viewsHeader.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        visible: panel.historyVisible

        historyModel: panel.historyModel
        historyLoading: panel.historyLoading
        repo: panel.repo

        onCommitActivated: function(sha, shortSha, summary) {
            panel.commitActivated(sha, shortSha, summary);
        }
    }
    GitChangesList {
        id: changesView

        anchors.top: viewsHeader.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.bottom: commitRow.top
        anchors.bottomMargin: Theme.spacingSmall
        anchors.left: parent.left
        anchors.right: parent.right
        visible: !panel.historyVisible

        changesModel: panel.changesModel
        repo: panel.repo

        onStageToggleRequested: function(index) { panel.stageToggleRequested(index); }
        onDiffRequested: function(absPath) { panel.diffRequested(absPath); }
        onDiscardRequested: function(index) { panel.discardRequested(index); }
        onOpenRequested: function(absPath) { panel.openRequested(absPath); }
    }
    GitCommitBox {
        id: commitRow

        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        height: 30

        historyVisible: panel.historyVisible
        errorText: panel.errorText
        stagedCount: panel.stagedCount

        onCommitRequested: function(message) { panel.commitRequested(message); }
    }
}
