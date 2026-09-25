import QtQuick
import KineinVectis

// A barra de status (Etapa 2, F2 do roadmaps/43, 2026-09-18): ela diz O QUE
// ESTA' ACONTECENDO. Esquerda: o workspace e a toolchain. Centro: o job em
// curso com progresso e cancelar (build, testes, indice, rsync — o
// JobsController ja' sabia, a barra nao dizia); sem job, os resumos do
// projeto. Direita: o contexto do arquivo ativo, o Python, os servidores de
// linguagem (que antes so' o log via), o botao IDE e o core. O git saiu
// daqui: mora no widget da barra principal (F1). A esquerda para antes da
// direita — a colisao a 1280 px que a foto 02 mostrou.
Rectangle {
    id: bar

    property string workspaceRoot: ""
    property string workspaceKindLabel: ""
    property bool logsActive: false
    property bool running: false
    property bool coreConnected: false
    property string coreProtocolVersion: ""
    property string coreStatus: ""
    property string toolchainSummary: ""
    property bool toolchainVisible: false
    property string indexSummary: ""
    property string contextSummary: ""
    property string contextDetail: ""
    property string pythonSummary: ""
    // O job em curso (ActiveJobController).
    property string jobTitle: ""
    property real jobProgress: -1
    property string jobMessage: ""
    property bool jobCanCancel: false
    property int jobCount: 0
    // Os servidores de linguagem (LspStatusController).
    property string lspSummary: ""
    // "Ln:Col" do arquivo ativo (fechamento da Etapa 2, 2026-09-18); vazio sem arquivo.
    property string cursorSummary: ""
    property string lspDetail: ""
    property bool lspFailed: false

    // O Remote so' aparece quando o workspace E' espelhado (§5.2 da
    // especificacao): um perfil apenas selecionado nao ocupa a barra.
    property bool remoteIsMirror: false
    property string remoteTarget: ""
    property bool remoteSyncing: false
    property string remoteSyncDirection: ""
    property bool remoteSyncFailed: false
    property string remoteSyncMessage: ""
    property bool remoteDeploying: false
    property bool remoteProbed: false
    property bool remoteProbeOk: false
    property double remoteProbedAt: 0

    signal remotePanelRequested()
    signal logsRequested()
    signal jobsRequested()
    signal toolchainMenuRequested(real menuX, real menuY)
    signal cancelJobRequested()

    height: 28
    color: Theme.background1

    Row {
        id: faixaEsquerda

        anchors.verticalCenter: parent.verticalCenter
        anchors.left: parent.left
        anchors.leftMargin: Theme.spacingMedium
        anchors.right: faixaDireita.left
        anchors.rightMargin: Theme.spacingMedium
        spacing: Theme.spacingMedium
        clip: true

        Text {
            anchors.verticalCenter: parent.verticalCenter
            visible: bar.workspaceRoot !== ""
            width: Math.min(implicitWidth, Math.max(120, faixaEsquerda.width * 0.32))
            elide: Text.ElideMiddle
            text: bar.workspaceKindLabel + "  ·  " + bar.workspaceRoot
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeStatus
            font.family: Theme.monoFont
        }

        // Chip da toolchain: diz o que vai rodar e abre o seletor.
        Rectangle {
            id: toolchainChip

            anchors.verticalCenter: parent.verticalCenter
            visible: bar.toolchainVisible
            width: toolchainTexto.width + 2 * Theme.spacingSmall
            height: 18
            radius: Theme.radiusXSmall
            color: toolchainArea.containsMouse ? Theme.surface2 : "transparent"
            border.color: Theme.borderSoft
            border.width: 1

            Text {
                id: toolchainTexto

                anchors.centerIn: parent
                text: qsTr("toolchain: %1").arg(bar.toolchainSummary)
                color: Theme.textMuted
                font.pixelSize: Theme.fontSizeStatus
            }

            MouseArea {
                id: toolchainArea

                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: {
                    const pos = toolchainChip.mapToItem(bar, 0, 0);
                    bar.toolchainMenuRequested(pos.x, pos.y);
                }
            }
        }

        StatusBarRemoteWidget {
            anchors.verticalCenter: parent.verticalCenter
            isMirror: bar.remoteIsMirror
            targetName: bar.remoteTarget
            syncing: bar.remoteSyncing
            syncDirection: bar.remoteSyncDirection
            syncFailed: bar.remoteSyncFailed
            syncMessage: bar.remoteSyncMessage
            deploying: bar.remoteDeploying
            probed: bar.remoteProbed
            probeOk: bar.remoteProbeOk
            probedAt: bar.remoteProbedAt
            onPanelRequested: bar.remotePanelRequested()
        }

        // O job em curso ocupa o centro; sem job, os resumos do projeto.
        StatusBarJobWidget {
            anchors.verticalCenter: parent.verticalCenter
            title: bar.jobTitle
            progress: bar.jobProgress
            message: bar.jobMessage
            canCancel: bar.jobCanCancel
            runningCount: bar.jobCount
            onCancelRequested: bar.cancelJobRequested()
            onJobsRequested: bar.jobsRequested()
        }

        Text {
            anchors.verticalCenter: parent.verticalCenter
            visible: bar.running && bar.jobTitle === ""
            text: qsTr("executando…")
            color: Theme.accent
            font.pixelSize: Theme.fontSizeStatus
        }

        StatusBarProjectSummaries {
            anchors.verticalCenter: parent.verticalCenter
            visible: bar.jobTitle === ""
            indexSummary: bar.indexSummary
            contextSummary: bar.contextSummary
            contextDetail: bar.contextDetail
            pythonSummary: bar.pythonSummary
        }
    }

    Row {
        id: faixaDireita

        anchors.verticalCenter: parent.verticalCenter
        anchors.right: parent.right
        anchors.rightMargin: Theme.spacingMedium
        spacing: Theme.spacingMedium

        // A posicao do cursor, a esquerda do LSP (a referencia poe Ln:Col ali).
        Text {
            anchors.verticalCenter: parent.verticalCenter
            visible: bar.cursorSummary !== ""
            text: bar.cursorSummary
            color: Theme.textMuted
            font.family: Theme.monoFont
            font.pixelSize: Theme.fontSizeStatus
        }

        // Os servidores de linguagem: ● todos rodando, … subindo, ✗ caiu.
        Text {
            id: lspTexto

            anchors.verticalCenter: parent.verticalCenter
            visible: bar.lspSummary !== ""
            text: bar.lspSummary
            color: bar.lspFailed ? Theme.errorSoft : Theme.textMuted
            font.pixelSize: Theme.fontSizeStatus

            MouseArea {
                id: lspArea

                anchors.fill: parent
                hoverEnabled: true
                acceptedButtons: Qt.NoButton
                onContainsMouseChanged: {
                    if (containsMouse && bar.lspDetail !== "") {
                        TooltipController.showFor(lspTexto, bar.lspDetail, "top");
                    } else {
                        TooltipController.hideFor(lspTexto);
                    }
                }
            }
        }

        Rectangle {
            anchors.verticalCenter: parent.verticalCenter
            width: logsToggleText.width + 2 * Theme.spacingSmall
            height: 20
            radius: Theme.radius
            color: bar.logsActive ? Theme.surfaceSelected
                                  : (logsToggleArea.containsMouse
                                     ? Theme.surface2 : "transparent")
            border.color: Theme.borderSoft
            border.width: 1

            Text {
                id: logsToggleText

                anchors.centerIn: parent
                text: qsTr("IDE")
                color: bar.logsActive ? Theme.accent : Theme.textSecondary
                font.pixelSize: Theme.fontSizeStatus
            }

            MouseArea {
                id: logsToggleArea

                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: bar.logsRequested()
            }
        }

        Rectangle {
            width: 7
            height: 7
            radius: 4
            anchors.verticalCenter: parent.verticalCenter
            color: bar.coreConnected ? Theme.successSoft : Theme.errorSoft
        }

        Text {
            anchors.verticalCenter: parent.verticalCenter
            text: bar.coreConnected
                  ? qsTr("core · IPC %1").arg(bar.coreProtocolVersion)
                  : qsTr("core %1").arg(bar.coreStatus)
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeStatus
        }
    }
}
