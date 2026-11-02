pragma ComponentBehavior: Bound
import QtQuick

// Local tool catalogue and query state; commands require an explicit user action.
Item {
    id: root

    property bool panelVisible: false
    property string distroName: ""
    property string family: ""
    property var tools: []
    property string errorText: ""
    property bool busy: false
    property string query: ""
    property string availability: "all"
    readonly property var filteredTools: root.tools.filter(tool => {
        const needle = root.query.trim().toLocaleLowerCase();
        return (root.availability === "all" || tool.installed === (root.availability === "installed"))
            && (needle === "" || (tool.id + " " + tool.name + " " + tool.summary).toLocaleLowerCase().indexOf(needle) >= 0);
    })

    // Ferramenta cujo passo a passo esta aberto. Uma so' por vez: a lista
    // inteira aberta viraria uma parede de comando.
    property string expandedId: ""

    signal listRequested()
    signal commandRequested(string command)
    signal windowRequested()

    visible: false

    function open() {
        windowRequested();
    }

    function prepare() {
        panelVisible = true;
        refresh();
    }

    function refresh() {
        if (busy) return;
        busy = true;
        errorText = "";
        tools = [];
        listRequested();
    }

    function close() {
        panelVisible = false;
    }

    function toggle(id) {
        expandedId = expandedId === id ? "" : id;
    }

    function handleList(distro, newFamily, newTools) {
        if (!busy) return;
        busy = false;
        distroName = distro;
        family = newFamily;
        tools = newTools;
        errorText = "";
    }

    function handleFailed(method, message) {
        if (method === "setup.list") {
            busy = false;
            tools = [];
            errorText = message;
        }
    }
}
