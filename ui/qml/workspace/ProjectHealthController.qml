import QtQuick

Item {
    id: root

    property string workspaceRoot: ""
    property string workspaceKind: ""
    property var toolsList: []
    property bool scanningEnvironment: false

    property bool active: false
    property string status: "idle"
    property string message: ""
    property string actionLabel: ""
    property string actionTarget: ""
    property string dismissedKey: ""

    visible: false

    onWorkspaceRootChanged: {
        dismissedKey = "";
        update();
    }
    onWorkspaceKindChanged: update()
    onToolsListChanged: update()
    onScanningEnvironmentChanged: update()
    Component.onCompleted: update()

    function dismiss() {
        dismissedKey = status + "|" + message;
        update();
    }

    function hasTool(toolId) {
        const tools = toolsList !== undefined && toolsList !== null
                ? toolsList : [];
        for (let i = 0; i < tools.length; i++) {
            if (tools[i].id === toolId && tools[i].status === "detected") {
                return true;
            }
        }
        return false;
    }

    function hasAnyTool(toolIds) {
        for (let i = 0; i < toolIds.length; i++) {
            if (hasTool(toolIds[i])) {
                return true;
            }
        }
        return false;
    }

    function requiredToolGroups(kind) {
        if (kind === "rustCargo") {
            return [["cargo"], ["rustc"], ["rust-analyzer"]];
        }
        if (kind === "cmake") {
            return [["cmake"], ["ninja"], ["clangd"], ["clangxx", "gxx"]];
        }
        return [];
    }

    function missingGroups(groups) {
        const missing = [];
        for (let i = 0; i < groups.length; i++) {
            if (!hasAnyTool(groups[i])) {
                missing.push(groups[i].join("/"));
            }
        }
        return missing;
    }

    function apply(newStatus, newMessage, newActionLabel, newActionTarget) {
        status = newStatus;
        message = newMessage;
        actionLabel = newActionLabel;
        actionTarget = newActionTarget;
        active = newStatus !== "idle" && newStatus !== "ok"
                && (newStatus + "|" + newMessage) !== dismissedKey;
    }

    function update() {
        if (workspaceRoot === "") {
            apply("idle", "", "", "");
            return;
        }
        if (scanningEnvironment) {
            apply("busy", qsTr("verificando o ambiente do projeto..."), "", "");
            return;
        }
        if (workspaceKind === "unknown") {
            apply("info",
                  qsTr("tipo de projeto não detectado; build e run indisponíveis"),
                  "", "");
            return;
        }
        const groups = requiredToolGroups(workspaceKind);
        if (groups.length === 0) {
            apply("ok", "", "", "");
            return;
        }
        const hasTools = toolsList !== undefined && toolsList !== null
                && toolsList.length > 0;
        if (!hasTools) {
            apply("info", qsTr("ambiente ainda não verificado para este projeto"),
                  qsTr("Verificar"), "scan");
            return;
        }
        const missing = missingGroups(groups);
        if (missing.length > 0) {
            apply("warning",
                  qsTr("ferramentas ausentes: %1").arg(missing.join(", ")),
                  qsTr("Ferramentas"), "tools");
            return;
        }
        apply("ok", "", "", "");
    }
}
