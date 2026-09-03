pragma ComponentBehavior: Bound
import QtQuick

// Estado do catalogo de bibliotecas (etapa 20 do roadmaps/35).
//
// Guarda o que o core respondeu e o que o usuario escolheu. NAO decide nada:
// quem sabe o que existe, o que esta instalado e o que seria escrito e o core.
//
// Nao fala com o CoreClient direto: pede por sinal e recebe do roteador.
Item {
    id: root

    property string workspaceRoot: ""
    property var libraries: []
    property string selectedId: ""
    property string target: ""
    property var plan: null
    property bool panelVisible: false
    property string errorText: ""

    signal listRequested()
    signal planRequested(string id, string target)
    // A aplicacao NAO acontece aqui: o plano vira Configuration Action, e quem
    // aplica e o dominio configaction, com o preview e o consentimento dele.
    signal applyStepRequested(string actionId, var params)

    visible: false

    onWorkspaceRootChanged: {
        libraries = [];
        selectedId = "";
        plan = null;
        errorText = "";
        if (workspaceRoot !== "") {
            listRequested();
        }
    }

    function open() {
        panelVisible = true;
        if (libraries.length === 0) {
            listRequested();
        }
    }

    function close() {
        panelVisible = false;
    }

    function select(id) {
        selectedId = id;
        plan = null;
        if (target !== "") {
            planRequested(id, target);
        }
    }

    function setTarget(name) {
        target = name;
        if (selectedId !== "") {
            planRequested(selectedId, target);
        }
    }

    function libraryById(id) {
        return libraries.find(function(lib) { return lib.id === id; }) || null;
    }

    function handleList(newLibraries) {
        libraries = newLibraries;
        errorText = "";
    }

    function handlePlan(newPlan) {
        plan = newPlan;
        errorText = "";
    }

    function handleFailed(method, message) {
        if (method === "library.list" || method === "library.plan") {
            errorText = message;
        }
    }
}
