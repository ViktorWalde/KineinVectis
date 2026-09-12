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

    // O contexto de compilador do arquivo ATIVO (a segunda metade do P0):
    // `contextPath` e' o que a UI perguntou; `fileContext` e' a ultima
    // resposta PARA ESSE caminho — resposta de arquivo anterior e' descartada.
    property string contextPath: ""
    property var fileContext: ({})

    readonly property string indexState: stats.state !== undefined ? stats.state : "idle"
    readonly property bool ready: indexState === "ready"
    readonly property bool building: indexState === "building"

    signal statusRequested()
    signal contextRequested(string path)

    visible: false

    onWorkspaceRootChanged: {
        // O indice e' do projeto: trocar de workspace zera a tela ate' o core
        // mandar o novo (ele comeca a construir no proprio workspace.open).
        stats = ({});
        progressFiles = 0;
        progressSymbols = 0;
        contextPath = "";
        fileContext = ({});
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
        // O contexto pode ter mudado com o indice (configure novo, Cargo.toml
        // salvo): a resposta do arquivo ativo e' pedida de novo.
        if (contextPath !== "" && ready) {
            contextRequested(contextPath);
        }
    }

    // O arquivo ativo mudou (aba trocada, aberta ou fechada). Vazio limpa.
    function setActivePath(path) {
        const novo = path === undefined || path === null ? "" : path;
        if (novo === contextPath) return;
        contextPath = novo;
        fileContext = ({});
        if (novo !== "" && workspaceRoot !== "") {
            contextRequested(novo);
        }
    }

    function handleContext(context) {
        if (context === undefined || context === null || context.path === undefined) return;
        // Resposta atrasada de outro arquivo: fica a do arquivo ativo.
        if (context.path !== contextPath) return;
        fileContext = context;
    }

    // O resumo curto para a barra: o que o arquivo E' para o compilador.
    function contextSummary() {
        const c = fileContext;
        if (contextPath === "" || c.path === undefined) return "";
        if (c.unit !== undefined) {
            const partes = [baseName(c.unit.compiler)];
            if (c.unit.standard !== undefined) partes.push(c.unit.standard);
            partes.push(qsTr("%1 -I").arg(c.unit.includes.length));
            partes.push(qsTr("%1 -D").arg(c.unit.defines.length));
            return partes.join(" · ") + (c.hint !== undefined ? " ⚠" : "");
        }
        if (c.crate !== undefined) {
            return qsTr("cargo · %1 (%2, %3)").arg(c.crate.package).arg(c.crate.kind).arg(c.crate.edition);
        }
        if (c.python !== undefined) {
            const partes = ["python", c.python.origin];
            if (c.python.version !== undefined) partes.push(c.python.version.replace(/^Python /, ""));
            return partes.join(" · ") + (c.python.warning !== undefined ? " ⚠" : "");
        }
        if (c.hint !== undefined) return qsTr("sem contexto ⚠");
        return "";
    }

    // O detalhe ao pairar: a origem e a dica, ou os caminhos que importam.
    function contextDetail() {
        const c = fileContext;
        if (c.path === undefined) return "";
        const partes = [];
        if (c.unit !== undefined) partes.push(qsTr("diretório %1").arg(c.unit.directory));
        if (c.crate !== undefined) partes.push(qsTr("alvo %1 · %2").arg(c.crate.target).arg(c.crate.manifest));
        if (c.python !== undefined) partes.push(c.python.interpreter);
        if (c.source !== undefined) partes.push(c.source);
        if (c.hint !== undefined) partes.push(c.hint);
        if (c.python !== undefined && c.python.warning !== undefined) partes.push(c.python.warning);
        return partes.join(" — ");
    }

    function baseName(caminho) {
        if (caminho === undefined || caminho === null) return "";
        const i = String(caminho).lastIndexOf("/");
        return i < 0 ? String(caminho) : String(caminho).substring(i + 1);
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
