pragma ComponentBehavior: Bound
import QtQuick

// Estado das Configuration Actions (roadmap 30, etapa 2).
//
// Guarda a lista contextual, a acao selecionada, os parametros digitados e o
// preview devolvido pelo core. NAO decide nada de configuracao: quem sabe se
// uma acao esta disponivel, o que ela escreve e se o disco mudou e o core —
// aqui so mora estado de UI (ARCHITECTURE.md §3.1 regra 3).
//
// Nao fala com o CoreClient direto: pede por sinal e recebe do roteador.
Item {
    id: root

    property string workspaceRoot: ""
    property bool dialogVisible: false

    // O que o core devolveu no ultimo `configAction.list`. O ListModel carrega
    // so os papeis PLANOS que a lista desenha; os objetos crus ficam no mapa
    // ao lado. Motivo medido: o ListModel do QML converte array/objeto aninhado
    // em ListModel proprio, e `params[0].name` deixa de existir — a validacao
    // de parametro obrigatorio passava a nao ver parametro nenhum.
    property var actionsModel: ListModel {}

    // AS BIBLIOTECAS ENTRAM NA MESMA LISTA (2026-09-04).
    //
    // Pedido do autor: *"em vez de 'Biblioteca C/C++', que só aparece
    // funcionalidade para C/C++, deveria aparecer o Configure Actions e o nome
    // mudar"*. Ele estava vendo duas telas que fazem a MESMA coisa — ativar
    // algo no projeto — com listas separadas, e foi por isso que o Cargo
    // parecia inalcancavel.
    //
    // A juncao nao inventa camada nova: uma biblioteca JA' E' um pacote de
    // acoes de configuracao (`findPackage` + `addTargetLinkLibraries`). O que
    // faltava era mostra-las no mesmo lugar.
    property var libraryController: null

    // As acoes cruas do ultimo `configAction.list`. Guardadas porque a lista
    // precisa ser RECONSTRUIDA quando as bibliotecas chegam: elas vem de outra
    // resposta, e sem isto o painel abriria com as acoes e sem as bibliotecas
    // ate' o proximo `configAction.list`.
    property var rawActions: []

    Connections {
        target: root.libraryController

        function onLibrariesChanged() {
            root.rebuildList();
        }
    }
    property var actionsById: ({})
    property var activeBuildSystems: []
    // Filtro de escopo escolhido pelo usuario: "" = todos (spec 9.2 §6.3).
    property string scopeFilter: ""
    property string searchQuery: ""

    // Acao selecionada e o que o usuario digitou nos campos dela.
    property string selectedId: ""
    property var selectedAction: null
    property var paramValues: ({})

    // Preview vigente. `previewId` diz de QUAL acao ele e: sem isso, uma
    // resposta atrasada de outra acao pintaria o diff errado na tela.
    property string previewId: ""
    property var previewFiles: []
    property var previewReport: []
    property var previewNotes: []
    property string previewSummary: ""
    property bool previewLoading: false
    property string errorText: ""
    property string statusText: ""

    signal listRequested(bool includeHiddenByScope)
    signal previewRequested(string id, var params)
    signal applyRequested(string id, var params, var expected)

    visible: false

    onWorkspaceRootChanged: {
        clear();
        if (workspaceRoot !== "") {
            listRequested(false);
        }
    }

    function clear() {
        actionsModel.clear();
        actionsById = ({});
        activeBuildSystems = [];
        scopeFilter = "";
        searchQuery = "";
        clearSelection();
        statusText = "";
    }

    function clearSelection() {
        selectedId = "";
        selectedAction = null;
        paramValues = {};
        clearPreview();
    }

    function clearPreview() {
        previewId = "";
        previewFiles = [];
        previewReport = [];
        previewNotes = [];
        previewSummary = "";
        previewLoading = false;
        errorText = "";
    }

    // Abrir o painel pede as DUAS listas: acoes e bibliotecas. Sem isto o
    // painel abriria so' com o que ja' estivesse em memoria.
    function openDialog() {
        if (libraryController !== null) {
            libraryController.listRequested();
        }
        dialogVisible = true;
        if (workspaceRoot !== "") {
            listRequested(false);
        }
    }

    function closeDialog() {
        dialogVisible = false;
    }

    // Abre o dialogo JA na acao pedida, com os parametros preenchidos.
    //
    // E por aqui que o catalogo de bibliotecas entrega o trabalho: o plano dele
    // decide QUAL acao e com quais parametros, e a escrita continua acontecendo
    // aqui — com o preview e o consentimento que este dominio ja tem. O
    // `library` nunca escreve arquivo, e este metodo e a fronteira.
    //
    // A selecao so' acontece depois da lista chegar: `select` procura no
    // catalogo, e pedir antes acharia vazio. Por isso o pedido fica pendente.
    function openWith(actionId, params) {
        dialogVisible = true;
        pendingSelection = actionId;
        pendingParams = params === undefined ? ({}) : params;
        if (workspaceRoot !== "") {
            listRequested(false);
        }
    }

    function refresh() {
        listRequested(false);
    }

    // Preenche a lista com o que o core mandou. A ordem e a do catalogo: a UI
    // nao reordena, so filtra.
    // Acao pedida por outro dominio antes de a lista existir.
    property string pendingSelection: ""
    property var pendingParams: ({})

    // Acrescenta as bibliotecas ao fim da lista, com a MESMA lingua de estado
    // das acoes: `alreadyApplied` quando ja' estao no projeto.
    function appendLibraries() {
        if (libraryController === null) {
            return;
        }
        const libs = libraryController.libraries;
        for (let index = 0; index < libs.length; ++index) {
            const lib = libs[index];
            actionsModel.append({
                actionId: "library:" + lib.id,
                title: lib.name,
                description: lib.summary,
                scope: "cmake",
                category: qsTr("Bibliotecas"),
                risk: "medium",
                riskLabel: "",
                riskExplanation: "",
                effect: "edit",
                actionState: lib.applied === true ? "alreadyApplied" : "available",
                reason: lib.license + " · " + lib.pinnedVersion,
                affects: "CMakeLists.txt"
            });
        }
    }

    // `true` quando o id selecionado e' de uma biblioteca, nao de uma acao.
    function isLibrary(id) {
        return typeof id === "string" && id.indexOf("library:") === 0;
    }

    function libraryIdOf(id) {
        return id.substring("library:".length);
    }

    // Reconstroi a lista a partir do que ja' foi recebido dos dois lados.
    function rebuildList() {
        const byId = {};
        actionsModel.clear();
        for (let index = 0; index < rawActions.length; ++index) {
            const action = rawActions[index];
            byId[action.id] = action;
            actionsModel.append({
                actionId: action.id,
                title: action.title,
                description: action.description,
                scope: action.scope,
                category: action.category,
                risk: action.risk,
                riskLabel: action.riskLabel,
                riskExplanation: action.riskExplanation,
                effect: action.effect,
                // `state` e propriedade do QQuickItem: o papel do modelo tem que
                // ter outro nome, senao o delegate sombreia o estado visual.
                actionState: action.state,
                reason: action.reason !== undefined ? action.reason : "",
                affects: action.affects.join(", ")
            });
        }
        appendLibraries();
        actionsById = byId;
    }

    function handleListed(actions, buildSystems) {
        const previous = selectedId;
        rawActions = actions;
        rebuildList();
        activeBuildSystems = buildSystems;
        statusText = "";
        // Pedido vindo de outro dominio tem precedencia sobre a selecao
        // anterior: quem chamou `openWith` acabou de dizer o que quer ver.
        if (pendingSelection !== "") {
            const pedida = pendingSelection;
            const parametros = pendingParams;
            pendingSelection = "";
            pendingParams = ({});
            select(pedida);
            if (selectedId === pedida) {
                for (const nome in parametros) {
                    setParam(nome, parametros[nome]);
                }
                if (!missingRequiredParam()) {
                    previewRequested(selectedId, paramValues);
                }
            }
        } else if (previous !== "") {
            select(previous);
        }
    }

    function actionAt(id) {
        const entry = actionsById[id];
        return entry !== undefined ? entry : null;
    }

    function select(id) {
        // Linha de BIBLIOTECA: quem sabe montar o plano dela e' o dominio
        // `library`, e ele ja' faz isso. O painel unificado escolhe qual das
        // duas visoes mostrar; nao reimplementa nenhuma das duas.
        if (isLibrary(id)) {
            selectedId = id;
            selectedAction = null;
            clearPreview();
            libraryController.select(libraryIdOf(id));
            return;
        }
        const entry = actionAt(id);
        if (entry === null) {
            clearSelection();
            return;
        }
        const changed = id !== selectedId;
        selectedId = id;
        selectedAction = entry;
        if (changed) {
            paramValues = {};
            clearPreview();
        }
        requestPreview();
    }

    // O valor atual de um campo. Existe para o campo poder ser CONTROLADO —
    // clicar num chip de sugestao precisa aparecer no texto, e sem leitura o
    // TextInput seria a unica fonte da verdade.
    function paramValue(name) {
        return paramValues[name] !== undefined ? paramValues[name] : "";
    }

    function setParam(name, value) {
        const values = {};
        for (const key in paramValues) {
            values[key] = paramValues[key];
        }
        values[name] = value;
        paramValues = values;
    }

    // Falta algum parametro obrigatorio? Enquanto faltar, nem se pede preview:
    // o core recusaria, e um erro previsivel nao e diagnostico, e ruido.
    function missingRequiredParam() {
        if (selectedAction === null) {
            return "";
        }
        const declared = selectedAction.params;
        for (let index = 0; index < declared.length; ++index) {
            const param = declared[index];
            const value = paramValues[param.name];
            if (param.required && (value === undefined || value.trim() === "")) {
                return param.label;
            }
        }
        return "";
    }

    function canPreview() {
        return selectedAction !== null && selectedAction.state !== "unavailable"
                && selectedAction.state !== "hiddenByScope" && missingRequiredParam() === "";
    }

    function requestPreview() {
        if (!canPreview()) {
            clearPreview();
            return;
        }
        previewLoading = true;
        errorText = "";
        previewRequested(selectedId, paramValues);
    }

    function handlePreviewed(preview) {
        previewLoading = false;
        if (preview.id !== selectedId) {
            return;
        }
        previewId = preview.id;
        previewSummary = preview.summary;
        previewFiles = preview.files !== undefined ? preview.files : [];
        previewReport = preview.report !== undefined ? preview.report : [];
        previewNotes = preview.notes !== undefined ? preview.notes : [];
        errorText = "";
    }

    // O Apply devolve ao core o `before` que o preview mostrou: e o mesmo
    // compare-before-save do editor, aplicado ao CONSENTIMENTO (ARCHITECTURE
    // §7.1). Sem isso, um Apply dado dez minutos depois sobrescreveria em
    // silencio uma edicao feita nesse intervalo.
    function apply() {
        if (previewId !== selectedId || previewId === "") {
            return;
        }
        const expected = [];
        for (let index = 0; index < previewFiles.length; ++index) {
            const file = previewFiles[index];
            const entry = { path: file.path };
            if (file.before !== undefined) {
                entry.content = file.before;
            }
            expected.push(entry);
        }
        applyRequested(selectedId, paramValues, expected);
    }

    function handleApplied(id, message, files, jobId) {
        statusText = jobId !== "" ? qsTr("%1 (job %2)").arg(message).arg(jobId) : message;
        errorText = "";
        paramValues = {};
        clearPreview();
        // O efeito mudou o disco: o estado das acoes mudou junto.
        listRequested(false);
    }

    function handleFailed(method, message) {
        if (method !== "configAction.preview" && method !== "configAction.apply") {
            return;
        }
        previewLoading = false;
        errorText = message;
    }
}
