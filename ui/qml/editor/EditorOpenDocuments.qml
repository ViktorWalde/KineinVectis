import QtQuick

// OS DOCUMENTOS ABERTOS, IDENTIFICADOS POR ID (fatia V5, 2026-09-25).
//
// O aceite da V5 no roadmap 47: "nenhuma operacao de dominio depende da posicao
// visual da aba". Ate' aqui `selectTab(index)` e `closeTab(index)` eram a API —
// e posicao muda sozinha: fechar uma aba desloca todas as seguintes, e
// `closeTabsUnderPath` chegava a fechar varias num laco.
//
// O ID e' SINTETICO, e nao o caminho, porque o caminho NAO e' estavel: renomear
// um arquivo (ou uma pasta acima dele) reescreve o `path` da aba pelo
// `applyPathRenameToTabs`. O documento continua o mesmo; o nome dele e' que
// mudou.
Item {
    id: root

    property alias model: documents
    property int revision: 0

    // O INDICE DE CADA DOCUMENTO, FORA DO MODELO (2026-09-25).
    //
    // Este mapa e a regra de "o indice nunca sai de um binding" nasceram do
    // mesmo laco, que o gate pegou: "Binding loop detected for property
    // breakpointLines", no `ShellEditorHost`.
    //
    // A cadeia era esta: os breakpoints da calha dependem do caminho do arquivo
    // atual, que sai de `openFilesModel.get(currentTab)`. Com `currentTab`
    // sendo um binding, a leitura do `ListModel` entrava no grafo de
    // dependencias de QUEM chamou — e ler uma linha do modelo a materializa, o
    // que notifica o modelo e reavalia o binding, em laco. Medido nos dois
    // sentidos: com o binding, o laco aparece; sem ele, some.
    //
    // Dai as duas metades da solucao. Aqui, o mapa e' reconstruido nas
    // MUTACOES — onde ler o modelo e' seguro, porque nao ha' binding em
    // avaliacao. La', o `currentTab` do controller e' recalculado por SINAL, e
    // nao por expressao. Nenhum binding atravessa o `ListModel`.
    property var indexByDocId: ({})

    // Comeca em 1: zero e' "nenhum documento", e um id nunca e' reaproveitado
    // nesta sessao — fechar e reabrir o mesmo arquivo da' um documento novo.
    property int nextDocId: 1

    visible: false

    ListModel {
        id: documents
    }

    function reindex() {
        const map = ({});
        for (let i = 0; i < documents.count; i++) {
            map[documents.get(i).docId] = i;
        }
        root.indexByDocId = map;
        root.revision += 1;
    }

    function add(fields) {
        const docId = root.nextDocId;
        root.nextDocId += 1;
        const row = ({ "docId": docId });
        for (const key in fields) {
            row[key] = fields[key];
        }
        documents.append(row);
        root.reindex();
        return docId;
    }

    function remove(docId) {
        const index = root.indexOf(docId);
        if (index < 0) {
            return false;
        }
        documents.remove(index);
        root.reindex();
        return true;
    }

    function clear() {
        documents.clear();
        root.reindex();
    }

    // `map` entra como argumento para o binding de quem chama reavaliar quando
    // a lista muda — e, de proposito, para que o binding NAO leia o
    // `ListModel`. Quem chama passa `openDocuments.indexByDocId`.
    function indexOf(docId, map) {
        if (docId <= 0) {
            return -1;
        }
        const index = (map !== undefined ? map : root.indexByDocId)[docId];
        return index === undefined ? -1 : index;
    }

    function docIdAt(index) {
        if (index < 0 || index >= documents.count) {
            return 0;
        }
        return documents.get(index).docId;
    }

    function docIdForPath(path) {
        for (let i = 0; i < documents.count; i++) {
            if (documents.get(i).path === path) {
                return documents.get(i).docId;
            }
        }
        return 0;
    }

    // Quem fica no lugar de quem sai: a aba seguinte, ou a anterior se a que
    // sai for a ultima. Zero quando nao sobra ninguem.
    function successorOf(docId) {
        const index = root.indexOf(docId);
        if (index < 0 || documents.count <= 1) {
            return 0;
        }
        return root.docIdAt(index + 1 < documents.count ? index + 1 : index - 1);
    }

    function docIdsUnder(path, pathRules) {
        const ids = [];
        for (let i = 0; i < documents.count; i++) {
            if (pathRules.isUnder(documents.get(i).path, path)) {
                ids.push(documents.get(i).docId);
            }
        }
        return ids;
    }
}
