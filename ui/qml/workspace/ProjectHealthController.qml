import QtQuick

Item {
    id: root

    property string workspaceRoot: ""
    property string workspaceKind: ""
    property var workspaceBuildSystems: []
    property var toolsList: []
    property bool scanningEnvironment: false
    property bool cmakeStatusKnown: false
    property bool cmakeConfigured: false
    // Auto-setup ao abrir (radar de docs-privada/diario/18): o configure dispara sozinho
    // UMA vez por workspace; falha devolve o aviso acionavel (sem loop).
    property bool autoConfigureAttempted: false
    property bool autoConfigureFailed: false
    property bool cargoMetadataFailed: false
    property string cargoMetadataError: ""

    property bool active: false
    property string status: "idle"
    property string message: ""
    property string actionLabel: ""
    property string actionTarget: ""
    property string dismissedKey: ""

    signal autoConfigureRequested()

    visible: false

    onWorkspaceRootChanged: {
        dismissedKey = "";
        cmakeStatusKnown = false;
        cmakeConfigured = false;
        autoConfigureAttempted = false;
        autoConfigureFailed = false;
        cargoMetadataFailed = false;
        cargoMetadataError = "";
        update();
    }
    onWorkspaceKindChanged: update()
    onWorkspaceBuildSystemsChanged: update()
    onToolsListChanged: update()
    onScanningEnvironmentChanged: update()
    Component.onCompleted: update()

    function handleCmakeStatus(configured) {
        cmakeStatusKnown = true;
        cmakeConfigured = configured;
        if (hasBuildSystem("cmake") && !configured
                && !autoConfigureAttempted) {
            autoConfigureAttempted = true;
            autoConfigureRequested();
        }
        update();
    }

    function handleCmakeFinished(success) {
        if (!success && autoConfigureAttempted) {
            autoConfigureFailed = true;
        }
        update();
    }

    function handleCargoMetadataResolved() {
        cargoMetadataFailed = false;
        cargoMetadataError = "";
        update();
    }

    function handleCargoMetadataFailed(message) {
        cargoMetadataFailed = true;
        cargoMetadataError = message;
        update();
    }

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

    function hasBuildSystem(buildSystem) {
        const systems = workspaceBuildSystems !== undefined
                && workspaceBuildSystems !== null
                ? workspaceBuildSystems : [];
        return systems.indexOf(buildSystem) >= 0;
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

    function requiredWorkspaceToolGroups() {
        const groups = [];
        const systems = workspaceBuildSystems !== undefined
                && workspaceBuildSystems !== null
                ? workspaceBuildSystems : [];
        for (let index = 0; index < systems.length; index++) {
            let systemGroups = [];
            if (systems[index] === "cargo") {
                systemGroups = requiredToolGroups("rustCargo");
            } else if (systems[index] === "cmake") {
                systemGroups = requiredToolGroups("cmake");
            }
            for (let groupIndex = 0; groupIndex < systemGroups.length;
                 groupIndex++) {
                const key = systemGroups[groupIndex].join("|");
                let duplicate = false;
                for (let existing = 0; existing < groups.length; existing++) {
                    if (groups[existing].join("|") === key) {
                        duplicate = true;
                        break;
                    }
                }
                if (!duplicate) {
                    groups.push(systemGroups[groupIndex]);
                }
            }
        }
        return groups.length > 0 ? groups : requiredToolGroups(workspaceKind);
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
        if (workspaceKind === "unknown"
                && workspaceBuildSystems.length === 0) {
            apply("info",
                  qsTr("tipo de projeto não detectado; build e run indisponíveis"),
                  "", "");
            return;
        }
        const groups = requiredWorkspaceToolGroups();
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
        if (hasBuildSystem("cmake") && cmakeStatusKnown && !cmakeConfigured) {
            if (autoConfigureAttempted && !autoConfigureFailed) {
                apply("info",
                      qsTr("configurando o projeto CMake automaticamente..."),
                      qsTr("Jobs"), "jobs");
                return;
            }
            apply("warning",
                  qsTr("CMake sem configure — análise e run em modo degradado"),
                  qsTr("Configurar"), "cmakeConfigure");
            return;
        }
        if (hasBuildSystem("cargo") && cargoMetadataFailed) {
            apply("warning",
                  qsTr("cargo metadata falhou: %1").arg(cargoMetadataError),
                  qsTr("Tentar de novo"), "cargoMetadata");
            return;
        }
        apply("ok", "", "", "");
    }
}
