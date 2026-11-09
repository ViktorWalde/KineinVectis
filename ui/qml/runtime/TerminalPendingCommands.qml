import QtQuick

// INF-02 (62 §3.4): the line the core holds until a session's first prompt,
// by session id. Every change assigns a new map, so bindings that read it
// re-evaluate. PendingShellInput owns the line before the session exists; this owns it
// after, until `event.terminal.command` settles it.
QtObject {
    id: root
    property var commands: ({})
    signal actionRequested(string id, string action)
    signal droppedAtExit(string command)

    function set(id, command) {
        const next = Object.assign({}, commands);
        if (command === "") delete next[id];
        else next[id] = command;
        commands = next;
    }

    function clear() {
        commands = ({});
    }

    function commandFor(id) { return commands[id] || ""; }

    function request(id, action) {
        if (commandFor(id) !== "") actionRequested(id, action);
    }

    function forget(id) {
        if (commandFor(id) !== "") set(id, "");
    }

    /// The wait ended by prompt, request or exit; only exit loses the line.
    function settled(id, state, reason) {
        const command = commandFor(id);
        if (command === "") return;
        set(id, "");
        if (state === "discarded" && reason === "exit") droppedAtExit(command);
    }
}
