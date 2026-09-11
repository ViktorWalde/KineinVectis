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

    // O KIT ativo (etapa 14): nome do preset, vazio = o padrao do workspace.
    // A escolha de toolchain deixou de ser do workspace e passou a ser do kit.
    property string preset: ""
    property string sysroot: ""
    property string targetTriple: ""
    // Chip do alvo embarcado: vai no `launch` do DAP (probe-rs). Existia no
    // protocolo e nao chegava a tela ate' 2026-09-11.
    property string chip: ""
    // `toolchainFile` que o PRESET declara. E informacao, nao escolha: quando
    // existe, ele tem precedencia sobre o que o usuario escolher aqui, e a tela
    // precisa dizer isso em vez de deixar procurar no lugar errado.
    property string presetToolchainFile: ""

    readonly property bool crossCompiling: targetTriple !== ""

    signal getRequested(string preset)
    signal setRequested(string role, string id, string preset)
    signal setKitRequested(string preset, string sysroot, string targetTriple, string chip)

    visible: false

    onWorkspaceRootChanged: {
        selections = [];
        candidates = [];
        errorText = "";
        menuVisible = false;
        preset = "";
        sysroot = "";
        targetTriple = "";
        chip = "";
        presetToolchainFile = "";
        if (workspaceRoot !== "") {
            getRequested("");
        }
    }

    function handleResolved(newSelections, newCandidates, newPreset, newSysroot,
                            newTargetTriple, newChip, newPresetToolchainFile) {
        selections = newSelections;
        candidates = newCandidates;
        preset = newPreset === undefined ? "" : newPreset;
        sysroot = newSysroot === undefined ? "" : newSysroot;
        targetTriple = newTargetTriple === undefined ? "" : newTargetTriple;
        chip = newChip === undefined ? "" : newChip;
        presetToolchainFile = newPresetToolchainFile === undefined ? "" : newPresetToolchainFile;
        errorText = "";
    }

    // Troca o kit ativo e recarrega — o que muda e o preset, nao o workspace.
    function selectKit(name) {
        preset = name === undefined ? "" : name;
        getRequested(preset);
    }

    // `undefined` PRESERVA o campo; string vazia LIMPA. O core trata igual, e
    // e por isso que mexer no sysroot nao apaga o alvo.
    function applyKit(newSysroot, newTargetTriple, newChip) {
        setKitRequested(preset, newSysroot, newTargetTriple, newChip);
    }

    function handleFailed(method, message) {
        if (method !== "toolchain.set" && method !== "toolchain.get"
                && method !== "toolchain.setKit") {
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

    // O rotulo de um id, ou o proprio id quando ele nao esta' entre os
    // candidatos desta maquina.
    function labelOf(role, id) {
        const options = candidatesFor(role);
        for (let index = 0; index < options.length; ++index) {
            if (options[index].id === id) {
                return options[index].label;
            }
        }
        // Escolhido mas ausente: dizer o nome cru e melhor do que fingir que
        // esta tudo bem — o core tambem para de fixar o caminho nesse caso.
        return id + qsTr(" (ausente)");
    }

    function labelFor(role) {
        const selection = selectionFor(role);
        if (selection === null) {
            return qsTr("nenhum detectado");
        }
        if (selection.id !== undefined) {
            return labelOf(role, selection.id);
        }
        // AUTOMATICO NAO E' MAIS "nao sei": desde 2026-09-04 o core escolhe o
        // primeiro candidato e diz qual. A palavra "automático" sozinha
        // escondia justamente a informacao que o autor precisa para discordar.
        if (selection.effectiveId !== undefined) {
            return labelOf(role, selection.effectiveId) + qsTr(" · automático");
        }
        return qsTr("nenhum detectado");
    }

    // `true` quando quem escolheu foi o core, e nao o autor.
    function isAutomatic(role) {
        const selection = selectionFor(role);
        return selection !== null && selection.automatic === true;
    }

    // Resumo curto para a barra de status.
    //
    // Antes mostrava SO' o que o autor tinha fixado, e num projeto novo dizia
    // apenas "automática" — verdadeiro e inutil. Agora diz o que vai ser
    // USADO, marcando quando a escolha nao foi dele.
    function summary() {
        const partes = [];
        const papeis = ["cxxCompiler", "generator"];
        let algumAutomatico = false;
        for (let index = 0; index < papeis.length; ++index) {
            const selection = selectionFor(papeis[index]);
            if (selection === null) {
                continue;
            }
            const id = selection.id !== undefined
                     ? selection.id : selection.effectiveId;
            if (id === undefined) {
                continue;
            }
            partes.push(labelOf(papeis[index], id));
            if (selection.automatic === true) {
                algumAutomatico = true;
            }
        }
        if (partes.length === 0) {
            return qsTr("nenhuma detectada");
        }
        return partes.join(" · ") + (algumAutomatico ? qsTr(" · automática") : "");
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
        setRequested(role, id, preset);
    }
}
