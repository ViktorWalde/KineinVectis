pragma ComponentBehavior: Bound
import QtQuick

// O SETUP do Remote: o que a MAQUINA ja' tem (0.132.0, fatia R0.5), separado
// do alvo deste projeto.
//
// Nasceu em 2026-09-24 como o roadmap 48 §8.3 previu: "RemoteController
// permanece fachada enquanto couber na catraca; se a maquina de estados
// crescer, os filhos possiveis sao RemoteSetupController, ... Eles nao nascem
// preventivamente." A catraca do `RemoteController` mandou, e este e' o corte
// que ela pedia — nao um desacoplamento por precaucao.
//
// O corte e' por PERGUNTA, nao por tamanho: aqui a resposta nao depende do
// projeto aberto (os aliases do `~/.ssh/config` e o que o `ssh -G` diz deles),
// entao trocar de workspace nao reinicia nada disto.
//
// Nao fala com o CoreClient: pede por sinal, recebe do roteador.
Item {
    id: root

    // Estado de APRESENTACAO derivado do que o core respondeu NESTA sessao —
    // nunca um fato inventado sobre a maquina.
    property string discovery: "idle"  // idle | loading | ready | failed
    property var aliases: []
    property var aliasSources: []
    // Qual host esta' sendo explicado, para descartar resposta atrasada.
    property string resolving: ""
    property var resolved: null
    // DONO UNICO de "esta' carregando": o guard daqui e o botao da view leem
    // o mesmo fato. Duas comparacoes com a string divergiriam em silencio.
    readonly property bool discovering: root.discovery === "loading"

    // "Configurar servidor" sem formulario (0.134.0): a pessoa cola a linha
    // `ssh` que ja' usa e o core a interpreta. Ideia do Remote-SSH do VS Code,
    // registrada na secao 3.1 da especificacao.
    property string pasted: ""
    property var proposalSource: []
    readonly property bool canParse: root.pasted.trim() !== ""

    signal discoverRequested()
    signal resolveRequested(string host)
    signal parseRequested(string command)
    signal proposalReady(var target)

    visible: false

    function discover() {
        if (discovering) {
            return;
        }
        discovery = "loading";
        discoverRequested();
    }

    function resolve(host) {
        const alvo = (host || "").trim();
        if (alvo === "" || resolving === alvo) {
            return;
        }
        resolving = alvo;
        resolved = null;
        resolveRequested(alvo);
    }

    function handleAliases(list, sources) {
        aliases = list || [];
        aliasSources = sources || [];
        discovery = "ready";
    }

    function handleResolved(summary) {
        // Resposta de outro host (ou de um gesto ja' abandonado) nao pode
        // trocar o resumo em tela pelo de um alvo que ninguem pediu.
        if (!summary || summary.host !== resolving) {
            return;
        }
        resolved = summary;
        resolving = "";
    }

    function parse() {
        if (!canParse) {
            return;
        }
        proposalSource = [];
        parseRequested(pasted.trim());
    }

    // O core devolveu a leitura. NADA foi salvo: a proposta vai para o
    // rascunho, a procedencia fica na tela, e quem grava e' a pessoa.
    function handleParsed(result) {
        if (!result || !result.target) {
            return;
        }
        proposalSource = result.source || [];
        proposalReady(result.target);
    }

    // Cada falha no seu estado: descobrir falhar nao diz nada sobre resolver.
    function handleFailed(method) {
        if (method === "remote.discover") {
            discovery = "failed";
        } else if (method === "remote.resolve") {
            resolving = "";
        } else if (method === "remote.parseCommand") {
            proposalSource = [];
        }
    }
}
