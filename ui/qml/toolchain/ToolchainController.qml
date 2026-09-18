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
    signal setKitRequested(string preset, string sysroot, string targetTriple, string chip,
                           string toolchainFile, string svdFile)
    signal inspectSysrootRequested(string path)
    signal importKitRequested(string path)
    // O seletor de pasta NATIVO da IDE (o FolderPicker da Start Screen) para
    // o SDK/sysroot (2026-09-17): quem o abre e' a composicao (AppDomains),
    // e o caminho escolhido volta por `handlePickedPath`.
    signal folderPickRequested(string purpose, string startPath)

    // O GERENCIADOR QUE LE O DISCO (42 §8 itens b e d, 2026-09-13): o arquivo
    // de toolchain do kit; o relatorio do sysroot; a proposta de kit lida de
    // um SDK — nada disso grava ate' `applyProposal`.
    property string toolchainFile: ""
    // O CMSIS-SVD do kit (P3, 0.118.0): os registradores de periferico no
    // depurador (probe-rs). Como o toolchainFile: sinal proprio, campo do kit.
    property string svdFile: ""
    property var sysrootReport: ({})
    property var kitProposal: ({})
    property string importError: ""
    signal installableRequested()
    signal installRequested(string id)

    // O PROVEDOR DE INSTALACAO (integracoes/39 §5, 2026-09-13): o catalogo
    // que o core publica, com URL/tamanho/sha256 VISIVEIS antes do clique;
    // `installing` e' o id em andamento (um por vez: sao centenas de MB).
    property var installable: []
    property string installRoot: ""
    property string projectFamily: ""
    property string installing: ""
    property string lastInstallOutcome: ""
    property bool installCatalogVisible: false

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
        installable = [];
        projectFamily = "";
        installing = "";
        lastInstallOutcome = "";
        toolchainFile = "";
        svdFile = "";
        sysrootReport = ({});
        kitProposal = ({});
        importError = "";
        if (workspaceRoot !== "") {
            getRequested("");
            installableRequested();
        }
    }

    function handleInstallable(toolchains, newInstallRoot, newProjectFamily) {
        installable = toolchains === undefined || toolchains === null ? [] : toolchains;
        installRoot = newInstallRoot === undefined ? "" : newInstallRoot;
        projectFamily = newProjectFamily === undefined ? "" : newProjectFamily;
    }

    // Recomendadas primeiro, o resto na ordem do catalogo — apresentacao;
    // quem decide o que e' recomendado e' o core.
    function installableSorted() {
        const recomendadas = installable.filter(t => t.recommended === true);
        const outras = installable.filter(t => t.recommended !== true);
        return recomendadas.concat(outras);
    }

    // O catalogo mistura toolchains e firmwares (C5): o `kind` e' do core, e
    // esta e' a UNICA derivacao dele na tela (catraca de duplicacao).
    function isFirmware(entrada) {
        return entrada !== undefined && entrada !== null && entrada.kind === "firmware";
    }

    // Os firmwares JA' BAIXADOS: o que o "Gravar" oferece no lugar do build.
    function installedFirmwares() {
        return installable.filter(t => isFirmware(t) && t.installed === true);
    }

    function install(id) {
        if (installing !== "" || workspaceRoot === "") return;
        const alvo = installable.find(t => t.id === id);
        if (alvo === undefined || alvo.installed === true) return;
        installing = id;
        lastInstallOutcome = "";
        installRequested(id);
    }

    // O job acabou: o catalogo e os candidatos sao pedidos de novo — a
    // toolchain nova ja' e' candidato do kit (o core refez a deteccao).
    function handleInstalled(outcome) {
        installing = "";
        if (outcome === undefined || outcome === null) return;
        lastInstallOutcome = outcome.success === true
                ? qsTr("%1 %2 instalada em %3").arg(outcome.id).arg(outcome.version).arg(outcome.path)
                : qsTr("falhou: %1").arg(outcome.error);
        installableRequested();
        getRequested(preset);
    }

    // O que so' um processo responde (integracoes/39): a dica de sysroot do
    // compilador cross de distro, e os alvos Rust INSTALADOS. `rustTargetsKnown`
    // separa "sem rustup" (nada a dizer) de "rustup sem o alvo" (dizer o comando).
    property string sysrootHint: ""
    property var rustTargets: []
    property bool rustTargetsKnown: false

    function handleAdvice(newSysrootHint, newRustTargets, known) {
        sysrootHint = newSysrootHint === undefined || newSysrootHint === null ? "" : newSysrootHint;
        rustTargets = newRustTargets === undefined || newRustTargets === null ? [] : newRustTargets;
        rustTargetsKnown = known === true;
    }

    // O alvo Rust do kit nao esta' instalado: o comando exato, nao um aviso vago.
    function rustTargetHint() {
        if (!rustTargetsKnown || targetTriple === "") return "";
        for (let i = 0; i < rustTargets.length; i++) {
            if (rustTargets[i] === targetTriple) return "";
        }
        return qsTr("alvo Rust %1 não instalado: rustup target add %1").arg(targetTriple);
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
    // e por isso que mexer no sysroot nao apaga o alvo. O arquivo de toolchain
    // ausente na chamada preserva o atual (um sinal nao carrega `undefined`).
    function applyKit(newSysroot, newTargetTriple, newChip, newToolchainFile, newSvdFile) {
        setKitRequested(preset, newSysroot, newTargetTriple, newChip,
                        newToolchainFile === undefined ? toolchainFile : newToolchainFile,
                        newSvdFile === undefined ? svdFile : newSvdFile);
    }

    function handleKitFile(newToolchainFile) {
        toolchainFile = newToolchainFile === undefined || newToolchainFile === null
                ? "" : newToolchainFile;
    }

    function handleKitSvd(newSvdFile) {
        svdFile = newSvdFile === undefined || newSvdFile === null ? "" : newSvdFile;
    }

    // O caminho do campo "Pasta/SDK": a tela o le daqui, para o seletor de
    // pasta e a digitacao escreverem no mesmo lugar.
    property string importPath: ""

    function pickImportPath() {
        folderPickRequested("kitPath", importPath !== "" ? importPath : "");
    }

    function handlePickedPath(purpose, path) {
        if (purpose === "kitPath") {
            importPath = path;
        }
    }

    function inspectSysroot(path) {
        const limpo = path === undefined ? "" : String(path).trim();
        if (limpo === "") return;
        inspectSysrootRequested(limpo);
    }

    function handleSysrootReport(report) {
        sysrootReport = report === undefined || report === null ? ({}) : report;
    }

    // Uma linha: o veredito do core, mais o que ha' (libc, .pc).
    function sysrootSummary() {
        const r = sysrootReport;
        if (r.verdict === undefined) return "";
        return r.verdict;
    }

    function importKit(path) {
        const limpo = path === undefined ? "" : String(path).trim();
        if (limpo === "") return;
        kitProposal = ({});
        importError = "";
        importKitRequested(limpo);
    }

    function handleKitProposal(proposal) {
        kitProposal = proposal === undefined || proposal === null ? ({}) : proposal;
        importError = "";
    }

    readonly property bool hasKitProposal: kitProposal.kind !== undefined

    // A proposta em linhas: o que o SDK declarou, para o autor ler antes de aplicar.
    function proposalLines() {
        const p = kitProposal;
        if (p.kind === undefined) return [];
        const linhas = [qsTr("%1: %2").arg(p.kind).arg(p.path)];
        if (p.cCompiler !== undefined) linhas.push(qsTr("CC: %1").arg(p.cCompiler));
        if (p.cxxCompiler !== undefined) linhas.push(qsTr("CXX: %1").arg(p.cxxCompiler));
        if (p.gdb !== undefined) linhas.push(qsTr("gdb: %1").arg(p.gdb));
        if (p.sysroot !== undefined) linhas.push(qsTr("sysroot: %1").arg(p.sysroot));
        if (p.targetTriple !== undefined) linhas.push(qsTr("alvo: %1").arg(p.targetTriple));
        if (p.toolchainFile !== undefined) linhas.push(qsTr("toolchain file: %1").arg(p.toolchainFile));
        if (p.hint !== undefined) linhas.push(p.hint);
        return linhas;
    }

    // Aplica a proposta ao kit: sysroot, alvo e arquivo de toolchain, num
    // gesto so'. O chip fica como esta' (a proposta nao fala de chip).
    function applyProposal() {
        const p = kitProposal;
        if (p.kind === undefined || workspaceRoot === "") return;
        setKitRequested(preset,
                        p.sysroot === undefined ? "" : p.sysroot,
                        p.targetTriple === undefined ? "" : p.targetTriple,
                        chip,
                        p.toolchainFile === undefined ? "" : p.toolchainFile,
                        svdFile);
    }

    function handleFailed(method, message) {
        if (method === "toolchain.importKit") {
            kitProposal = ({});
            importError = message;
            return;
        }
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
