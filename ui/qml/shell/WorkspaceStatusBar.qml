import QtQuick
import KineinVectis

Rectangle {
    id: bar

    property string workspaceRoot: ""
    property string workspaceKindLabel: ""
    property bool logsActive: false
    property bool building: false
    property bool testing: false
    property bool analyzing: false
    property bool scanningEnvironment: false
    property bool running: false
    property bool coreConnected: false
    property string coreProtocolVersion: ""
    property string coreStatus: ""
    property string gitBranchLabel: ""
    property int gitAheadCount: 0
    property int gitBehindCount: 0
    property int gitChangeCount: 0
    // Toolchain (roadmap 30, etapa 5). Vazio esconde o chip: projeto sem
    // build system nao tem o que escolher.
    property string toolchainSummary: ""
    property bool toolchainVisible: false
    // O indice do projeto inteiro: "N arquivos · N linhas · N simbolos", ou
    // o progresso enquanto constroi. Vazio = nada a mostrar.
    property string indexSummary: ""
    // O contexto de compilador do arquivo ATIVO: "c++ · gnu++23 · 12 -I · 9 -D",
    // "cargo · kinein-core (lib, 2024)", "python · .venv · 3.12.3". Vazio =
    // sem arquivo ou sem o que dizer; o detalhe aparece ao pairar.
    property string contextSummary: ""
    property string contextDetail: ""
    // O Python do projeto (PythonController.summary()); vazio fora de Python.
    property string pythonSummary: ""

    signal logsRequested()
    signal toolchainMenuRequested(real menuX, real menuY)
    signal cancelBuildRequested()
    signal cancelTestsRequested()
    signal cancelQualityRequested()
    signal cancelEnvironmentScanRequested()

    height: 28
    color: Theme.background1

    // A faixa da esquerda para ANTES da direita: com o workspace, a
    // toolchain, o indice e o git, a 1280 px ela invadia o botao "IDE"
    // (medido na foto de 2026-09-18, pente-fino/Etapa 2). O caminho do
    // workspace e' o que cede: elide no meio.
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

        // O git antes dos resumos do indice: quando falta espaco, o que cede
        // e' a contagem de simbolos, nao a branch.
        Text {
            anchors.verticalCenter: parent.verticalCenter
            visible: bar.gitBranchLabel !== ""
            text: {
                let label = "⎇ " + bar.gitBranchLabel;
                if (bar.gitAheadCount > 0) {
                    label += " ↑" + bar.gitAheadCount;
                }
                if (bar.gitBehindCount > 0) {
                    label += " ↓" + bar.gitBehindCount;
                }
                if (bar.gitChangeCount === 1) {
                    label += qsTr("  ·  1 alteração");
                } else if (bar.gitChangeCount > 1) {
                    label += qsTr("  ·  %1 alterações").arg(bar.gitChangeCount);
                }
                return label;
            }
            color: bar.gitChangeCount > 0 ? Theme.textSecondary : Theme.textMuted
            font.pixelSize: Theme.fontSizeStatus
            font.family: Theme.monoFont
        }
        // Os resumos do projeto (indice, contexto, python): dono proprio.
        StatusBarProjectSummaries {
            anchors.verticalCenter: parent.verticalCenter
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

        Row {
            anchors.verticalCenter: parent.verticalCenter
            visible: bar.building
            spacing: Theme.spacingSmall

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: qsTr("compilando...")
                color: Theme.accent
                font.pixelSize: Theme.fontSizeStatus
            }

            KvIconButton {
                anchors.verticalCenter: parent.verticalCenter
                compact: true
                iconName: "close"
                danger: true
                tooltip: qsTr("Cancelar build")
                onClicked: bar.cancelBuildRequested()
            }
        }

        Row {
            anchors.verticalCenter: parent.verticalCenter
            visible: bar.testing
            spacing: Theme.spacingSmall

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: qsTr("testando...")
                color: Theme.accent
                font.pixelSize: Theme.fontSizeStatus
            }

            KvIconButton {
                anchors.verticalCenter: parent.verticalCenter
                compact: true
                iconName: "close"
                danger: true
                tooltip: qsTr("Cancelar testes")
                onClicked: bar.cancelTestsRequested()
            }
        }

        Row {
            anchors.verticalCenter: parent.verticalCenter
            visible: bar.analyzing
            spacing: Theme.spacingSmall

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: qsTr("analisando...")
                color: Theme.accent
                font.pixelSize: Theme.fontSizeStatus
            }

            KvIconButton {
                anchors.verticalCenter: parent.verticalCenter
                compact: true
                iconName: "close"
                danger: true
                tooltip: qsTr("Cancelar análise")
                onClicked: bar.cancelQualityRequested()
            }
        }

        Row {
            anchors.verticalCenter: parent.verticalCenter
            visible: bar.scanningEnvironment
            spacing: Theme.spacingSmall

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: qsTr("scan de ambiente...")
                color: Theme.accent
                font.pixelSize: Theme.fontSizeStatus
            }

            KvIconButton {
                anchors.verticalCenter: parent.verticalCenter
                compact: true
                iconName: "close"
                danger: true
                tooltip: qsTr("Cancelar scan")
                onClicked: bar.cancelEnvironmentScanRequested()
            }
        }

        Text {
            anchors.verticalCenter: parent.verticalCenter
            visible: bar.running
            text: qsTr("executando...")
            color: Theme.accent
            font.pixelSize: Theme.fontSizeStatus
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
                  ? qsTr("core conectado · IPC %1").arg(bar.coreProtocolVersion)
                  : qsTr("core %1").arg(bar.coreStatus)
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeStatus
        }
    }
}
