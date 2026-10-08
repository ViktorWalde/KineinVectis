import QtQuick

// O HISTORICO de consultas da conexao (passo 13b, roadmaps/59 §5.2.1). O core
// anota o que rodou e guarda fora do projeto; aqui ficam so' a lista que o menu
// mostra, o pedido de limpar e o texto de cada linha. Uma conexao por vez: a
// do console ativo, pedida de novo a cada abertura do menu.
QtObject {
    id: root

    property string workspaceRoot: ""
    property string name: ""
    property var entries: []
    property bool loading: false
    property string error: ""

    signal listRequested(string name)
    signal clearRequested(string name)

    onWorkspaceRootChanged: root.reset("")

    function reset(name) {
        root.name = name;
        root.entries = [];
        root.loading = false;
        root.error = "";
    }

    function open(name) {
        root.reset(name);
        if (name === "") return;
        root.loading = true;
        root.listRequested(name);
    }

    function handleListed(name, entries) {
        if (name !== root.name) return;
        root.entries = entries;
        root.loading = false;
    }

    function clear() {
        if (root.name !== "") root.clearRequested(root.name);
    }

    function handleCleared(name) {
        if (name === root.name) root.entries = [];
    }

    function handleFailed(message) {
        root.loading = false;
        root.error = message;
    }

    // A instrucao numa linha e curta: o menu nao quebra linha.
    function label(entry) {
        const text = String(entry.sql).replace(/\s+/g, " ").trim();
        return text.length > 72 ? text.slice(0, 71) + "…" : text;
    }

    // Quando rodou: so' a hora, se foi hoje; dia e hora, se antes.
    function when(entry, now) {
        const date = new Date(entry.at * 1000);
        return Qt.formatDateTime(date, date.toDateString() === now.toDateString() ? "HH:mm" : "dd/MM HH:mm");
    }
}
