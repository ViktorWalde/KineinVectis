pragma ComponentBehavior: Bound
import QtQuick

// Estado do indice do projeto INTEIRO (pilar 0 do roadmaps/42, decisao do
// autor em 2026-09-12: a IDE le todas as pastas, arquivos e declaracoes).
//
// Guarda os totais que o core manda (event.index.finished / index.status) e o
// progresso enquanto constroi. NAO le arquivo nenhum: quem caminha a arvore e
// parseia e' o core, em job. A busca por nome (`#nome`) mora no
// SearchEverywhereController, que pede ao core e recebe do roteador.
Item {
    id: root

    property string workspaceRoot: ""
    property var stats: ({})
    property int progressFiles: 0
    property int progressSymbols: 0

    readonly property string indexState: stats.state !== undefined ? stats.state : "idle"
    readonly property bool ready: indexState === "ready"
    readonly property bool building: indexState === "building"

    signal statusRequested()

    visible: false

    onWorkspaceRootChanged: {
        // O indice e' do projeto: trocar de workspace zera a tela ate' o core
        // mandar o novo (ele comeca a construir no proprio workspace.open).
        stats = ({});
        progressFiles = 0;
        progressSymbols = 0;
        if (workspaceRoot !== "") {
            stats = ({ state: "building" });
            statusRequested();
        }
    }

    function handleStatus(newStats) {
        stats = newStats === undefined || newStats === null ? ({}) : newStats;
    }

    function handleProgress(files, symbols) {
        progressFiles = files;
        progressSymbols = symbols;
        if (!building) {
            stats = ({ state: "building" });
        }
    }

    function handleFinished(newStats) {
        handleStatus(newStats);
    }

    // "1.010 arquivos · 70 mil linhas · 4.658 símbolos" — ou o progresso.
    // Campo ausente nao vira "undefined".
    function summary() {
        if (building) {
            return progressFiles > 0
                    ? qsTr("indexando… %1 arquivos, %2 símbolos").arg(progressFiles).arg(progressSymbols)
                    : qsTr("indexando…");
        }
        if (!ready) {
            return indexState === "failed"
                    ? qsTr("índice falhou: %1").arg(stats.error !== undefined ? stats.error : "")
                    : "";
        }
        return qsTr("%1 arquivos · %2 linhas · %3 símbolos")
                .arg(formatCount(stats.files))
                .arg(formatCount(stats.lines))
                .arg(formatCount(stats.symbols));
    }

    // 70030 -> "70 mil"; 4658 -> "4.658"; 1010 -> "1.010". Sem depender da
    // locale do processo: o separador de milhar e' o ponto, sempre.
    function formatCount(n) {
        const v = Number(n);
        if (n === undefined || isNaN(v)) return "0";
        if (v >= 100000) return Math.round(v / 1000) + " mil";
        const texto = String(Math.round(v));
        let saida = "";
        for (let i = 0; i < texto.length; i++) {
            const resto = texto.length - i;
            saida += texto.charAt(i);
            if (resto > 1 && (resto - 1) % 3 === 0) saida += ".";
        }
        return saida;
    }
}
