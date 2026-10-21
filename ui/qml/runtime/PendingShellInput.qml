import QtQuick

// Owns one command waiting for its requested terminal, never the next arbitrary
// session. The JSON-RPC request id comes from CoreClient through the router.
QtObject {
    id: root
    property string command: ""
    property var requestId: -1
    property string errorText: ""
    property int timeoutMs: 15000
    signal dropped(string command)

    readonly property Timer deadline: Timer {
        interval: root.timeoutMs
        repeat: false
        onTriggered: root.giveUp(qsTr("O terminal não abriu a tempo. O comando não foi enviado; tente novamente."))
    }

    function hold(text) {
        if (text === undefined || text === null || text.trim() === "") return false;
        if (command !== "") {
            errorText = qsTr("Outro comando aguarda a abertura do terminal. Este comando não foi enviado; tente novamente após a abertura.");
            dropped(text);
            return false;
        }
        command = text;
        requestId = -1;
        errorText = "";
        deadline.restart();
        return true;
    }

    function bindRequest(id) {
        if (command === "") return;
        if (id === undefined || id === null || id < 0) {
            giveUp(qsTr("O serviço da IDE está indisponível. O comando não foi enviado; aguarde a reconexão e tente novamente."));
            return;
        }
        requestId = id;
    }

    function matches(id) {
        return command !== "" && requestId >= 0 && id === requestId;
    }

    function release() {
        const held = command;
        command = "";
        requestId = -1;
        deadline.stop();
        return held;
    }

    function take(id) { return matches(id) ? release() : ""; }

    function fail(id, message) {
        if (matches(id)) giveUp(message);
    }

    function giveUp(message) {
        if (command === "") return;
        errorText = message || qsTr("O comando não foi enviado ao terminal. Tente novamente.");
        dropped(release());
    }

    function cancel(message, resetError) {
        if (resetError) errorText = "";
        giveUp(message);
    }
}
