pragma ComponentBehavior: Bound
import QtQuick

// Estado do ALVO LINUX POR SSH (P6 fatia 1 do roadmaps/42, 2026-09-17): a
// Raspberry Pi, a placa com imagem propria, como recurso do projeto.
//
// Guarda o que o core respondeu e o que o autor esta' editando. NAO decide
// nada: quem valida o perfil, quem compoe a linha `ssh …` e quem le a sonda
// e' o core. NAO HA' SENHA AQUI: SSH e' por chave, e o que o `ssh` do sistema
// precisar perguntar, pergunta no terminal da IDE.
//
// Nao fala com o CoreClient direto: pede por sinal e recebe do roteador.
Item {
    id: root

    property string workspaceRoot: ""
    property var targets: []
    property string selectedName: ""
    property bool panelVisible: false
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

    // O deploy (remote.deploy -> event.remote.deployed) e o programa NO
    // ALVO que rodar/depurar usam: relativo entra no deployDir.
    property bool deploying: false
    property string deployMessage: ""
    property string deploySource: ""
    property string program: ""

    // O que o ultimo `remote.command` produziu, e para que fim foi pedido.
    property string pendingKind: ""
    property string lastCommand: ""
    property string lastOutcome: ""

    // Qual seccao do painel esta' aberta (R1/V2). Dono aqui porque ela
    // sobrevive a fechar e reabrir o painel, e porque a acao primaria pode
    // LEVAR a pessoa ate' a seccao onde o gesto vive.
    property string section: "visao"

    // Por que a sonda falhou, TIPADO pelo core (0.133.0): authentication |
    // host | network | other. A UI escolhe o gesto por isto, nunca lendo a
    // frase — frase muda de idioma, tipo nao.
    property string probeFailure: ""
    // Uma linha ARMADA: composta pelo core e mostrada, esperando um gesto
    // explicito. Copiar chave nao pode acontecer porque alguem sondou.
    property string armedCommand: ""
    property string armedName: ""


    signal listRequested()
    signal saveRequested(var target)
    signal removeRequested(string name)
    signal probeRequested(string name)
    signal deployRequested(string name, string source, string dest)
    signal commandRequested(string name, string kind, string program, int port)
    // O que o resultado de `remote.command` vira: configuracao de execucao
    // ("Rodar em pi"), os dois campos do kit, ou uma linha no terminal.
    signal runConfigRequested(string name, string command)
    signal kitRemoteRequested(string remoteTarget, string debugServer)
    signal shellRequested(string command)

    visible: false

    // O SETUP como filho (roadmap 48 §8.3 nomeia os filhos possiveis e diz que
    // eles nao nascem preventivamente — este nasceu quando a catraca mandou).
    // Ele guarda o que a MAQUINA tem; a fachada guarda o alvo DESTE projeto.
    // Trocar de workspace nao o reinicia: o `~/.ssh/config` nao mudou.
    readonly property alias setup: setupController

    RemoteSetupController {
        id: setupController

        onProposalReady: function(target) { root.useProposal(target); }
    }

    // O espelho e o sync (roadmap 48 §8.3). Ele nao conhece o catalogo: o alvo
    // chega por property e a selecao volta por sinal.
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


    // TUDO que o setup propoe vira rascunho por aqui — o alias escolhido e a
    // linha `ssh` colada sao a mesma operacao, e ter duas funcoes quase iguais
    // era o comeco de duas verdades sobre o que um alvo novo e'.
    //
    // Campo ausente vira vazio, e porta ausente vira 0: a ponte descarta os
    // dois, entao o perfil fica so' com o que a pessoa realmente disse e o
    // OpenSSH continua decidindo o resto. Esse e' o criterio de aceite da R0.5.
    //
    // Salvar continua sendo gesto dela: ler nao e' gravar.
    function useProposal(target) {
        if (!target || !target.host) {
            return;
        }
        draft = { name: target.name || "", host: target.host,
                  user: target.user || "", port: target.port || 0,
                  identityFile: target.identityFile || "",
                  deployDir: target.deployDir || "" };
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

    function open() {
        panelVisible = true;
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

    function close() {
        panelVisible = false;
    }

    function clearVerdict() {
        probing = false;
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

    // Copia campo a campo: o alvo tem `deny_unknown_fields`, e um campo a
    // mais (ou uma senha) seria recusado — e nao deve nem chegar la'.
    function cloneTarget(source) {
        return {
            name: source.name || "",
            host: source.host || "",
            user: source.user || "",
            port: source.port || 22,
            identityFile: source.identityFile || "",
            deployDir: source.deployDir || ""
        };
    }

    function select(name) {
        selectedName = name;
        const encontrado = targetByName(name);
        draft = encontrado === null ? emptyDraft() : cloneTarget(encontrado);
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

    function save() {
        errorText = "";
        saveRequested(cloneTarget(draft));
    }

    function remove() {
        if (selectedName !== "") {
            removeRequested(selectedName);
        }
    }

    // A sonda e o deploy sao do alvo SALVO: o core so' conhece o que esta'
    // em .kinein/remotes.json.
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
        deploying = true;
        deployMessage = "";
        deployRequested(selectedName, deploySource, "");
    }

    function requestCommand(kind) {
        if (!selectedSaved) {
            return;
        }
        pendingKind = kind;
        lastOutcome = "";
        commandRequested(selectedName, kind, program, 0);
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
        // Chegou lista e nada esta' selecionado: cair no primeiro. Medido na
        // foto de 2026-09-24 — com dois alvos salvos e nenhum selecionado, o
        // painel oferecia "Salvar alvo" e dizia "nenhum alvo ainda", as duas
        // coisas falsas. E o aceite da V2 e' o contrario disso: "no segundo uso
        // do mesmo alvo, o usuario nao toca nos campos de perfil".
        if (selectedName === "" && targets.length > 0) {
            select(targets[0].name);
        }
    }

    function handleJobAccepted(method, jobId, command) {
        lastCommand = command;
    }

    function handleProbed(outcome) {
        probing = false;
        probedName = outcome.name || "";
        probeOk = outcome.success === true;
        probeArch = outcome.arch || "";
        probeKernel = outcome.kernel || "";
        probeTools = outcome.tools || [];
        probeMessage = probeOk ? "" : (outcome.error || qsTr("a sonda falhou"));
        probeFailure = probeOk ? "" : (outcome.failure || "other");
    }

    // O alvo recusou a chave: o unico gesto que resolve isso e' copiar a sua.
    // DONO UNICO de "oferecer o gesto da chave". Com uma linha ja' armada o
    // botao sai de cena — deixar a view decidir isso partiria a derivacao em
    // dois lugares que divergem calados.
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

    // So' aqui algo sai para o terminal, e so' depois de a linha ter estado
    // na tela. A IDE nao gera chave, nao digita senha e nao roda sozinha.
    function runArmed() {
        if (armedCommand === "") {
            return;
        }
        const linha = armedCommand;
        disarm();
        shellRequested(linha);
        lastOutcome = qsTr("rodando no terminal: aceite o host key e digite a senha lá, uma vez");
    }

    function handleDeployed(outcome) {
        deploying = false;
        deployMessage = outcome.success === true
            ? qsTr("enviado para %1:%2").arg(outcome.name).arg(outcome.dest)
            : qsTr("deploy falhou: %1").arg(outcome.error || "");
    }

    // O resultado PURO vira a acao que o autor pediu. Cada destino tem um
    // dono fora daqui (run configs, kit, terminal); este controller so' liga.
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
            lastOutcome = qsTr("shell aberto no terminal");
        } else if (kind === "copyId") {
            // ARMA, nao roda: a linha aparece e espera confirmacao.
            armedCommand = result.command || "";
            armedName = result.name || "";
            lastOutcome = qsTr("confira a linha e confirme para rodar no terminal");
        }
    }

    function handleFailed(method, message) {
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
