pragma ComponentBehavior: Bound
import QtQuick

// Estado da toolchain do workspace (roadmap 30, etapa 5).
//
// Guarda o que o core respondeu e o que o usuario abriu no menu. NAO decide
// nada: quem sabe o que existe na maquina, o que e valido e o que vira
// argumento de `cmake` e o core. Aqui so mora estado de UI.
//
// Nao fala com o CoreClient direto: pede por sinal e recebe do roteador.
Item {
    id: root

    property string workspaceRoot: ""
    property var selections: []
    property var candidates: []
    property bool menuVisible: false
    property real menuX: 0
    property real menuY: 0
    property string errorText: ""

    signal getRequested()
    signal setRequested(string role, string id)

    visible: false

    onWorkspaceRootChanged: {
        selections = [];
        candidates = [];
        errorText = "";
        menuVisible = false;
        if (workspaceRoot !== "") {
            getRequested();
        }
    }

    function handleResolved(newSelections, newCandidates) {
        selections = newSelections;
        candidates = newCandidates;
        errorText = "";
    }

    function handleFailed(method, message) {
        if (method !== "toolchain.set" && method !== "toolchain.get") {
            return;
        }
        errorText = message;
    }

    function selectionFor(role) {
        for (let index = 0; index < selections.length; ++index) {
            if (selections[index].role === role) {
                return selections[index];
            }
        }
        return null;
    }

    function candidatesFor(role) {
        const found = [];
        for (let index = 0; index < candidates.length; ++index) {
            if (candidates[index].role === role) {
                found.push(candidates[index]);
            }
        }
        return found;
    }

    function labelFor(role) {
        const selection = selectionFor(role);
        if (selection === null || selection.id === undefined) {
            return qsTr("automático");
        }
        const options = candidatesFor(role);
        for (let index = 0; index < options.length; ++index) {
            if (options[index].id === selection.id) {
                return options[index].label;
            }
        }
        // Escolhido mas ausente: dizer o nome cru e melhor do que fingir que
        // esta tudo bem — o core tambem para de fixar o caminho nesse caso.
        return selection.id + qsTr(" (ausente)");
    }

    // Resumo curto para a barra de status: so o que o usuario FIXOU.
    function summary() {
        const partes = [];
        const papeis = ["cxxCompiler", "cCompiler", "generator"];
        for (let index = 0; index < papeis.length; ++index) {
            const selection = selectionFor(papeis[index]);
            if (selection !== null && selection.id !== undefined) {
                partes.push(labelFor(papeis[index]));
            }
        }
        return partes.length === 0 ? qsTr("automática") : partes.join(" · ");
    }

    function openMenu(x, y) {
        menuX = x;
        menuY = y;
        menuVisible = true;
        getRequested();
    }

    function closeMenu() {
        menuVisible = false;
    }

    function choose(role, id) {
        setRequested(role, id);
    }
}
