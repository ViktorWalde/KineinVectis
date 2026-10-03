pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A barra de status (Etapa 2, F2 do roadmaps/43, 2026-09-18): ela diz O QUE
// ESTA' ACONTECENDO. Esquerda: a pasta do projeto (o sistema de build e a
// toolchain subiram ao cabecalho na 0.3.8 F3 — eram repetidos aqui). Centro: o job em
// curso com progresso e cancelar (build, testes, indice, rsync — o
// JobsController ja' sabia, a barra nao dizia); sem job, os resumos do
// projeto. Direita: o contexto do arquivo ativo, o Python, os servidores de
// linguagem (que antes so' o log via), o botao IDE e o core. O git saiu
// daqui: mora no widget da barra principal (F1). A esquerda para antes da
// direita — a colisao a 1280 px que a foto 02 mostrou.
Rectangle {
    id: bar

    property string workspaceRoot: ""
    property string workspaceName: ""
    // O arquivo ativo relativo ao projeto ("src/main.cpp"); vazio sem arquivo.
    property string breadcrumb: ""
    property bool logsActive: false
    property bool running: false
    property bool coreConnected: false
    property string coreProtocolVersion: ""
    property string coreStatus: ""
    property string indexSummary: ""
    property string contextSummary: ""
    property string contextDetail: ""
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
    signal cancelJobRequested()
    // Arrastar para reordenar (0.3.9): cada faixa so' dentro dela mesma.
    property var leftOrder: []
    property var rightOrder: []
    signal itemMoved(string strip, string key, int dropIndex, var visibleKeys)
    readonly property ShellLayoutCodec codec: ShellLayoutCodec {}

    height: 28
    // Moldura, como a faixa de menus: sobre o fundo da janela (0.3.9).
    color: Theme.frame

    StatusBarParts {
        id: statusParts
    }

    Row {
        id: leftStrip

        anchors.verticalCenter: parent.verticalCenter
        anchors.left: parent.left
        anchors.leftMargin: Theme.spacingMedium
        anchors.right: rightStrip.left
        anchors.rightMargin: Theme.spacingMedium
        spacing: Theme.spacingMedium
        clip: true

        Repeater {
            model: bar.codec.ordered(["path", "remote", "activity"], bar.leftOrder)

            delegate: StatusBarSlot {
                parts: statusParts.byKey
                reorder: leftReorder
                bar: bar
                strip: leftStrip
            }
        }
    }

    Row {
        id: rightStrip

        anchors.verticalCenter: parent.verticalCenter
        anchors.right: parent.right
        anchors.rightMargin: Theme.spacingMedium
        spacing: Theme.spacingMedium

        Repeater {
            model: bar.codec.ordered(["cursor", "lsp", "ide", "core"], bar.rightOrder)

            delegate: StatusBarSlot {
                parts: statusParts.byKey
                reorder: rightReorder
                bar: bar
                strip: rightStrip
            }
        }
    }

    ReorderController {
        id: leftReorder

        container: leftStrip
        onMoved: (key, dropIndex, visibleKeys) => bar.itemMoved("statusLeft", key, dropIndex, visibleKeys)
    }

    ReorderController {
        id: rightReorder

        container: rightStrip
        onMoved: (key, dropIndex, visibleKeys) => bar.itemMoved("statusRight", key, dropIndex, visibleKeys)
    }
}
