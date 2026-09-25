pragma ComponentBehavior: Bound
import QtQuick

// O WORKSPACE ESPELHADO: a pasta do alvo que a IDE abre como espelho local, e
// as sincronias entre os dois lados.
//
// Nasceu em 2026-09-24 como o roadmap 48 §8.3 previu — "se a maquina de estados
// crescer, os filhos possiveis sao RemoteSetupController,
// RemoteWorkspaceController, ... Eles nao nascem preventivamente". A catraca do
// `RemoteController` mandou, e este e' o corte que ela pedia.
//
// O corte e' por PERGUNTA: aqui tudo responde "onde estou trabalhando e o que
// ja' foi de um lado para o outro". O catalogo, a sonda e os comandos ficam na
// fachada, porque respondem outras.
//
// Nao fala com o CoreClient: pede por sinal, recebe do roteador.
Item {
    id: root

    // O alvo com que Puxar/Empurrar falam — vem da fachada, que e' dona dele.
    property string targetName: ""
    property bool targetReady: false

    property string openPath: ""
    property var mirror: null
    // O espelho cujo pull ainda esta' em curso: quando ele terminar, a IDE abre
    // a pasta. Guardar o caminho e' o que distingue "este pull" de outro.
    property string pendingMirror: ""
    property bool syncing: false
    property string syncMessage: ""
    // O HUD da V4 precisa distinguir "nao sincronizou ainda" de "tentou e
    // falhou": a §4 proibe confundir save local que deu certo com push remoto
    // que falhou. Texto nao serve para isso — booleano serve.
    property bool syncFailed: false
    property string syncDirection: ""

    readonly property bool isMirror: mirror !== null && mirror !== undefined
                                     && mirror.name !== undefined

    signal openRequested(string name, string path)
    signal syncRequested(string direction, var paths)
    signal workspaceOpenRequested(string path)
    // O espelho diz de qual alvo ele e'; quem seleciona e' a fachada.
    signal selectRequested(string name)
    signal commandComposed(string command)

    visible: false

    // O espelho que o workspace aberto e' (vem do workspace.open / status).
    // Vazio = comum. Num espelho, o alvo dele fica selecionado assim que a
    // lista chegar — e' o alvo com que Puxar/Empurrar falam.
    function handleMirror(map) {
        mirror = (map && map.name) ? map : null;
        if (mirror !== null) {
            selectRequested(mirror.name);
        }
    }

    function openFolder() {
        if (!targetReady || openPath.trim() === "") {
            return;
        }
        syncing = true;
        syncMessage = "";
        openRequested(targetName, openPath.trim());
    }

    function handleOpenAccepted(jobId, command, mirrorPath) {
        pendingMirror = mirrorPath;
        commandComposed(command);
    }

    function sync(direction) {
        if (!isMirror) {
            return;
        }
        syncing = true;
        syncDirection = direction;
        syncFailed = false;
        syncMessage = "";
        syncRequested(direction, []);
    }

    // O pull do `remote.open` abre o espelho; os demais so' contam.
    function handleSynced(outcome) {
        syncing = false;
        syncDirection = outcome.direction || "";
        syncFailed = outcome.success !== true;
        const n = (outcome.changed || []).length;
        if (outcome.success === true) {
            syncMessage = outcome.direction === "pull"
                ? qsTr("puxado de %1: %2 caminho(s)").arg(outcome.name).arg(n)
                : qsTr("empurrado para %1: %2 caminho(s)").arg(outcome.name).arg(n);
            if (pendingMirror !== "" && outcome.mirror === pendingMirror
                    && outcome.direction === "pull") {
                pendingMirror = "";
                workspaceOpenRequested(outcome.mirror);
            }
        } else {
            syncMessage = qsTr("sincronia falhou: %1").arg(outcome.error || "");
            pendingMirror = "";
        }
    }

    // Qualquer falha de `remote.*` interrompe o que estava em curso aqui.
    function handleFailed() {
        syncing = false;
        pendingMirror = "";
    }

    function reset() {
        syncing = false;
        syncMessage = "";
        syncFailed = false;
        syncDirection = "";
    }
}
