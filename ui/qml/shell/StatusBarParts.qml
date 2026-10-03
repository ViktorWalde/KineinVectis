import QtQuick
import KineinVectis

// As pecas da barra de status (0.3.9), cada uma um Component por chave, para
// a barra desenha-las na ordem que o usuario arrastou. Sem `pragma
// ComponentBehavior: Bound` de proposito: o Loader do Qt 6.4 so' cria
// `sourceComponent` nascido fora de arquivo Bound (53 §5.2.1); o slot as cria
// com createObject, entregando `bar` e `strip` ja' na criacao — as pecas nao
// leem ids de fora.
Item {
    id: root

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

        // A TRILHA do arquivo ativo: "projeto › src › main.cpp" (0.3.9 —
        // subiu de cima do editor para ca', como na JetBrains). Sem arquivo,
        // so' o projeto. A pasta inteira fica na dica.
        Item {
            id: pathItem

            // Entregues na criacao pelo StatusBarSlot (createObject).
            property var bar: null
            property Item strip: null
            readonly property bool shown: pathItem.bar.workspaceRoot !== ""

            width: Math.min(trail.implicitWidth, Math.max(160, pathItem.strip.width * 0.45))
            height: trail.height
            clip: true

            EditorBreadcrumbs {
                id: trail

                anchors.verticalCenter: parent.verticalCenter
                path: pathItem.bar.workspaceName
                      + (pathItem.bar.breadcrumb !== "" ? "/" + pathItem.bar.breadcrumb : "")
            }

            HoverHandler {
                id: trailHover

                onHoveredChanged: {
                    if (hovered) TooltipController.showFor(pathItem, pathItem.bar.workspaceRoot, "top");
                    else TooltipController.hideFor(pathItem);
                }
            }
        }
    }

    Component {
        id: remotePart

        StatusBarRemoteWidget {
            id: remoteItem

            // Entregues na criacao pelo StatusBarSlot (createObject).
            property var bar: null
            property Item strip: null
            readonly property bool shown: hud.visible === true

            isMirror: remoteItem.bar.remoteIsMirror
            targetName: remoteItem.bar.remoteTarget
            syncing: remoteItem.bar.remoteSyncing
            syncDirection: remoteItem.bar.remoteSyncDirection
            syncFailed: remoteItem.bar.remoteSyncFailed
            syncMessage: remoteItem.bar.remoteSyncMessage
            deploying: remoteItem.bar.remoteDeploying
            probed: remoteItem.bar.remoteProbed
            probeOk: remoteItem.bar.remoteProbeOk
            probedAt: remoteItem.bar.remoteProbedAt
            onPanelRequested: remoteItem.bar.remotePanelRequested()
        }
    }

    // O job em curso ocupa o centro; sem job, os resumos do projeto.
    Component {
        id: activityPart

        Row {
            id: activity

            // O x do lugar na faixa (o slot o mantem): o resto da largura
            // dos resumos se mede a partir dele.
            property real slotX: 0
            // Entregues na criacao pelo StatusBarSlot (createObject).
            property var bar: null
            property Item strip: null
            readonly property bool shown: true

            spacing: Theme.spacingMedium

            StatusBarJobWidget {
                anchors.verticalCenter: parent.verticalCenter
                title: activity.bar.jobTitle
                progress: activity.bar.jobProgress
                message: activity.bar.jobMessage
                canCancel: activity.bar.jobCanCancel
                runningCount: activity.bar.jobCount
                onCancelRequested: activity.bar.cancelJobRequested()
                onJobsRequested: activity.bar.jobsRequested()
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                visible: activity.bar.running && activity.bar.jobTitle === ""
                text: qsTr("executando…")
                color: Theme.accent
                font.pixelSize: Theme.fontSizeStatus
            }

            StatusBarProjectSummaries {
                anchors.verticalCenter: parent.verticalCenter
                // O que sobra da faixa depois dos itens antes dele (que nao
                // dependem desta largura: sem laco de binding).
                availableWidth: activity.strip.width - activity.slotX - x
                visible: activity.bar.jobTitle === ""
                indexSummary: activity.bar.indexSummary
                contextSummary: activity.bar.contextSummary
                contextDetail: activity.bar.contextDetail
            }
        }
    }

    // A posicao do cursor, a esquerda do LSP (a referencia poe Ln:Col ali).
    Component {
        id: cursorPart

        Text {
            id: cursorItem

            // Entregues na criacao pelo StatusBarSlot (createObject).
            property var bar: null
            property Item strip: null
            readonly property bool shown: cursorItem.bar.cursorSummary !== ""

            text: cursorItem.bar.cursorSummary
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

            // Entregues na criacao pelo StatusBarSlot (createObject).
            property var bar: null
            property Item strip: null
            readonly property bool shown: lspText.bar.lspSummary !== ""

            text: lspText.bar.lspSummary
            color: lspText.bar.lspFailed ? Theme.errorSoft : Theme.textMuted
            font.pixelSize: Theme.fontSizeStatus

            MouseArea {
                anchors.fill: parent
                hoverEnabled: true
                acceptedButtons: Qt.NoButton
                onContainsMouseChanged: {
                    if (containsMouse && lspText.bar.lspDetail !== "") {
                        TooltipController.showFor(lspText, lspText.bar.lspDetail, "top");
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
            id: ideItem

            // Entregues na criacao pelo StatusBarSlot (createObject).
            property var bar: null
            property Item strip: null
            readonly property bool shown: true

            width: logsToggleText.width + 2 * Theme.spacingSmall
            height: 20
            radius: Theme.radius
            color: ideItem.bar.logsActive ? Theme.surfaceSelected
                                  : (logsToggleArea.containsMouse
                                     ? Theme.surface2 : "transparent")
            border.color: Theme.borderSoft
            border.width: 1

            Text {
                id: logsToggleText

                anchors.centerIn: parent
                text: qsTr("IDE")
                color: ideItem.bar.logsActive ? Theme.accent : Theme.textSecondary
                font.pixelSize: Theme.fontSizeStatus
            }

            MouseArea {
                id: logsToggleArea

                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: ideItem.bar.logsRequested()
            }
        }
    }

    Component {
        id: corePart

        Row {
            id: coreItem

            // Entregues na criacao pelo StatusBarSlot (createObject).
            property var bar: null
            property Item strip: null
            readonly property bool shown: true

            spacing: Theme.spacingSmall

            Rectangle {
                width: 7
                height: 7
                radius: width / 2
                anchors.verticalCenter: parent.verticalCenter
                color: coreItem.bar.coreConnected ? Theme.successSoft : Theme.errorSoft
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: coreItem.bar.coreConnected
                      ? qsTr("core · IPC %1").arg(coreItem.bar.coreProtocolVersion)
                      : qsTr("core %1").arg(coreItem.bar.coreStatus)
                color: Theme.textMuted
                font.pixelSize: Theme.fontSizeStatus
            }
        }
    }
}
