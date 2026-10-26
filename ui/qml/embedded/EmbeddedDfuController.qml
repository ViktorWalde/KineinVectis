import QtQuick

// USB descriptors remain discovery; no selection here can launch firmware.
Item {
    id: root
    visible: false

    property var interfaces: []
    property bool busy: false
    property bool measured: false
    property bool complete: false
    property bool toolAvailable: true
    property string tool: ""
    property string rawOutput: ""
    property string hint: ""
    property string errorText: ""
    property double requestId: -1
    readonly property string verdict: busy ? qsTr("Consultando USB DFU…")
        : errorText !== "" ? (measured ? qsTr("Consulta USB DFU não verificada") : qsTr("Dispositivos a verificar"))
        : complete ? qsTr("%1 alternativa(s) USB DFU listada(s).").arg(interfaces.length) : ""

    signal listRequested()

    function clear() {
        requestId = -1;
        busy = false;
        measured = false;
        complete = false;
        toolAvailable = true;
        interfaces = [];
        tool = "";
        rawOutput = "";
        hint = "";
        errorText = "";
    }

    function refresh() {
        clear();
        busy = true;
        listRequested();
    }

    function bindRequest(id) {
        requestId = id;
        if (id >= 0) return;
        busy = false;
        errorText = qsTr("Não foi possível solicitar a consulta USB DFU. Aguarde a conexão e tente novamente.");
    }

    function handleResult(result, id) {
        if (!busy || requestId < 0 || requestId !== id) return;
        requestId = -1;
        busy = false;
        measured = true;
        complete = result.complete === true;
        toolAvailable = result.toolAvailable === true;
        interfaces = complete ? (result.interfaces || []) : [];
        tool = result.tool || "";
        rawOutput = result.rawOutput || "";
        hint = result.hint || "";
        errorText = complete ? "" : hint || qsTr("A consulta USB DFU não foi verificada. Confira a saída e tente novamente.");
    }

    function handleFailed(id, message) {
        if (!busy || requestId < 0 || requestId !== id) return;
        clear();
        errorText = message;
    }

    function disconnected() {
        clear();
        errorText = qsTr("A conexão com o serviço da IDE foi interrompida. Consulte USB DFU novamente após reconectar.");
    }

    function isDfu(entry) { return !!entry && entry.mode === "dfu"; }

    function summary(entry) {
        const mode = isDfu(entry) ? qsTr("DFU") : qsTr("Runtime");
        return qsTr("%1 · %2:%3 · configuração %4 · interface %5 · alternativa %6")
            .arg(mode).arg(entry.vid).arg(entry.pid).arg(entry.configuration).arg(entry.interface).arg(entry.alternate);
    }
}
