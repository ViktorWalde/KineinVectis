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

    visible: false

    onWorkspaceRootChanged: servers = ({})

    function handleStatus(language, status, message) {
        const novo = Object.assign({}, servers);
        if (status === "stopped") {
            delete novo[language];
        } else {
            novo[language] = { status: status, message: message || "" };
        }
        servers = novo;
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
        const rodando = runningCount();
        if (rodando < total) {
            return qsTr("LSP … %1").arg(withStatus(["starting", "restarting"]).join(", "));
        }
        return qsTr("LSP ● %1").arg(rodando);
    }

    function detail() {
        return languages().map(function(l) {
            const s = servers[l];
            return l + ": " + s.status + (s.message !== "" ? " — " + s.message : "");
        }).join("\n");
    }

    function hasFailure() {
        return failed().length > 0;
    }
}
