import QtQuick

// Uma aba lógica por execução; PTY/grid novos a cada tentativa. A identidade
// vem do core e não depende do título abreviado nem do intérprete escolhido.
Item {
    id: root
    property var runtime: null
    property bool pending: false
    property string pendingName: ""
    property string terminalId: ""
    property var tabs: ({})
    property var outcomes: ({})
    property var closingTabs: ({})
    property var accepted: ({})
    property int generation: 0
    visible: false

    function clear() {
        pending = false;
        pendingName = "";
        terminalId = "";
        tabs = ({});
        outcomes = ({});
        closingTabs = ({});
        accepted = ({});
        generation++;
    }

    function begin(name) {
        pending = true;
        pendingName = name;
    }

    function disconnected() {
        pending = false;
        pendingName = "";
        generation++;
    }

    function title(command) {
        const parts = command.trim().split(/\s+/);
        let base = parts.length > 0 ? parts[0] : command;
        if (base.indexOf("/") >= 0) {
            base = base.substring(base.lastIndexOf("/") + 1).replace(/'$/, "");
        }
        const rest = parts.slice(1).join(" ");
        const label = rest === "" ? base : base + " " + rest;
        return "▶ " + (label.length > 28 ? label.substring(0, 27) + "…" : label);
    }

    function started(command, id, key, workspace) {
        if (!id || !key || workspace !== runtime.workspaceRoot) return;
        if (runtime.indexOfTerminal(id) >= 0) {
            if (accepted[id] === generation) return;
            if (!isFinished(id)) return;
            runtime.removeTerminalTab(id);
        }
        pending = false;
        const label = pendingName !== "" ? "▶ " + pendingName : title(command);
        pendingName = "";
        const previous = tabs[key];
        const index = previous ? runtime.indexOfTerminal(previous) : -1;
        if (index >= 0) {
            runtime.removeTerminalTab(previous);
        }
        runtime.terminalsModel.insert(index >= 0 ? index : runtime.terminalsModel.count,
                                      {termId: id, title: label});
        runtime.terminalsRevision++;
        runtime.terminalRenders[id] = ({});
        runtime.selectTerminal(id);
        tabs[key] = id;
        accepted[id] = generation;
        terminalId = id;
    }

    function isFinished(id) { return outcomes[id] !== undefined; }

    function closing(id) { closingTabs[id] = true; }

    function finished(id, code) {
        if (closingTabs[id]) return false;
        if (isFinished(id)) return true;
        if (id !== terminalId) return false;
        const index = runtime.indexOfTerminal(id);
        if (index < 0) return false;
        const outcome = code === undefined ? -1 : code;
        outcomes[id] = outcome;
        const label = runtime.terminalsModel.get(index).title;
        runtime.terminalsModel.setProperty(index, "title",
            label + (outcome === 0 ? " ✓" : " ✗ " + outcome));
        runtime.terminalsRevision++;
        return true;
    }

    function release(id) {
        delete outcomes[id];
        delete closingTabs[id];
        delete accepted[id];
        for (const key of Object.keys(tabs)) {
            if (tabs[key] === id) delete tabs[key];
        }
        if (terminalId === id) terminalId = "";
    }

    function failed(method) {
        if (method === "run.start" || method === "run.script") {
            pending = false;
            pendingName = "";
        }
        if (method === "terminal.close") closingTabs = ({});
    }
}
