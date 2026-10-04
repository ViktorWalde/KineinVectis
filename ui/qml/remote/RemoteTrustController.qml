pragma ComponentBehavior: Bound
import QtQuick

// CONFIAR NO SERVIDOR na primeira conexao (0.153.0, 2026-10-04): filho do
// RemoteController, como o setup e o workspace (a catraca do controller).
//
// A sonda parou no host key ("unknownHost"): a IDE le' a impressao digital do
// servidor (`remote.hostKey`, sem logar) e mostra; a pessoa confere e confia
// (`remote.trustHost`), e o core grava EXATAMENTE a chave vista no
// `known_hosts`. Antes isso era `yes` digitado no terminal, no meio do
// `ssh-copy-id` — relato do autor: o Remoto visual tem de ser pratico.
//
// Nao fala com o CoreClient: pede por sinal, recebe do roteador.
Item {
    id: root

    // De qual alvo e' o host key mostrado (resposta atrasada de outro e'
    // descartada).
    property string name: ""
    // Como o known_hosts o nomeia: `host` ou `[host]:porta`.
    property string host: ""
    // [{ kind, fingerprint }], a mais forte primeiro.
    property var keys: []
    property bool loading: false
    property bool trusting: false
    property string errorText: ""
    // O arquivo onde a ultima confianca foi gravada (para dizer onde).
    property string recordedIn: ""

    readonly property bool showing: root.name !== "" && (root.loading || root.keys.length > 0 || root.errorText !== "")

    signal hostKeyRequested(string name)
    signal trustRequested(string name, var fingerprints)
    signal trusted(string name)

    visible: false

    function request(target) {
        root.name = target;
        root.keys = [];
        root.errorText = "";
        root.loading = true;
        root.hostKeyRequested(target);
    }

    // DONO UNICO de "a sonda parou na primeira conexao" (as regras, o cartao e
    // o ponto perguntam aqui).
    function isFirstContact(failure) {
        return failure === "unknownHost";
    }

    // A sonda terminou: parou no host key? Mostra a impressao; senao, limpa.
    function follow(failure, target) {
        if (root.isFirstContact(failure)) root.request(target);
        else root.clear();
    }

    function clear() {
        root.name = "";
        root.host = "";
        root.keys = [];
        root.loading = false;
        root.trusting = false;
        root.errorText = "";
    }

    function handleHostKey(result) {
        if (result.name !== root.name) return;
        root.loading = false;
        root.host = result.host || "";
        root.keys = result.keys || [];
    }

    // Confia na MAIS FORTE (a primeira): e' a que o ssh vai usar.
    function trust() {
        if (root.keys.length === 0 || root.trusting) return;
        root.trusting = true;
        root.errorText = "";
        root.trustRequested(root.name, [root.keys[0].fingerprint]);
    }

    function handleTrusted(result) {
        if (result.name !== root.name) return;
        const target = root.name;
        root.recordedIn = result.file || "";
        root.clear();
        root.trusted(target);
    }

    function handleFailed(message) {
        root.loading = false;
        root.trusting = false;
        root.errorText = message;
    }
}
