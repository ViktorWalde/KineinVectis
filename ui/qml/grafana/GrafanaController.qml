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
// banco; ver `docs/seguranca/40`.
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
    // Token da sessao. Nunca persistido, nunca enviado ao `save`.
    property string sessionToken: ""

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
        tokenRequired = false;
    }

    function clearProbe() {
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

    function open() {
        panelVisible = true;
        if (workspaceRoot !== "") {
            getRequested();
        }
    }

    function close() {
        panelVisible = false;
        // O TOKEN MORRE COM O PAINEL. Deixa-lo vivo seria guardar credencial
        // por conveniencia, que e' exatamente o habito que a decisao de
        // 2026-09-04 fechou.
        clearToken();
    }

    function setDraftField(campo, valor) {
        const copia = {};
        for (const chave in draft) {
            copia[chave] = draft[chave];
        }
        copia[campo] = valor;
        draft = copia;
    }

    function save() {
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
        probing = true;
        clearProbe();
        probing = true;
        probeRequested(sessionToken);
    }

    // O autor respondeu ao pedido de token: guarda na sessao e tenta de novo.
    function probeWithToken(token) {
        sessionToken = token;
        tokenRequired = false;
        probe();
    }

    function handleProfile(perfil, existe) {
        hasInstance = existe;
        profile = existe ? perfil : emptyProfile();
        draft = existe ? perfil : emptyProfile();
        errorText = "";
        if (!existe) {
            clearProbe();
        }
    }

    function handleProbed(resultado) {
        probing = false;
        reachable = resultado.reachable === true;
        authenticated = resultado.authenticated === true;
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
            tokenRequired = true;
            errorText = "";
            return;
        }
        errorText = message;
    }

    // A URL completa de um dashboard: o `url` que o Grafana devolve e' um
    // caminho relativo a instancia.
    function dashboardUrl(caminho) {
        return profile.url + caminho;
    }
}
