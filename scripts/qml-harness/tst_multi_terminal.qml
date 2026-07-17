import QtQuick
// Carrega o RuntimeController.qml REAL (arquivo do projeto, sem copia).
import "../../ui/qml/runtime"

Item {
    id: root
    width: 100
    height: 100

    property int openedRequests: 0
    property string inputId: ""
    property string inputData: ""
    property string closeId: ""

    RuntimeController {
        id: runtime

        workspaceRoot: "/tmp/workspace"
        terminalActive: true
        onTerminalOpenRequested: root.openedRequests += 1
        onTerminalInputRequested: function(id, data) {
            root.inputId = id;
            root.inputData = data;
        }
        onTerminalCloseRequested: function(id) {
            root.closeId = id;
        }
    }

    // O dono do Assistente. O RuntimeController acima nao sabe que ele existe:
    // recebe rotulo + `kind` opaco e devolve o `kind` em `terminalOpened`.
    AssistantController {
        id: agentes

        runtimeController: runtime
        // Formato do `tools.detect`: o mesmo de cargo/clangd. `codex` ausente
        // de proposito — o seletor mostra a sugestao e nao deixa escolher.
        toolsList: [
            { id: "claude", displayName: "Claude Code", status: "detected",
              path: "/usr/bin/claude", version: "1.0" },
            { id: "codex", displayName: "Codex CLI", status: "missing",
              path: "", suggestedInstall: "npm i -g @openai/codex" }
        ]
    }

    Component.onCompleted: {
        let failures = 0;

        // Cada resposta de open materializa uma aba e a mais nova fica ativa.
        runtime.handleTerminalOpened("t1", "/bin/sh");
        runtime.handleTerminalRender({ id: "t1", marker: "primeiro" });
        runtime.handleTerminalOpened("t2", "/bin/sh");
        runtime.handleTerminalRender({ id: "t2", marker: "segundo" });
        if (runtime.terminalsModel.count !== 2) failures += 1;
        if (runtime.activeTerminalId !== "t2") failures += 2;
        if (runtime.terminalRender.marker !== "segundo") failures += 4;

        // Trocar de aba restaura o ultimo grid daquela sessao e o input leva id.
        runtime.selectTerminal("t1");
        if (runtime.terminalRender.marker !== "primeiro") failures += 8;
        runtime.sendTerminalKey("x");
        if (root.inputId !== "t1" || root.inputData !== "x") failures += 16;

        // Fechar a ativa seleciona uma vizinha sem destruir o estado dela.
        runtime.closeTerminal("t1");
        if (root.closeId !== "t1") failures += 32;
        runtime.handleTerminalClosed("t1");
        if (runtime.terminalsModel.count !== 1) failures += 64;
        if (runtime.activeTerminalId !== "t2") failures += 128;
        if (runtime.terminalRender.marker !== "segundo") failures += 256;

        // Render atrasado de uma sessao fechada nao pode ressuscitar estado.
        runtime.handleTerminalRender({ id: "t1", marker: "atrasado" });
        runtime.selectTerminal("t2");
        if (runtime.terminalRender.marker !== "segundo") failures += 512;

        // Crash/desconexao nao emite closed individual: terminalActive limpa tudo.
        runtime.terminalActive = false;
        if (runtime.terminalsModel.count !== 0) failures += 1024;
        if (runtime.activeTerminalId !== "") failures += 2048;
        if (Object.keys(runtime.terminalRender).length !== 0) failures += 4096;

        // Sem sessao, abrir o painel pede um shell novo.
        runtime.openTerminalPanel();
        if (root.openedRequests !== 1) failures += 8192;

        // ---- Assistente: escolha na UI, sessao comum no core -------------
        //
        // O core nao sabe o que e "Assistente": para ele toda sessao e um
        // $SHELL no PTY. Rotulo e agente sao da UI. Se um dia isto virar
        // parametro do protocolo, a politica por programa que o 0.59.0 removeu
        // voltou.
        runtime.terminalActive = true;
        runtime.clearTerminals();
        agentes.clear();
        root.openedRequests = 0;

        // O SELETOR VEM ANTES DA SESSAO: o atalho nao pode abrir PTY nenhum
        // enquanto o agente nao foi escolhido.
        agentes.openAssistant();
        if (!agentes.selectorVisible) failures += 16384;
        if (root.openedRequests !== 0) failures += 32768;

        // Agente ausente nao e escolhivel: nada abre, o seletor fica de pe.
        agentes.choose("codex");
        if (root.openedRequests !== 0 || !agentes.selectorVisible) failures += 65536;

        // Escolhido um agente disponivel: o seletor some e a sessao e pedida.
        agentes.choose("claude");
        if (agentes.selectorVisible) failures += 131072;
        if (root.openedRequests !== 1) failures += 262144;

        // O core responde: aba rotulada, e o comando so entao e digitado — pelo
        // mesmo terminal.input do usuario, e vindo do `path` DETECTADO.
        runtime.handleTerminalOpened("c1", "/bin/sh");
        if (String(runtime.terminalsModel.get(0).title).indexOf("Assistente") !== 0) {
            failures += 524288;
        }
        if (runtime.terminalsModel.get(0).kind !== "assistant") failures += 1048576;
        if (root.inputId !== "c1" || root.inputData !== "/usr/bin/claude\n") {
            failures += 2097152;
        }
        if (runtime.activeTerminalId !== "c1") failures += 4194304;

        // O comando e entregue UMA VEZ SO. Uma segunda sessao de contexto que
        // apareca sem escolha nova (um open que falhou e voltou tarde, por
        // exemplo) NAO pode herdar o comando da anterior.
        root.inputData = "";
        agentes.handleTerminalOpened("c9", agentes.assistantKind);
        if (root.inputData !== "") failures += 68719476736;

        // Uma aba comum aberta depois nao e do contexto: nao recebe rotulo e o
        // agente nem e consultado sobre ela.
        root.inputId = "";
        root.inputData = "";
        runtime.handleTerminalOpened("t9", "/bin/sh");
        if (runtime.terminalsModel.get(1).kind !== "") failures += 8388608;
        if (String(runtime.terminalsModel.get(1).title).indexOf("Terminal") !== 0) {
            failures += 16777216;
        }
        if (root.inputData !== "") failures += 33554432;

        // Estado aceso do icone no rail: segue a aba ATIVA, nao a existencia.
        // Abrir "t9" acabou de ATIVA-LO, entao aqui existe um contexto vivo que
        // NAO esta ativo — o caso que separa "ativa" de "existe".
        if (agentes.activeTerminalIsAssistant) failures += 67108864;
        runtime.selectTerminal("c1");
        if (!agentes.activeTerminalIsAssistant) failures += 134217728;

        // O icone do rail exige as DUAS coisas: sessao de contexto ativa E o
        // painel do terminal na frente. Contexto ativo com o painel escondido
        // nao acende — senao o rail mentiria sobre o que esta na tela.
        runtime.terminalPanelVisible = false;
        if (agentes.assistantSessionVisible) failures += 137438953472;
        runtime.terminalPanelVisible = true;
        if (!agentes.assistantSessionVisible) failures += 274877906944;
        runtime.selectTerminal("t9");
        if (agentes.assistantSessionVisible) failures += 549755813888;
        runtime.selectTerminal("c1");

        // Com contexto vivo, o atalho FOCA em vez de acumular aba — e nao
        // reabre o seletor: ele so existe antes da sessao.
        root.openedRequests = 0;
        runtime.selectTerminal("t9");
        agentes.openAssistant();
        if (agentes.selectorVisible) failures += 268435456;
        if (root.openedRequests !== 0) failures += 536870912;
        if (runtime.activeTerminalId !== "c1") failures += 1073741824;
        if (runtime.terminalsModel.count !== 2) failures += 2147483648;

        // Fechado o contexto, o atalho oferece escolher de novo — e a numeracao
        // NAO repete.
        runtime.handleTerminalClosed("c1");
        if (agentes.activeTerminalIsAssistant) failures += 4294967296;
        agentes.openAssistant();
        agentes.choose("claude");
        runtime.handleTerminalOpened("c2", "/bin/sh");
        const contexto = runtime.terminalsModel.get(runtime.terminalsModel.count - 1);
        if (String(contexto.title) === "Assistente 1") failures += 8589934592;

        // Sem workspace o atalho e inerte (nao ha raiz para o PTY): nem seletor.
        runtime.clearTerminals();
        agentes.clear();
        runtime.workspaceRoot = "";
        root.openedRequests = 0;
        agentes.openAssistant();
        if (root.openedRequests !== 0 || agentes.selectorVisible) {
            console.warn("FALHA: atalho ativo sem workspace");
            failures += 17179869184;
        }

        // Caminho detectado com espaco vai CITADO: sem aspas o shell quebraria
        // a linha no meio do path e rodaria outra coisa.
        runtime.workspaceRoot = "/tmp/workspace";
        runtime.clearTerminals();
        agentes.clear();
        agentes.toolsList = [
            { id: "claude", displayName: "Claude Code", status: "detected",
              path: "/opt/ai tools/claude", version: "1.0" }
        ];
        root.inputData = "";
        agentes.openAssistant();
        agentes.choose("claude");
        runtime.handleTerminalOpened("c3", "/bin/sh");
        if (root.inputData !== '"/opt/ai tools/claude"\n') failures += 34359738368;
        // O codigo de saida de um processo tem 8 BITS: Qt.exit(256) sai como 0.
        // Enquanto o bitmask ia direto para o exit, todo check com bit >= 256
        // era letra morta: passava verde mesmo quebrado, que e exatamente a
        // doenca que esta suite existe para impedir. O mask agora vai para a
        // SAIDA (onde nao trunca) e o exit so diz passou/falhou.
        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
