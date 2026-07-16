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
        commitInput.text = "";
    }

    function kindColor(kind) {
        if (kind === "conflicted") {
            return Theme.errorSoft;
        }
        if (kind === "untracked" || kind === "added") {
            return Theme.successSoft;
        }
        if (kind === "deleted") {
            return Theme.textDisabled;
        }
        return Theme.infoSoft;
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

    Rectangle {
        id: branchMenu

        anchors.top: gitActions.bottom
        anchors.left: parent.left
        anchors.topMargin: Theme.spacingXSmall
        width: Math.min(340, parent.width)
        height: 190
        z: 20
        visible: panel.branchMenuVisible
        radius: Theme.radius
        color: Theme.background2
        border.color: Theme.accent
        border.width: 1

        ListView {
            id: branchesView

            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: newBranchRow.top
            anchors.margins: Theme.spacingSmall
            clip: true
            model: panel.branchesModel

            delegate: Rectangle {
                id: branchRow

                required property string name
                required property bool current

                width: branchesView.width
                height: 24
                radius: Theme.radiusXSmall
                color: branchArea.containsMouse ? Theme.surface2 : "transparent"

                Row {
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.left: parent.left
                    anchors.leftMargin: Theme.spacingSmall
                    spacing: Theme.spacingXSmall

                    KvIcon {
                        anchors.verticalCenter: parent.verticalCenter
                        visible: branchRow.current
                        name: "check"
                        size: 14
                        success: true
                    }

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        text: branchRow.name
                        color: branchRow.current ? Theme.accent : Theme.textPrimary
                        font.family: Theme.monoFont
                        font.pixelSize: 11
                    }
                }

                MouseArea {
                    id: branchArea

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: branchRow.current ? Qt.ArrowCursor : Qt.PointingHandCursor
                    onClicked: {
                        if (!branchRow.current) {
                            panel.branchCheckoutRequested(branchRow.name);
                        }
                    }
                }
            }
        }

        Row {
            id: newBranchRow

            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            anchors.margins: Theme.spacingSmall
            height: 28
            spacing: Theme.spacingSmall

            Rectangle {
                width: parent.width - createBranchButton.width - Theme.spacingSmall
                height: parent.height
                radius: Theme.radiusXSmall
                color: Theme.background0
                border.color: newBranchInput.activeFocus ? Theme.accent : Theme.borderSoft
                border.width: 1

                TextInput {
                    id: newBranchInput

                    anchors.fill: parent
                    anchors.margins: Theme.spacingSmall
                    verticalAlignment: TextInput.AlignVCenter
                    color: Theme.textPrimary
                    font.family: Theme.monoFont
                    font.pixelSize: 10
                    clip: true
                    onAccepted: panel.branchCreateRequested(text)

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        visible: newBranchInput.text === ""
                        text: qsTr("nova branch")
                        color: Theme.textMuted
                        font.pixelSize: 10
                    }
                }
            }

            Rectangle {
                id: createBranchButton

                width: createBranchLabel.width + 2 * Theme.spacingSmall
                height: parent.height
                radius: Theme.radiusXSmall
                color: createBranchArea.pressed ? Theme.accentDim : Theme.accent

                Text {
                    id: createBranchLabel

                    anchors.centerIn: parent
                    text: qsTr("Criar")
                    color: Theme.background0
                    font.pixelSize: 10
                    font.bold: true
                }

                MouseArea {
                    id: createBranchArea

                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: panel.branchCreateRequested(newBranchInput.text)
                }
            }
        }
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

    ListView {
        id: historyView


        // B2 (docs/roadmaps/24): barra de rolagem. `parent: historyView` é OBRIGATÓRIO — um filho
        // declarado dentro de um ListView vira filho do contentItem e ROLARIA
        // junto com a lista. O ListView segue sendo a fonte da verdade.
        VerticalScrollBar {
            id: scrollBar_historyView

            parent: historyView
            anchors.right: historyView.right
            anchors.top: historyView.top
            anchors.bottom: historyView.bottom

            contentSize: historyView.contentHeight
            viewportSize: historyView.height
            position: historyView.contentY

            onMoveRequested: function(position) {
                historyView.contentY = position;
            }
        }
        anchors.top: viewsHeader.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        clip: true
        visible: panel.historyVisible
        model: panel.historyModel

        Text {
            anchors.centerIn: parent
            visible: panel.historyModel.count === 0
            text: panel.historyLoading
                  ? qsTr("Carregando histórico...")
                  : (panel.repo
                     ? qsTr("Sem commits ainda.")
                     : qsTr("Este workspace não é um repositório git."))
            color: Theme.textMuted
            font.pixelSize: 11
        }

        delegate: Rectangle {
            id: commitRowItem

            required property string sha
            required property string shortSha
            required property string author
            required property string age
            required property string summary

            width: historyView.width
            height: 24
            radius: Theme.radiusXSmall
            color: commitRowArea.containsMouse ? Theme.surface2 : "transparent"

            Text {
                id: commitShaText

                anchors.verticalCenter: parent.verticalCenter
                anchors.left: parent.left
                anchors.leftMargin: Theme.spacingSmall
                text: commitRowItem.shortSha
                color: Theme.accent
                font.pixelSize: 11
                font.family: Theme.monoFont
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                anchors.left: commitShaText.right
                anchors.leftMargin: Theme.spacingSmall
                anchors.right: commitMetaText.left
                anchors.rightMargin: Theme.spacingSmall
                text: commitRowItem.summary
                color: Theme.textPrimary
                font.pixelSize: 11
                elide: Text.ElideRight
            }

            Text {
                id: commitMetaText

                anchors.verticalCenter: parent.verticalCenter
                anchors.right: parent.right
                anchors.rightMargin: Theme.spacingSmall
                text: commitRowItem.author + ", " + commitRowItem.age
                color: Theme.textMuted
                font.pixelSize: 10
            }

            MouseArea {
                id: commitRowArea

                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: panel.commitActivated(commitRowItem.sha,
                                                 commitRowItem.shortSha,
                                                 commitRowItem.summary)
            }
        }
    }

    ListView {
        id: changesView


        // B2 (docs/roadmaps/24): barra de rolagem. `parent: changesView` é OBRIGATÓRIO — um filho
        // declarado dentro de um ListView vira filho do contentItem e ROLARIA
        // junto com a lista. O ListView segue sendo a fonte da verdade.
        VerticalScrollBar {
            id: scrollBar_changesView

            parent: changesView
            anchors.right: changesView.right
            anchors.top: changesView.top
            anchors.bottom: changesView.bottom

            contentSize: changesView.contentHeight
            viewportSize: changesView.height
            position: changesView.contentY

            onMoveRequested: function(position) {
                changesView.contentY = position;
            }
        }
        anchors.top: viewsHeader.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.bottom: commitRow.top
        anchors.bottomMargin: Theme.spacingSmall
        anchors.left: parent.left
        anchors.right: parent.right
        clip: true
        visible: !panel.historyVisible
        model: panel.changesModel

        Text {
            anchors.centerIn: parent
            visible: panel.changesModel.count === 0
            text: panel.repo
                  ? qsTr("Sem mudanças — árvore limpa.")
                  : qsTr("Este workspace não é um repositório git.")
            color: Theme.textMuted
            font.pixelSize: 11
        }

        delegate: Rectangle {
            id: changeRow

            required property int index
            required property string path
            required property string absPath
            required property string kind
            required property bool staged

            width: changesView.width
            height: 24
            radius: Theme.radiusXSmall
            color: changeRowArea.containsMouse ? Theme.surface2 : "transparent"

            Rectangle {
                id: stageBox

                anchors.verticalCenter: parent.verticalCenter
                anchors.left: parent.left
                anchors.leftMargin: Theme.spacingSmall
                width: 14
                height: 14
                radius: Theme.radiusXSmall
                color: changeRow.staged ? Theme.accentDim : "transparent"
                border.color: changeRow.staged ? Theme.accent : Theme.borderStrong
                border.width: 1

                KvIcon {
                    anchors.centerIn: parent
                    visible: changeRow.staged
                    name: "check"
                    size: 11
                    active: true
                }

                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: panel.stageToggleRequested(changeRow.index)
                }
            }

            Text {
                id: changePathText

                anchors.verticalCenter: parent.verticalCenter
                anchors.left: stageBox.right
                anchors.leftMargin: Theme.spacingSmall
                anchors.right: diffChip.left
                anchors.rightMargin: Theme.spacingSmall
                text: changeRow.path
                color: panel.kindColor(changeRow.kind)
                font.pixelSize: 11
                font.family: Theme.monoFont
                elide: Text.ElideMiddle
            }

            MouseArea {
                id: changeRowArea

                anchors.fill: parent
                hoverEnabled: true
                z: -1
                onClicked: panel.openRequested(changeRow.absPath)
            }

            Rectangle {
                id: diffChip

                anchors.verticalCenter: parent.verticalCenter
                anchors.right: discardChip.left
                anchors.rightMargin: Theme.spacingXSmall
                width: diffChipLabel.width + 2 * Theme.spacingSmall
                height: 18
                radius: Theme.radiusXSmall
                visible: changeRowArea.containsMouse || diffChipArea.containsMouse
                         || discardChipArea.containsMouse
                color: diffChipArea.containsMouse ? Theme.surfaceSelected : Theme.surface1
                border.color: Theme.borderSoft
                border.width: 1

                Text {
                    id: diffChipLabel

                    anchors.centerIn: parent
                    text: qsTr("diff")
                    color: Theme.textSecondary
                    font.pixelSize: 9
                }

                MouseArea {
                    id: diffChipArea

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: panel.diffRequested(changeRow.absPath)
                }
            }

            Rectangle {
                id: discardChip

                anchors.verticalCenter: parent.verticalCenter
                anchors.right: parent.right
                anchors.rightMargin: Theme.spacingSmall
                width: 18
                height: 18
                radius: Theme.radiusXSmall
                visible: diffChip.visible
                color: discardChipArea.containsMouse ? Theme.surfaceSelected : Theme.surface1
                border.color: Theme.borderSoft
                border.width: 1

                Text {
                    anchors.centerIn: parent
                    text: "↩"
                    color: Theme.errorSoft
                    font.pixelSize: 10
                }

                MouseArea {
                    id: discardChipArea

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: panel.discardRequested(changeRow.index)
                }
            }
        }
    }

    Text {
        anchors.bottom: commitRow.top
        anchors.bottomMargin: 2 * Theme.spacingSmall + 2
        anchors.left: parent.left
        anchors.right: parent.right
        visible: panel.errorText !== "" && !panel.historyVisible
        text: panel.errorText
        color: Theme.errorSoft
        font.pixelSize: 10
        elide: Text.ElideRight
    }

    Row {
        id: commitRow

        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        height: 30
        spacing: Theme.spacingSmall
        visible: !panel.historyVisible

        Rectangle {
            width: parent.width - commitButton.width - Theme.spacingSmall
            height: 30
            radius: Theme.radius
            color: Theme.background0
            border.color: commitInput.activeFocus ? Theme.accent : Theme.borderSoft
            border.width: 1

            TextInput {
                id: commitInput

                anchors.fill: parent
                anchors.leftMargin: Theme.spacingSmall
                anchors.rightMargin: Theme.spacingSmall
                verticalAlignment: TextInput.AlignVCenter
                color: Theme.textPrimary
                font.pixelSize: 12
                clip: true
                selectByMouse: true
                onAccepted: {
                    panel.commitRequested(commitInput.text);
                    commitInput.text = "";
                }

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    visible: commitInput.text === ""
                    text: qsTr("Mensagem do commit...")
                    color: Theme.textMuted
                    font.pixelSize: 12
                }
            }
        }

        Rectangle {
            id: commitButton

            readonly property bool commitEnabled: panel.stagedCount > 0
                && commitInput.text.trim() !== ""

            anchors.verticalCenter: parent.verticalCenter
            width: commitLabel.width + 2 * Theme.spacingMedium
            height: 30
            radius: Theme.radius
            opacity: commitEnabled ? 1.0 : 0.5
            color: commitEnabled
                   ? (commitButtonArea.pressed ? Theme.accentDim : Theme.accent)
                   : Theme.surface1
            border.color: commitEnabled ? "transparent" : Theme.borderSoft
            border.width: commitEnabled ? 0 : 1

            Text {
                id: commitLabel

                anchors.centerIn: parent
                text: panel.stagedCount > 0
                      ? qsTr("Commit (%1)").arg(panel.stagedCount)
                      : qsTr("Commit")
                color: commitButton.commitEnabled
                       ? Theme.background0 : Theme.textMuted
                font.pixelSize: 12
                font.bold: true
            }

            MouseArea {
                id: commitButtonArea

                anchors.fill: parent
                cursorShape: commitButton.commitEnabled
                             ? Qt.PointingHandCursor : Qt.ArrowCursor
                onClicked: {
                    if (commitButton.commitEnabled) {
                        panel.commitRequested(commitInput.text);
                        commitInput.text = "";
                    }
                }
            }
        }
    }
}
