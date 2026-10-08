pragma ComponentBehavior: Bound
import QtQuick

// Estado da OBSERVABILIDADE (etapa 27 do roadmaps/35).
//
// Guarda o que o core respondeu sobre o Grafana deste workspace e o que o
// autor esta' editando. NAO decide nada: quem valida o endereco, quem sabe de
// onde vem o token e quem conversa por HTTP e' o core.
//
// O TOKEN VIVE AQUI E SO' AQUI, em `sessionToken`, e some sozinho:
//   - ao fechar o painel;
//   - ao trocar de workspace;
//   - ao esquecer a instancia.
// Ele nunca vai para o perfil (que e' o que o core persiste) e nunca aparece
// no log do cliente, que redige por nome de campo. Mesma regra da senha de
// banco; ver `DocsPublic/seguranca/40`.
//
// Nao fala com o CoreClient direto: pede por sinal e recebe do roteador.
Item {
    id: root

    property string workspaceRoot: ""
    property bool panelVisible: false

    // A instancia salva. `hasInstance` distingue "nao ha' nenhuma" de "ha' uma
    // com campos vazios": a tela desenha um convite no primeiro caso e um
    // formulario preenchido no segundo.
    property bool hasInstance: false
    property var profile: root.emptyProfile()
    property string errorText: ""

    // O rascunho do formulario, separado do que esta' salvo. Sem ele, digitar
    // no campo ja' teria mudado o que a tela diz estar gravado.
    property var draft: root.emptyProfile()

    // "ESTA POLITICA USA UMA VARIAVEL DE AMBIENTE?" — um fato, um dono. A
    // pergunta era feita duas vezes, aqui e no painel, e o gate de duplicacao
    // pegou: duas copias da mesma derivacao divergem em silencio, como as duas
    // copias de `isWordChar` divergiram (roadmaps/39 §5).
    readonly property bool draftUsesVariable: root.draft.tokenSource === "environment"

    // O que a ultima sonda achou.
    property bool probing: false
    property bool reachable: false
    property bool authenticated: false
    property string version: ""
    property string database: ""
    property string message: ""
    property var dataSources: []
    property var dashboards: []
    // O CRUZAMENTO E' O PONTO DO PAINEL: quais bancos DESTE projeto o Grafana
    // ja' observa. Um link para o Grafana e' um favorito; isto e' integracao.
    property var matches: []

    // O core disse que PEDIR O TOKEN resolve. A UI abre o campo por este
    // booleano, nunca lendo `message`.
    property bool tokenRequired: false
    // O SERVIDOR RECUSOU o token desta sessao — diferente de "pediu um".
    property bool authFailed: false

    // QUANDO a ultima medida aconteceu, e SOBRE QUAL endereco. Sem os dois, a
    // tela nao tem como dizer "medido agora" sem mentir, nem como saber que o
    // que esta' nela pertence a outra instancia (§6).
    property double probedAt: 0
    property string resultUrl: ""

    // Reavaliado a cada tique: idade envelhece sozinha.
    property double agora: Date.now()

    // O RASCUNHO DIVERGIU do perfil salvo: a tela volta para a configuracao, e
    // o resultado anterior deixa de valer como prova do que esta' escrito.
    property bool editing: false

    GrafanaStateRules {
        id: regras
    }

    GrafanaActionRules {
        id: acoes
    }

    readonly property var facts: ({
        "hasInstance": root.hasInstance,
        "editing": root.editing,
        "probing": root.probing,
        "probedAt": root.probedAt,
        "reachable": root.reachable,
        "authenticated": root.authenticated,
        "tokenRequired": root.tokenRequired,
        "authFailed": root.authFailed,
        "dataSourceCount": root.dataSources.length,
        "dashboardCount": root.dashboards.length,
        "url": root.draft.url,
        "resultUrl": root.resultUrl,
        "agora": root.agora
    })

    readonly property var panelState: regras.stateFor(root.facts)
    readonly property var primaryAction: acoes.primaryFor(root.panelState, root.facts)
    readonly property bool setupExpanded: acoes.setupExpanded(root.panelState)
    readonly property bool authVisible: acoes.authVisible(root.panelState)
    readonly property string contentPhrase: acoes.contentPhrase(root.panelState, root.facts)
    readonly property string statusPhrase:
        acoes.statusPhrase(root.panelState, root.facts, root.primaryAction.hint, regras)

    Timer {
        // Meio minuto: a frase mais curta fala em minutos, entao nunca se
        // mostra idade errada por muito tempo.
        interval: 30000
        running: root.panelVisible
        repeat: true
        onTriggered: root.agora = Date.now()
    }
    // Token da sessao. Nunca persistido, nunca enviado ao `save`.
    property string sessionToken: ""
    // A QUEM ELE PERTENCE (§7.1): workspace + URL CONFIRMADA. Guardar o token
    // sem o dono e' o que permite manda-lo para a instancia errada depois de
    // uma edicao — uma requisicao que "funciona" e vaza.
    property string sessionTokenContext: ""
    // A VIEW SABE QUE HA' UM TOKEN, NUNCA QUAL (§7.2 regra 8). E' o que basta
    // para oferecer "Esquecer credencial" sem passar a credencial adiante.
    readonly property bool hasSessionToken: root.sessionToken !== ""

    readonly property string credentialContext:
        regras.credentialContext(root.workspaceRoot, root.profile.url)

    // O QUE FOI PEDIDO, para carimbar o que voltar. Sem isto a resposta era
    // carimbada com o rascunho de AGORA: editar a URL durante a sonda fazia o
    // resultado antigo passar por medida do endereco novo.
    property string pendingContext: ""
    // SALVAR E MEDIR SAO UM GESTO SO' (§5.1). O `Conectar` prometia testar o
    // endereco e so' gravava; a medida vem quando o perfil volta do core.
    property bool probeAfterProfile: false

    signal windowRequested()
    signal getRequested()
    signal saveRequested(var profile)
    signal forgetRequested()
    signal probeRequested(string token)

    visible: false

    onWorkspaceRootChanged: {
        hasInstance = false;
        profile = emptyProfile();
        draft = emptyProfile();
        clearToken();
        clearProbe();
        errorText = "";
        if (workspaceRoot !== "") {
            getRequested();
        }
    }

    // O padrao e' o caso comum: um Grafana local, sem token, que ja' responde
    // a versao. Pedir o token de cara seria um obstaculo que a IDE inventou —
    // o mesmo raciocinio que tirou o prompt de senha do banco.
    function emptyProfile() {
        return {
            url: "http://localhost:3000",
            tokenSource: "none",
            tokenVariable: ""
        };
    }

    function clearToken() {
        sessionToken = "";
        sessionTokenContext = "";
        tokenRequired = false;
        authFailed = false;
    }

    function clearProbe() {
        probedAt = 0;
        resultUrl = "";
        authFailed = false;
        probing = false;
        reachable = false;
        authenticated = false;
        version = "";
        database = "";
        message = "";
        dataSources = [];
        dashboards = [];
        matches = [];
    }

    // Janela acoplada desde 2026-10-04 (59 §6.1): abrir PEDE a janela; ela,
    // ao aparecer, chama `prepare` (o relogio anda e o perfil vem).
    function open() {
        windowRequested();
    }

    function prepare() {
        panelVisible = true;
        if (workspaceRoot !== "") getRequested();
    }

    function close() {
        panelVisible = false;
        // O TOKEN SOBREVIVE A FECHAR O PAINEL — decisao de 2026-09-22 (§7),
        // que mudou a fronteira de proposito: abrir e fechar uma tool window
        // virava um login novo a cada vez. Ele continua morrendo em tudo que
        // significa "outra instancia ou outro projeto": trocar de workspace,
        // confirmar outra URL, esquecer, ser recusado, ou o processo acabar.
    }

    // §7.2 regra 4: apagar a credencial e' gesto EXPLICITO e disponivel mesmo
    // autenticado — nao um efeito colateral de fechar uma janela.
    function forgetCredential() {
        clearToken();
        // O que esta' na tela foi obtido COM o token; sem ele deixa de ser
        // prova do que a IDE pode ver agora.
        clearProbe();
    }

    function setDraftField(campo, valor) {
        if (campo === "url" && valor !== root.draft.url) {
            // Trocar o endereco invalida o resultado anterior VISUALMENTE
            // (§6): o que esta' na tela pertence a outra instancia.
            root.editing = true;
        }
        const copia = {};
        for (const chave in draft) {
            copia[chave] = draft[chave];
        }
        copia[campo] = valor;
        draft = copia;
    }

    // O gesto de `Conectar`: grava e mede. A sonda sai quando o core devolver
    // o perfil, e nao antes — medir o que ainda nao foi confirmado mediria o
    // endereco velho.
    function connect() {
        probeAfterProfile = true;
        save();
    }

    function save() {
        if (draft.url !== profile.url) {
            // CONFIRMAR OUTRA URL APAGA O TOKEN ANTES de qualquer conversa com
            // ela (§7). A credencial foi dada para a instancia anterior.
            clearToken();
            clearProbe();
        }
        const enviar = {
            url: draft.url,
            tokenSource: draft.tokenSource
        };
        // Campo ausente e campo vazio sao coisas diferentes para o core, e o
        // perfil recusa campo desconhecido: so' mande `tokenVariable` quando a
        // politica for a que o usa.
        if (draftUsesVariable && draft.tokenVariable !== "") {
            enviar.tokenVariable = draft.tokenVariable;
        }
        errorText = "";
        saveRequested(enviar);
    }

    function forget() {
        clearProbe();
        clearToken();
        forgetRequested();
    }

    function probe() {
        errorText = "";
        // O RESULTADO ANTERIOR NAO E' APAGADO AQUI. A §6 e' explicita: "falha
        // de atualizacao pode manter o ultimo resultado como DESATUALIZADO,
        // nunca como resultado novo". Limpar antes de perguntar tornava essa
        // frase impossivel de cumprir — uma sonda que falhasse deixava a tela
        // vazia, e o autor perdia o que ja' sabia por ter tentado saber mais.
        // Quem apaga e' `clearProbe`, nas trocas que invalidam de verdade.
        probing = true;
        pendingContext = root.credentialContext;
        // SO' O DONO VIAJA: token de outro par workspace+URL nao vai junto,
        // mesmo estando em memoria. Ele nao e' apagado aqui — o par antigo
        // pode voltar — mas tambem nao e' oferecido a quem nao o pediu.
        const credencial = regras.sameContext(root.pendingContext, root.sessionTokenContext)
                         ? root.sessionToken : "";
        probeRequested(credencial);
    }

    // O autor respondeu ao pedido de token: guarda na sessao e tenta de novo.
    function probeWithToken(token) {
        sessionToken = token;
        sessionTokenContext = root.credentialContext;
        tokenRequired = false;
        authFailed = false;
        probe();
    }

    function handleProfile(perfil, existe) {
        hasInstance = existe;
        profile = existe ? perfil : emptyProfile();
        draft = existe ? perfil : emptyProfile();
        errorText = "";
        // O perfil voltou do core: o rascunho e' ele, e nao ha' mais divergencia.
        editing = false;
        if (!existe) {
            clearProbe();
        }
        if (probeAfterProfile) {
            probeAfterProfile = false;
            if (existe) {
                probe();
            }
        }
    }

    function handleProbed(resultado) {
        // RESPOSTA ATRASADA DA URL ANTIGA E' DESCARTADA (§7.1). Entre pedir e
        // responder, o autor pode ter confirmado outra instancia; aceitar isto
        // aqui mostraria o Grafana de ontem como medida de agora.
        if (!regras.sameContext(root.pendingContext, root.credentialContext)) {
            probing = false;
            return;
        }
        probing = false;
        // A MEDIDA TEM HORA E DONO: sem isto, "medido agora" seria afirmacao
        // sem prova, e trocar de endereco deixaria o resultado velho passando
        // por novo. O carimbo e' o que foi PEDIDO, nao o que esta' no rascunho
        // agora — o rascunho pode ter mudado no meio.
        probedAt = Date.now();
        agora = probedAt;
        resultUrl = root.profile.url;
        reachable = resultado.reachable === true;
        authenticated = resultado.authenticated === true;
        // RECUSA E' DIFERENTE DE AUSENCIA, e desde o protocolo 0.136.0 o core
        // diz qual das duas foi. Antes a UI so' via `authenticated: false` —
        // que e' TAMBEM o que ela ve quando ninguem ofereceu token — e quem
        // colava uma credencial errada lia "sem autenticacao", sem caminho de
        // volta para tentar outra. Achado contra um Grafana de verdade, em
        // 2026-09-26; nenhum harness veria, porque o mock nunca recusou.
        if (resultado.authRefused === true) {
            // O core so' marca recusa quando ELE enviou uma credencial e levou
            // 401/403 — nao ha' o que deduzir aqui. Mesma regra do
            // `SECRET_REQUIRED` (§7): o que o servidor negou nao fica em
            // memoria esperando ser reenviado. `clearToken` zera as duas
            // bandeiras, por isso elas voltam depois dele.
            clearToken();
            authFailed = true;
            tokenRequired = true;
        } else {
            authFailed = false;
        }
        version = resultado.version !== undefined ? resultado.version : "";
        database = resultado.database !== undefined ? resultado.database : "";
        message = resultado.message !== undefined ? resultado.message : "";
        dataSources = resultado.dataSources !== undefined ? resultado.dataSources : [];
        dashboards = resultado.dashboards !== undefined ? resultado.dashboards : [];
        matches = resultado.matches !== undefined ? resultado.matches : [];
    }

    function handleFailed(method, message, code) {
        if (method.indexOf("grafana.") !== 0) {
            return;
        }
        probing = false;
        if (code === "SECRET_REQUIRED") {
            // Pedir um token e' diferente de ter o token recusado: se ja'
            // havia um nesta sessao, o servidor o rejeitou.
            const recusado = root.sessionToken !== "";
            if (recusado) {
                // TOKEN REJEITADO E' APAGADO ANTES DE PEDIR OUTRO (§7): manter
                // em memoria o que o servidor ja' negou so' arrisca reenvia-lo.
                clearToken();
            }
            authFailed = recusado;
            tokenRequired = true;
            // NADA FOI MEDIDO, e por isso nada e' carimbado. O core recusa
            // `SECRET_REQUIRED` na RESOLUCAO DA POLITICA — politica `prompt`
            // sem token, ou variavel de ambiente ausente —, antes de qualquer
            // chamada ao Grafana (`handlers/grafana.rs`,
            // `resolve_grafana_token`). Eu carimbava hora e endereco aqui, e a
            // tela dizia "medido agora" sobre uma instancia com a qual ninguem
            // falou; o eixo `probe` ainda virava "failed", abrindo a
            // configuracao como se o ENDERECO tivesse problema. A medida
            // anterior, se houver, continua sendo a ultima verdadeira.
            errorText = "";
            return;
        }
        // O `Conectar` que falhou ao GRAVAR nao vira sonda: a promessa era
        // testar o endereco confirmado, e nao ha' endereco confirmado.
        probeAfterProfile = false;
        // A INSTANCIA NAO RESPONDEU — mas o que ela respondeu ANTES continua
        // na tela, marcado como velho pelo eixo `content`. Apagar seria punir
        // quem tentou atualizar.
        reachable = false;
        authenticated = false;
        errorText = message;
    }

    // A URL completa de um dashboard: o `url` que o Grafana devolve e' um
    // caminho relativo a instancia.
    function dashboardUrl(caminho) {
        return profile.url + caminho;
    }
}
