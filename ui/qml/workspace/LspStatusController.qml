import QtQuick

// O ESTADO DOS SERVIDORES DE LINGUAGEM para a barra de status (Etapa 2, F2).
// Ate' 2026-09-18 o `event.lsp.status` ia so' para a aba IDE: um clangd que
// morreu, um basedpyright que nao subiu, ninguem via sem abrir o log — o
// pente-fino nao pegou porque o C++ TRATAVA o evento (escrevia no log).
// Aqui: um mapa linguagem -> { status, message }, e o resumo que a barra
// mostra; trocar de workspace esquece.
Item {
    id: root

    property string workspaceRoot: ""
    // { language: { status: "running|starting|failed|exited|stopped|restarting", message } }
    property var servers: ({})
    // L6 (0.4.0d): a frente de trabalho em curso de cada servidor (indexando…),
    // { language: { title, message, percentage } }; percentage -1 = nao disse.
    property var progress: ({})

    visible: false

    onWorkspaceRootChanged: {
        servers = ({});
        progress = ({});
    }

    function handleStatus(language, status, message) {
        const novo = Object.assign({}, servers);
        if (status === "stopped") {
            delete novo[language];
        } else {
            novo[language] = { status: status, message: message || "" };
        }
        servers = novo;
        // Um servidor que parou ou caiu nao termina a frente: ela some com ele.
        if (status !== "running" && progress[language] !== undefined) {
            const withoutFront = Object.assign({}, progress);
            delete withoutFront[language];
            progress = withoutFront;
        }
    }

    function handleProgress(language, title, message, percentage, done) {
        const next = Object.assign({}, progress);
        if (done) {
            delete next[language];
        } else {
            next[language] = { title: title || "", message: message || "",
                               percentage: percentage === undefined ? -1 : percentage };
        }
        progress = next;
    }

    // "rust · Indexing 43%": a frente ativa, curta.
    function progressLabel(language) {
        const front = progress[language];
        if (front === undefined) return "";
        return language + " · " + front.title + (front.percentage >= 0 ? " " + front.percentage + "%" : "");
    }

    function languages() {
        return Object.keys(servers).sort();
    }

    function withStatus(list) {
        return languages().filter(function(l) { return list.indexOf(servers[l].status) >= 0; });
    }

    function runningCount() {
        return withStatus(["running"]).length;
    }

    // Caiu ou nao subiu: o que a barra pinta de vermelho.
    function failed() {
        return withStatus(["failed", "exited"]);
    }

    // "LSP ● 2" · "LSP ✗ python" · "LSP … cpp" — curto; o detalhe vai no tooltip.
    function summary() {
        const total = languages().length;
        if (total === 0) {
            return "";
        }
        const ruins = failed();
        if (ruins.length > 0) {
            return qsTr("LSP ✗ %1").arg(ruins.join(", "));
        }
        const working = Object.keys(progress).sort();
        if (working.length > 0) {
            return qsTr("LSP … %1").arg(working.map(progressLabel).join(", "));
        }
        const rodando = runningCount();
        if (rodando < total) {
            return qsTr("LSP … %1").arg(withStatus(["starting", "restarting"]).join(", "));
        }
        return qsTr("LSP ● %1").arg(rodando);
    }

    function detail() {
        return languages().map(function(l) {
            const s = servers[l];
            const front = progress[l];
            const work = front === undefined ? ""
                    : " — " + front.title + (front.message !== "" ? " (" + front.message + ")" : "");
            return l + ": " + s.status + (s.message !== "" ? " — " + s.message : "") + work;
        }).join("\n");
    }

    function hasFailure() {
        return failed().length > 0;
    }
}
