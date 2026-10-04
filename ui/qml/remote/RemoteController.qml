pragma ComponentBehavior: Bound
import QtQuick

// Estado do ALVO LINUX POR SSH (P6 fatia 1 do roadmaps/42, 2026-09-17): a
// Raspberry Pi, a placa com imagem propria, como recurso do projeto.
//
// Guarda o que o core respondeu e o que o autor edita; quem valida, compoe e
// le' a sonda e' o core. NAO HA' SENHA AQUI (SSH e' por chave; o que o `ssh`
// perguntar, pergunta no terminal). Pede por sinal e recebe do roteador.
Item {
    id: root

    property string workspaceRoot: ""
    property var targets: []
    property string selectedName: ""
    property string errorText: ""
    property var draft: root.emptyDraft()

    // Veredito da ultima sonda (remote.probe -> event.remote.probed).
    property bool probing: false
    property string probedName: ""
    property bool probeOk: false
    property string probeArch: ""
    property string probeKernel: ""
    property var probeTools: []
    property string probeMessage: ""

    // O deploy e o programa NO ALVO (relativo entra no deployDir).
    property bool deploying: false
    property string deployMessage: ""
    property bool deployOk: false
    property string deploySource: ""
    property string program: ""

    // O que o ultimo `remote.command` produziu, e para que fim foi pedido.
    property string pendingKind: ""
    property string lastCommand: ""
    property string lastOutcome: ""

    // A seccao aberta (R1/V2): dono aqui porque sobrevive a fechar a janela e
    // porque a acao primaria LEVA a pessoa ate' onde o gesto vive.
    property string section: "visao"
    readonly property bool configuring: section === "configurar"

    // Por que a sonda falhou, TIPADO pelo core: o gesto sai disto, nunca da frase.
    property string probeFailure: ""
    // QUANDO a sonda mediu (V4: "sondado uma vez" nao parece "conectado
    // agora"); 0 = nunca nesta sessao.
    property double probedAt: 0
    // Uma linha ARMADA: composta pelo core, a vista, esperando um gesto explicito.
    property string armedCommand: ""
    property string armedName: ""
    // A linha da chave rodou no terminal: o proximo gesto e' Sondar de novo.
    property bool keySent: false


    signal listRequested()
    signal windowRequested()
    signal saveRequested(var target)
    signal removeRequested(string name)
    signal probeRequested(string name)
    signal deployRequested(string name, string source, string dest)
    signal commandRequested(string name, string kind, string program, int port)
    // O resultado de `remote.command` vira configuracao, kit ou terminal.
    signal runConfigRequested(string name, string command)
    signal kitRemoteRequested(string remoteTarget, string debugServer)
    signal shellRequested(string command)

    visible: false

    // Os FILHOS (roadmap 48 §8.3; nascem quando a catraca manda): o setup
    // guarda o que a MAQUINA tem; a fachada, o alvo DESTE projeto.
    readonly property alias setup: setupController

    RemoteSetupController {
        id: setupController

        existingNames: root.targets.map(target => target.name)
        onProposalReady: function(target) { root.useProposal(target); }
    }

    readonly property RemoteContactLog contacts: RemoteContactLog {}
    // Confiar no servidor na primeira conexao (0.153.0): confiou, sonda de novo.
    readonly property alias trust: trustController

    RemoteTrustController {
        id: trustController

        onTrusted: name => { if (name === root.selectedName) root.probe(); }
    }

    // O espelho e o sync: o alvo chega por property e a selecao volta por sinal.
    readonly property alias workspace: workspaceController

    RemoteWorkspaceController {
        id: workspaceController

        targetName: root.selectedName
        targetReady: root.selectedSaved
        onSelectRequested: function(name) {
            if (root.targetByName(name) !== null) {
                root.select(name);
            }
        }
        onCommandComposed: function(command) { root.lastCommand = command; }
    }

    onWorkspaceRootChanged: {
        targets = [];
        selectedName = "";
        draft = emptyDraft();
        clearVerdict();
        errorText = "";
        lastCommand = "";
        lastOutcome = "";
        workspace.reset();
        if (workspaceRoot !== "") {
            listRequested();
        }
    }

    function emptyDraft() {
        return { name: "", host: "", user: "", port: 22, identityFile: "", deployDir: "" };
    }


    // TUDO que o setup propoe vira rascunho aqui; o ausente fica para o OpenSSH.
    function useProposal(target) {
        if (!target || !target.host) {
            return;
        }
        // Proposta de OUTRO alvo: "nao salvo", com "Salvar alvo" no cartao.
        if ((target.name || "") !== selectedName) { selectedName = ""; clearVerdict(); }
        draft = Object.assign(cloneTarget(target), { port: target.port || 0 }); // 0: o ssh decide
    }

    // "Usar o SSH que ja' funciona": o alias e' nome e host, e nada mais.
    function useAlias(alias) {
        const nome = (alias || "").trim();
        if (nome !== "") {
            useProposal({ name: nome, host: nome });
        }
    }

    function selectSection(id) {
        if (id !== "") {
            section = id;
        }
    }

    // Pedir a janela (acoplada desde 2026-10-04): quem a abre e' o shell.
    function open() {
        windowRequested();
    }

    // A janela ficou visivel: comeca na visao geral e le' o que falta.
    function prepare() {
        // "Visao geral como padrao" — decisao do autor em 2026-09-24. Ela so'
        // funciona porque a seccao deixou de abrir em branco: sem alvo, ela diz
        // o que falta, e a ACAO PRIMARIA leva a Configurar num clique.
        section = "visao";
        if (targets.length === 0) {
            listRequested();
        }
        // Ler o `~/.ssh/config` e' local e barato; deixar o autor pedir
        // "Procurar" para so' depois descobrir que ja' havia alias e' o
        // atrito que esta fatia existe para remover.
        if (setup.discovery === "idle") {
            setup.discover();
        }
    }

    function clearVerdict() {
        probing = false;
        probedAt = 0;
        probedName = "";
        probeOk = false;
        probeArch = "";
        probeKernel = "";
        probeTools = [];
        probeMessage = "";
        probeFailure = "";
        deploying = false;
        deployMessage = "";
        disarm();
    }

    function targetByName(name) {
        return targets.find(function(item) { return item.name === name; }) || null;
    }

    // Campo a campo: o alvo tem `deny_unknown_fields` (uma senha nem chega la').
    function cloneTarget(source) {
        return {
            name: source.name || "",
            host: source.host || "",
            user: source.user || "",
            port: source.port || 22,
            identityFile: source.identityFile || "",
            deployDir: source.deployDir || "", program: source.program || "", deploySource: source.deploySource || ""
        };
    }

    function select(name) {
        selectedName = name;
        const encontrado = targetByName(name);
        draft = encontrado === null ? emptyDraft() : cloneTarget(encontrado);
        program = draft.program || ""; deploySource = draft.deploySource || "";
        clearVerdict();
    }

    function startNew() {
        selectedName = "";
        draft = emptyDraft();
        clearVerdict();
    }

    function editDraft(field, value) {
        const atualizado = cloneTarget(draft);
        atualizado[field] = field === "port" ? (parseInt(value, 10) || 0) : value;
        draft = atualizado;
    }

    // Alvo NOVO com o nome de outro: o core o SUBSTITUIRIA (save por nome).
    function save() {
        const name = draft.name.trim();
        errorText = name !== selectedName && targetByName(name) !== null
                    ? qsTr("já existe um alvo “%1” — escolha outro nome").arg(name) : "";
        if (errorText === "") saveRequested(cloneTarget(draft));
    }

    function remove() {
        if (selectedName !== "") {
            removeRequested(selectedName);
        }
    }

    // A sonda e o deploy sao do alvo SALVO (o que esta' no remotes.json).
    readonly property bool selectedSaved: selectedName !== "" && targetByName(selectedName) !== null

    function probe() {
        if (!selectedSaved) {
            return;
        }
        clearVerdict();
        probing = true;
        probeRequested(selectedName);
    }

    function deploy() {
        if (!selectedSaved) {
            return;
        }
        rememberRun();
        deploying = true;
        deployMessage = "";
        deployRequested(selectedName, deploySource, "");
    }

    function requestCommand(kind) {
        if (!selectedSaved) {
            return;
        }
        rememberRun();
        pendingKind = kind;
        lastOutcome = "";
        commandRequested(selectedName, kind, program, 0);
    }

    // Programa e origem ficam NO ALVO (0.153.0): reabrir a IDE nao os perde.
    function rememberRun() {
        const saved = targetByName(selectedName);
        if (saved !== null && ((saved.program || "") !== program || (saved.deploySource || "") !== deploySource))
            saveRequested(Object.assign(cloneTarget(saved), { program: program, deploySource: deploySource }));
    }

    function handleTargets(newTargets) {
        targets = newTargets;
        errorText = "";
        if (selectedName === "" && draft.name !== "" && targetByName(draft.name.trim()) !== null) {
            selectedName = draft.name.trim();
            draft = cloneTarget(targetByName(selectedName));
        }
        if (selectedName !== "" && targetByName(selectedName) === null) {
            startNew();
        }
        // Lista sem selecao: cair no primeiro (foto de 2026-09-24: com dois
        // alvos e nenhum escolhido o painel dizia "nenhum alvo ainda").
        if (selectedName === "" && targets.length > 0) {
            select(targets[0].name);
        }
    }

    function handleJobAccepted(method, jobId, command) {
        lastCommand = command;
    }

    function handleProbed(outcome) {
        probing = false;
        keySent = false;
        probedName = outcome.name || "";
        probeOk = outcome.success === true;
        probeArch = outcome.arch || "";
        probeKernel = outcome.kernel || "";
        probeTools = outcome.tools || [];
        probeMessage = probeOk ? "" : (outcome.error || qsTr("a sonda falhou"));
        probeFailure = probeOk ? "" : (outcome.failure || "other");
        probedAt = Date.now();
        trust.follow(probeFailure, probedName);
        contacts.record(outcome);
    }

    // DONO UNICO de "oferecer copiar a chave"; com linha armada, sai de cena.
    readonly property bool canCopyId: !probeOk && probeFailure === "authentication"
                                      && probedName !== "" && selectedSaved
                                      && armedCommand === ""

    function copyId() {
        if (!canCopyId) {
            return;
        }
        pendingKind = "copyId";
        lastOutcome = "";
        commandRequested(selectedName, "copyId", "", 0);
    }

    function disarm() {
        armedCommand = "";
        armedName = "";
    }

    // A sessao nao nasceu e a linha caiu: a promessa se desmente onde apareceu.
    function reportShellDropped(command) {
        lastOutcome = qsTr("o terminal não abriu; a linha NÃO foi enviada: %1").arg(command);
    }

    // So' aqui algo sai para o terminal, depois de a linha estar na tela. A IDE
    // nao digita senha e nao roda sozinha (a chave nova, o ssh-keygen pergunta).
    function runArmed() {
        if (armedCommand === "") {
            return;
        }
        const linha = armedCommand;
        disarm();
        keySent = true;
        shellRequested(linha);
        lastOutcome = qsTr("rodando no terminal: digite a senha do alvo lá, uma vez; depois, Sondar");
    }

    function handleDeployed(outcome) {
        deploying = false;
        deployOk = outcome.success === true;
        deployMessage = outcome.success === true
            ? qsTr("enviado para %1:%2").arg(outcome.name).arg(outcome.dest)
            : qsTr("deploy falhou: %1").arg(outcome.error || "");
    }

    // O resultado PURO vira a acao pedida; o dono e' de fora (config, kit, terminal).
    function handleCommand(result) {
        lastCommand = result.command || "";
        const kind = pendingKind;
        pendingKind = "";
        if (kind === "run" || kind === "debugpy") {
            runConfigRequested(result.name, result.command);
            lastOutcome = kind === "debugpy"
                ? qsTr("configuracao \"%1\" salva; attach em %2").arg(result.name).arg(result.remoteTarget || "")
                : qsTr("configuracao \"%1\" salva").arg(result.name);
        } else if (kind === "debugServer") {
            kitRemoteRequested(result.remoteTarget || "", result.command);
            lastOutcome = qsTr("kit: remoteTarget %1 e debugServer gravados").arg(result.remoteTarget || "");
        } else if (kind === "shell") {
            shellRequested(result.command);
            lastOutcome = qsTr("shell no terminal: %1").arg(result.command);
        } else if (kind === "copyId") {
            // ARMA, nao roda: a linha aparece e espera confirmacao.
            armedCommand = result.command || "";
            armedName = result.name || "";
            lastOutcome = qsTr("confira a linha e confirme para rodar no terminal");
        }
    }

    function handleFailed(method, message) {
        if (method === "remote.hostKey" || method === "remote.trustHost") {
            trust.handleFailed(message);
            return;
        }
        if (method === "remote.discover" || method === "remote.resolve") {
            setup.handleFailed(method);
            errorText = message;
            return;
        }
        if (method.indexOf("remote.") === 0) {
            probing = false;
            deploying = false;
            pendingKind = "";
            workspace.handleFailed();
            errorText = message;
        }
    }
}
