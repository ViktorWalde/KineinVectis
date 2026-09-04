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
    // Os alvos do CMake que o core conhece, e DE ONDE ele os tirou.
    //
    // POR QUE ISTO EXISTE (2026-09-04). Relato de uso do autor: "o SQLite eu
    // nao consegui ativar". O painel pedia que ele DIGITASSE o nome do alvo, e
    // sem alvo nao havia plano — a IDE cobrava dele uma informacao que esta'
    // escrita no CMakeLists do proprio projeto.
    property var targets: []
    property string targetsOrigin: ""
    property string selectedId: ""
    property string target: ""
    property var plan: null
    property bool panelVisible: false
    property string errorText: ""

    signal listRequested()
    signal targetsRequested()
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
        targets = [];
        targetsOrigin = "";
        target = "";
        if (workspaceRoot !== "") {
            listRequested();
        }
    }

    function open() {
        panelVisible = true;
        if (libraries.length === 0) {
            listRequested();
        }
        // Sempre: o autor pode ter criado um alvo desde a ultima abertura.
        targetsRequested();
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

    // Um alvo so' NAO e' uma escolha: preencher e' o certo. Com varios, quem
    // escolhe e' o autor — a IDE nao adivinha em qual binario a biblioteca
    // entra.
    function handleTargets(newTargets, origin) {
        targets = newTargets;
        targetsOrigin = origin;
        if (target === "" && newTargets.length === 1) {
            setTarget(newTargets[0].name);
        }
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
