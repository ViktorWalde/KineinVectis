import QtQuick

// A COBERTURA dos testes na tela (D8 do roadmaps/41, 2026-09-17): o resumo
// do ultimo relatorio (arquivo, linhas encontradas/cobertas) e, para o
// arquivo ATIVO do editor, o mapa linha -> "covered"|"missed" que a calha
// pinta — a mesma forma do diff do git (diffLineKinds/diffRevision).
//
// Quem dispara o job e' o JobsController (startCoverage, como a analise);
// este controller so' guarda o desfecho e pede as linhas do arquivo ativo
// pelo roteador. Nao fala com o CoreClient direto.
Item {
    id: root

    property string workspaceRoot: ""
    // O ultimo desfecho: files [{ file, linesFound, linesHit }], tool, path.
    property var files: []
    property string tool: ""
    property string lastOutcome: ""
    property bool hasReport: false
    // O arquivo ativo e o que o relatorio diz dele.
    property string activePath: ""
    property var lineKinds: ({})
    property int revision: 0
    property bool activeKnown: false

    signal linesRequested(string file)

    visible: false

    onWorkspaceRootChanged: clear()

    function clear() {
        files = [];
        tool = "";
        lastOutcome = "";
        hasReport = false;
        activePath = "";
        lineKinds = {};
        revision++;
        activeKnown = false;
    }

    // Chamado na troca de aba do editor: sem relatorio nao pede nada.
    function setActivePath(path) {
        activePath = path === undefined || path === null ? "" : path;
        lineKinds = {};
        activeKnown = false;
        revision++;
        if (hasReport && activePath !== "") {
            linesRequested(activePath);
        }
    }

    function handleFinished(outcome) {
        if (outcome === undefined || outcome === null) return;
        hasReport = outcome.success === true;
        files = outcome.files !== undefined && outcome.files !== null ? outcome.files : [];
        tool = outcome.tool !== undefined ? outcome.tool : "";
        lastOutcome = hasReport
                ? qsTr("cobertura (%1): %2 arquivo(s), %3/%4 linhas").arg(tool).arg(files.length).arg(totalHit()).arg(totalFound())
                : qsTr("falhou: %1").arg(outcome.error !== undefined ? outcome.error : "");
        // O relatorio novo vale para o arquivo aberto agora.
        setActivePath(activePath);
    }

    // As linhas do arquivo pedido; um desfecho de outro arquivo nao entra.
    function handleLines(file, known, covered, missed) {
        if (file !== activePath) return;
        const kinds = {};
        for (let i = 0; i < covered.length; i++) kinds[covered[i]] = "covered";
        for (let j = 0; j < missed.length; j++) kinds[missed[j]] = "missed";
        activeKnown = known === true;
        lineKinds = kinds;
        revision++;
    }

    function totalFound() {
        let n = 0;
        for (let i = 0; i < files.length; i++) n += Number(files[i].linesFound);
        return n;
    }

    function totalHit() {
        let n = 0;
        for (let i = 0; i < files.length; i++) n += Number(files[i].linesHit);
        return n;
    }

    // "%" do arquivo ativo, para a barra; vazio sem relatorio/arquivo.
    function activeSummary() {
        if (!hasReport || !activeKnown) return "";
        let hit = 0, found = 0;
        for (const line in lineKinds) {
            found++;
            if (lineKinds[line] === "covered") hit++;
        }
        return found === 0 ? "" : qsTr("cobertura %1%").arg(Math.round(100 * hit / found));
    }
}
