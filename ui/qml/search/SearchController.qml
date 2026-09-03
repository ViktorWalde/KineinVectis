import QtQuick

// BUSCA E SUBSTITUIÇÃO NO PROJETO — o painel de baixo.
//
// A caixa modal que acha arquivo/símbolo/comando é outra coisa e tem dono
// próprio (`SearchEverywhereController`, separado em 2026-09-02): esta aqui é
// persistente, tem resultado navegável e tem uma **operação destrutiva** atrás
// dela. As duas dividiam um arquivo só e nada além do nome.
//
// # A sintaxe `\n`, e por que ela mora AQUI e não no core
//
// Desde 2026-09-02 (etapa 9 do `roadmaps/30`) o core acha e substitui texto que
// atravessa linhas. Só que o campo do painel é um `TextInput`, que é de UMA
// linha por construção: o usuário não consegue nem digitar nem colar um `\n`.
// A capacidade existiria sem ninguém alcançá-la, que é o anti-padrão do
// "mecanismo sem usuário" (`ARCHITECTURE.md` §8).
//
// Então a caixa aceita a sequência de dois caracteres `\n` e a traduz em quebra
// de linha antes de pedir ao core.
//
// **Por que a tradução não pode descer para o core:** o `query` do protocolo é
// texto LITERAL. Se o core interpretasse escapes, procurar por um `\n` de
// verdade — o que qualquer pessoa mexendo em código C ou Rust faz o tempo todo —
// passaria a ser impossível. Aqui a decisão é reversível e visível: quem
// precisar do literal escreve `\\n`, e o painel devolve a barra.
Item {
    id: root

    property string workspaceRoot: ""
    property alias searchModel: searchItemsModel
    property bool caseSensitive: false
    property bool searching: false
    property bool replaceMode: false
    property bool replacing: false
    property string replaceError: ""
    property string replaceSummary: ""
    property bool searchTruncated: false

    signal showTabRequested(string tab)
    signal focusSearchInputRequested()
    signal searchInFilesRequested(string query, bool caseSensitive)
    signal replaceInFilesRequested(string query, string replacement, bool caseSensitive)
    signal focusReplaceInputRequested()

    visible: false

    ListModel {
        id: searchItemsModel
    }

    function clear() {
        searchItemsModel.clear();
        caseSensitive = false;
        searching = false;
        replaceMode = false;
        replacing = false;
        replaceError = "";
        replaceSummary = "";
    }

    function openSearchPanel() {
        if (workspaceRoot === "") {
            return;
        }
        showTabRequested("search");
        focusSearchInputRequested();
    }

    function openReplacePanel() {
        if (workspaceRoot === "") {
            return;
        }
        replaceMode = true;
        replaceError = "";
        replaceSummary = "";
        showTabRequested("search");
        focusReplaceInputRequested();
    }

    // Traduz a sintaxe da CAIXA no texto literal que o core espera.
    //
    // `\n` vira quebra de linha; `\\n` vira o literal `\n` (barra + ene), para
    // quem procura a sequência de escape dentro do código. A varredura é feita
    // em uma passada, da esquerda para a direita, justamente para que `\\n` não
    // seja re-interpretado depois de a barra ser resolvida.
    function expandLineBreaks(text) {
        let saida = "";
        let indice = 0;
        while (indice < text.length) {
            if (text.charAt(indice) === "\\" && indice + 1 < text.length) {
                const seguinte = text.charAt(indice + 1);
                if (seguinte === "n") {
                    saida += "\n";
                    indice += 2;
                    continue;
                }
                if (seguinte === "\\") {
                    saida += "\\";
                    indice += 2;
                    continue;
                }
            }
            saida += text.charAt(indice);
            indice += 1;
        }
        return saida;
    }

    function runSearch(query) {
        if (query === "" || workspaceRoot === "" || searching) {
            return;
        }
        searching = true;
        searchTruncated = false;
        searchItemsModel.clear();
        searchInFilesRequested(expandLineBreaks(query), caseSensitive);
    }

    function runReplace(query, replacement) {
        if (query === "" || workspaceRoot === "" || replacing) {
            replaceError = query === "" ? qsTr("Informe o texto a substituir.") : "";
            return;
        }
        replacing = true;
        replaceError = "";
        replaceSummary = "";
        replaceInFilesRequested(expandLineBreaks(query),
                                expandLineBreaks(replacement), caseSensitive);
    }

    function rejectReplaceForDirtyEditors() {
        replacing = false;
        replaceError = qsTr("Salve todas as abas modificadas antes de substituir no projeto.");
    }

    function handleReplaceResult(files, replacements) {
        replacing = false;
        replaceError = "";
        // Os offsets do resultado anterior deixaram de representar o disco.
        // O usuario pode executar uma nova busca quando quiser conferir o
        // estado posterior, sem navegar por resultados obsoletos.
        searchItemsModel.clear();
        searchTruncated = false;
        replaceSummary = qsTr("%1 ocorrencia(s) em %2 arquivo(s).")
                .arg(replacements).arg(files.length);
    }

    function toggleCaseAndRun(query) {
        caseSensitive = !caseSensitive;
        runSearch(query);
    }

    function handleSearchResults(matches, truncated) {
        searching = false;
        searchItemsModel.clear();
        for (let i = 0; i < matches.length; i++) {
            const match = matches[i];
            searchItemsModel.append({
                path: match.path,
                line: match.line !== undefined ? Number(match.line) : 1,
                column: match.column !== undefined ? Number(match.column) : 1,
                preview: match.preview !== undefined ? match.preview : ""
            });
        }
        searchTruncated = truncated;
    }

    function relativeToRoot(path) {
        if (workspaceRoot !== "" && path.indexOf(workspaceRoot + "/") === 0) {
            return path.substring(workspaceRoot.length + 1);
        }
        return path;
    }

    function handleRequestFailed(method, message) {
        if (method === "fs.search") {
            searching = false;
        }
        if (method === "fs.replace") {
            replacing = false;
            replaceError = message;
        }
    }

}
