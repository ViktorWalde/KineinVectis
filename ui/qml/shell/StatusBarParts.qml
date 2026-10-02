import QtQuick
import KineinVectis

// As pecas da barra de status (0.3.9), cada uma um Component por chave, para
// a barra desenha-las na ordem que o usuario arrastou. Sem `pragma
// ComponentBehavior: Bound` de proposito: o Loader do Qt 6.4 so' cria
// `sourceComponent` nascido fora de arquivo Bound (53 §5.2.1).
Item {
    id: root

    property var bar: null
    property Item leftStrip: null

    visible: false

    // As pecas da barra, por chave. Cada uma diz se aparece (`shown`): o
    // Loader que a carrega fica invisivel junto, e o Row nao lhe da' espaco.
    // (O `visible` do Loader nao pode ler o `visible` da peca: o efetivo
    // dela depende do Loader — seria um laco.)
    readonly property var byKey: ({
        path: pathPart, remote: remotePart, activity: activityPart,
        cursor: cursorPart, lsp: lspPart, ide: idePart, core: corePart
    })

    Component {
        id: pathPart

        Text {
            readonly property bool shown: root.bar.workspaceRoot !== ""

            width: Math.min(implicitWidth, Math.max(120, root.leftStrip.width * 0.32))
            elide: Text.ElideMiddle
            text: root.bar.workspaceRoot
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeStatus
            font.family: Theme.monoFont
        }
    }

    Component {
        id: remotePart

        StatusBarRemoteWidget {
            readonly property bool shown: hud.visible === true

            isMirror: root.bar.remoteIsMirror
            targetName: root.bar.remoteTarget
            syncing: root.bar.remoteSyncing
            syncDirection: root.bar.remoteSyncDirection
            syncFailed: root.bar.remoteSyncFailed
            syncMessage: root.bar.remoteSyncMessage
            deploying: root.bar.remoteDeploying
            probed: root.bar.remoteProbed
            probeOk: root.bar.remoteProbeOk
            probedAt: root.bar.remoteProbedAt
            onPanelRequested: root.bar.remotePanelRequested()
        }
    }

    // O job em curso ocupa o centro; sem job, os resumos do projeto.
    Component {
        id: activityPart

        Row {
            id: activity

            readonly property bool shown: true

            spacing: Theme.spacingMedium

            StatusBarJobWidget {
                anchors.verticalCenter: parent.verticalCenter
                title: root.bar.jobTitle
                progress: root.bar.jobProgress
                message: root.bar.jobMessage
                canCancel: root.bar.jobCanCancel
                runningCount: root.bar.jobCount
                onCancelRequested: root.bar.cancelJobRequested()
                onJobsRequested: root.bar.jobsRequested()
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                visible: root.bar.running && root.bar.jobTitle === ""
                text: qsTr("executando…")
                color: Theme.accent
                font.pixelSize: Theme.fontSizeStatus
            }

            StatusBarProjectSummaries {
                anchors.verticalCenter: parent.verticalCenter
                // O que sobra da faixa depois dos itens antes dele (que nao
                // dependem desta largura: sem laco de binding).
                availableWidth: root.leftStrip.width - activity.parent.x - x
                visible: root.bar.jobTitle === ""
                indexSummary: root.bar.indexSummary
                contextSummary: root.bar.contextSummary
                contextDetail: root.bar.contextDetail
            }
        }
    }

    // A posicao do cursor, a esquerda do LSP (a referencia poe Ln:Col ali).
    Component {
        id: cursorPart

        Text {
            readonly property bool shown: root.bar.cursorSummary !== ""

            text: root.bar.cursorSummary
            color: Theme.textMuted
            font.family: Theme.monoFont
            font.pixelSize: Theme.fontSizeStatus
        }
    }

    // Os servidores de linguagem: ● todos rodando, … subindo, ✗ caiu.
    Component {
        id: lspPart

        Text {
            id: lspText

            readonly property bool shown: root.bar.lspSummary !== ""

            text: root.bar.lspSummary
            color: root.bar.lspFailed ? Theme.errorSoft : Theme.textMuted
            font.pixelSize: Theme.fontSizeStatus

            MouseArea {
                anchors.fill: parent
                hoverEnabled: true
                acceptedButtons: Qt.NoButton
                onContainsMouseChanged: {
                    if (containsMouse && root.bar.lspDetail !== "") {
                        TooltipController.showFor(lspText, root.bar.lspDetail, "top");
                    } else {
                        TooltipController.hideFor(lspText);
                    }
                }
            }
        }
    }

    Component {
        id: idePart

        Rectangle {
            readonly property bool shown: true

            width: logsToggleText.width + 2 * Theme.spacingSmall
            height: 20
            radius: Theme.radius
            color: root.bar.logsActive ? Theme.surfaceSelected
                                  : (logsToggleArea.containsMouse
                                     ? Theme.surface2 : "transparent")
            border.color: Theme.borderSoft
            border.width: 1

            Text {
                id: logsToggleText

                anchors.centerIn: parent
                text: qsTr("IDE")
                color: root.bar.logsActive ? Theme.accent : Theme.textSecondary
                font.pixelSize: Theme.fontSizeStatus
            }

            MouseArea {
                id: logsToggleArea

                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: root.bar.logsRequested()
            }
        }
    }

    Component {
        id: corePart

        Row {
            readonly property bool shown: true

            spacing: Theme.spacingSmall

            Rectangle {
                width: 7
                height: 7
                radius: width / 2
                anchors.verticalCenter: parent.verticalCenter
                color: root.bar.coreConnected ? Theme.successSoft : Theme.errorSoft
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: root.bar.coreConnected
                      ? qsTr("core · IPC %1").arg(root.bar.coreProtocolVersion)
                      : qsTr("core %1").arg(root.bar.coreStatus)
                color: Theme.textMuted
                font.pixelSize: Theme.fontSizeStatus
            }
        }
    }
}
