import QtQuick

// Correlated discovery lifecycle shared by DFU and Pico; domain labels remain local.
Item {
    id: root
    visible: false

    property var entries: []
    property string entriesKey: ""
    property string queryingText: ""
    property string unverifiedText: ""
    property string completeText: ""
    property string incompleteText: ""
    property bool busy: false
    property bool measured: false
    property bool complete: false
    property bool toolAvailable: true
    property string tool: ""
    property string rawOutput: ""
    property string hint: ""
    property string errorText: ""
    property double requestId: -1
    readonly property string verdict: busy ? queryingText
        : errorText !== "" ? (measured ? unverifiedText : qsTr("Dispositivos a verificar"))
        : complete ? completeText : ""

    signal listRequested()

    function clear() {
        requestId = -1;
        busy = false;
        measured = false;
        complete = false;
        toolAvailable = true;
        entries = [];
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
        errorText = qsTr("Não foi possível solicitar a consulta USB. Aguarde a conexão e tente novamente.");
    }

    function handleResult(result, id) {
        if (!busy || requestId < 0 || requestId !== id) return;
        requestId = -1;
        busy = false;
        measured = true;
        complete = result.complete === true;
        toolAvailable = result.toolAvailable === true;
        entries = complete ? (result[entriesKey] || []) : [];
        tool = result.tool || "";
        rawOutput = result.rawOutput || "";
        hint = result.hint || "";
        errorText = complete ? "" : hint || incompleteText;
    }

    function handleFailed(id, message) {
        if (!busy || requestId < 0 || requestId !== id) return;
        clear();
        errorText = message;
    }

    function disconnected() {
        clear();
        errorText = qsTr("A conexão com o serviço da IDE foi interrompida. Consulte USB novamente após reconectar.");
    }

}
