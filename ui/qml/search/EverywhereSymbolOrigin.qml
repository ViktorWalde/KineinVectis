import QtQuick

// A PERGUNTA DE SIMBOLO: qual e', e de quem e' a resposta (L1, 2026-09-26).
//
// `@nome` pergunta ao LSP sobre o DOCUMENTO aberto; `#nome` pergunta ao indice
// e, quando ha' arquivo aberto, tambem ao LSP sobre o WORKSPACE. Ate' aqui as
// duas respostas chegavam pelo mesmo sinal, sem identidade — e digitar `@nome`
// e depois `#nome` deixava a resposta do primeiro pintar a lista do segundo.
//
// O caminho de FALHA, no mesmo fluxo, sempre preservou o metodo. A assimetria e'
// que denunciava o defeito: era o SUCESSO que perdia a informacao que o
// fracasso mantinha.
//
// Desde o protocolo 0.135.0 a resposta carrega a identidade do pedido — `path`
// para documento, `query` para workspace. Aqui e' so' a comparacao, e ela e'
// pura: entra o que voltou, sai se vale ou nao.
QtObject {
    id: root

    // "document" | "workspace" | "" (nenhum pedido em voo).
    property string scope: ""
    // A ancora do pedido de documento, CARIMBADA por quem envia — o roteador,
    // que tem o caminho do editor. Ler de um binding aqui seria perguntar "qual
    // arquivo esta' ativo agora", e a pergunta certa e' "qual arquivo eu
    // perguntei".
    property string anchorPath: ""
    // O texto perguntado ao workspace.
    property string query: ""

    // O QUE O TEXTO PEDE, e o que impede de pedir. As tres recusas sao decisao
    // de produto — dizer "abra um workspace" e' melhor que devolver lista
    // vazia —, e por isso moram aqui, numa regra pura, e nao espalhadas no meio
    // do fluxo.
    //
    // Devolve { scope, needle, error }: `scope` vazio com `error` preenchido
    // significa "nao da' para perguntar, e o motivo e' este".
    function planFor(query, workspaceRoot, hasActiveEditorFile) {
        const isDocument = query.charAt(0) === "@";
        const needle = query.substring(1).trim();
        if (workspaceRoot === "") {
            return { "scope": "", "needle": needle,
                     "error": qsTr("Abra um workspace para buscar símbolos.") };
        }
        if (!isDocument && needle === "") {
            return { "scope": "", "needle": needle,
                     "error": qsTr("Digite o nome do símbolo após #.") };
        }
        if (isDocument && !hasActiveEditorFile) {
            return { "scope": "", "needle": needle,
                     "error": qsTr("Abra um arquivo com LSP para buscar símbolos do documento.") };
        }
        return { "scope": isDocument ? "document" : "workspace", "needle": needle, "error": "" };
    }

    // QUEM DECIDE O QUE FAZER COM O PLANO e' quem o produziu. Comparar a
    // string do escopo do lado de fora colocaria a MESMA derivacao em dois
    // arquivos — e duas copias da mesma regra divergem em silencio.
    //
    // Devolve `true` quando o pedido e' de documento.
    function ask(plan) {
        if (plan.scope === "document") {
            root.askDocument();
            return true;
        }
        root.askWorkspace(plan.needle);
        return false;
    }

    function askDocument() {
        root.scope = "document";
        root.anchorPath = "";
        root.query = "";
    }

    function askWorkspace(needle) {
        root.scope = "workspace";
        root.anchorPath = "";
        root.query = needle;
    }

    function noteAnchor(path) {
        root.anchorPath = path;
    }

    function forget() {
        root.scope = "";
        root.anchorPath = "";
        root.query = "";
    }

    // Sem carimbo nao ha' o que comparar: uma resposta que sobreviveu a um
    // pedido cancelado nao entra.
    function acceptsDocument(path) {
        return root.scope === "document" && root.anchorPath !== "" && path === root.anchorPath;
    }

    function acceptsWorkspace(query) {
        return root.scope === "workspace" && query === root.query;
    }
}
