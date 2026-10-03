pragma Singleton
import QtQuick
import KineinVectis

// O ESTADO DE UM CONTAINER — dono unico (2026-10-03). O estado vem do MOTOR
// (`state`): "running" roda; "paused" pausou; o resto (exited, created, dead)
// esta' parado. A lista, o detalhe, o controller e as linhas perguntam
// daqui; a tela nao interpreta o `status` humano ("Up 2 hours"), que muda de
// idioma e de forma entre Docker e Podman.
QtObject {
    // "running" | "paused" | "stopped".
    function stateOf(container) {
        const raw = container === null || container === undefined ? "" : container.state;
        return raw === "running" ? "running" : (raw === "paused" ? "paused" : "stopped");
    }

    function isRunning(container) {
        return stateOf(container) === "running";
    }

    // Rodando OU pausado: o que se para (e nao o que se inicia ou remove).
    function isActive(state) {
        return state !== "stopped";
    }

    function color(state) {
        return state === "running" ? Theme.successSoft : (state === "paused" ? Theme.warningSoft : Theme.textMuted);
    }

    // A acao em andamento, como a linha a mostra.
    function pendingLabel(action) {
        return ({ start: qsTr("iniciando…"), stop: qsTr("parando…"), restart: qsTr("reiniciando…"),
                  remove: qsTr("removendo…") })[action] || qsTr("trabalhando…");
    }

    function label(state) {
        return state === "running" ? qsTr("rodando") : (state === "paused" ? qsTr("pausado") : qsTr("parado"));
    }
}
