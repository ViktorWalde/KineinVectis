import QtQuick

Item {
    id: root

    property string workspaceRoot: ""
    property string workspaceKind: ""
    property var workspaceBuildSystems: []
    // Reconhecimento do modelo do core, independente de haver build system.
    property bool projectRecognized: false
    property var toolsList: []
    property bool scanningEnvironment: false
    property bool cmakeStatusKnown: false
    property bool cmakeConfigured: false
    // O preset com que a IDE configurou (P0, 0.115.0): o kit, o
    // CMakeUserPresets ou o CMakePresets do projeto; vazio = sem preset.
    property string cmakePreset: ""
    // Auto-setup ao abrir: o configure dispara sozinho
    // UMA vez por workspace; falha devolve o aviso acionavel (sem loop).
    property bool autoConfigureAttempted: false
    property bool autoConfigureFailed: false
    // 16.4b (roadmaps/59 §7.1.4): so' com a confianca da pessoa o configure
    // automatico roda o CMake do projeto. Vem do core (fora do projeto).
    property bool workspaceTrusted: false
    property bool cargoMetadataFailed: false
    property string cargoMetadataError: ""
    // O ambiente Python (bloco B do roadmaps/41): o PythonController diz se
    // falta ambiente e qual e' o botao; este controller so' escolhe a faixa.
    property bool pythonNeedsEnvironment: false
    property bool pythonCreating: false
    property string pythonMessage: ""
    property string pythonActionLabel: ""
    // Os stubs da placa (C4): faixa informativa num projeto MicroPython sem
    // typings/ — o botao instala pelo core.
    property bool pythonNeedsStubs: false
    property bool pythonInstallingStubs: false
    property string pythonStubsMessage: ""
    // §5c do roadmap 29: a CDB alcancavel pelo clangd e' mais velha que um
    // arquivo de build que a define, entao o clangd esta usando flags de um
    // projeto que mudou. O core mede (cdb.rs) e reporta no `cmake.status`;
    // aqui isso vira o mesmo aviso acionavel que o auto-configure ja tem.
    property bool cdbStale: false
    property string cdbStaleBecause: ""
    // Os presets entre os quais a pessoa escolhe (0.4.0b, roadmaps/52 §14.2):
    // com varios e nada anotado, o core recusa o configure sem preset, e a
    // faixa pergunta em vez de configurar sozinha.
    property var presetChoices: []
    property string presetChoicesFile: ""
    property var compileContext: ({})
    onCompileContextChanged: update()

    property bool active: false
    property string status: "idle"
    property string message: ""
    property string actionLabel: ""
    property string actionTarget: ""
    property string dismissedKey: ""
    property string noticeDetail: ""

    signal autoConfigureRequested()

    visible: false

    onWorkspaceRootChanged: {
        dismissedKey = "";
        cmakeStatusKnown = false;
        cmakeConfigured = false;
        cmakePreset = "";
        autoConfigureAttempted = false;
        autoConfigureFailed = false;
        cargoMetadataFailed = false;
        cargoMetadataError = "";
        cdbStale = false;
        cdbStaleBecause = "";
        presetChoices = [];
        presetChoicesFile = "";
        update();
    }
    onWorkspaceKindChanged: update()
    onWorkspaceBuildSystemsChanged: update()
    onProjectRecognizedChanged: update()
    onToolsListChanged: update()
    onScanningEnvironmentChanged: update()
    onPythonNeedsEnvironmentChanged: update()
    onPythonCreatingChanged: update()
    onPythonMessageChanged: update()
    onPythonNeedsStubsChanged: update()
    onPythonInstallingStubsChanged: update()
    onPythonStubsMessageChanged: update()
    Component.onCompleted: update()

    function handleCmakeStatus(configured, stale, staleBecause, preset, choices, choicesFile) {
        cmakeStatusKnown = true;
        cmakeConfigured = configured;
        cmakePreset = preset === undefined || preset === null ? "" : preset;
        presetChoices = choices === undefined || choices === null ? [] : choices;
        presetChoicesFile = choicesFile === undefined || choicesFile === null ? "" : choicesFile;
        cdbStale = stale === true;
        cdbStaleBecause = cdbStale && staleBecause !== undefined
                && staleBecause !== null ? staleBecause : "";
        configureIfTrusted();
        update();
    }

    // A confianca chegou (ou saiu): o configure que esperava por ela roda.
    function handleTrust(trusted) {
        workspaceTrusted = trusted;
        configureIfTrusted();
        update();
    }

    // O configure automatico executa o CMakeLists.txt do projeto: sem a
    // confianca, so' o "Configurar" da faixa (um gesto) o roda.
    function configureIfTrusted() {
        if (workspaceTrusted && hasBuildSystem("cmake") && cmakeStatusKnown
                && !cmakeConfigured && !autoConfigureAttempted && presetChoices.length === 0) {
            autoConfigureAttempted = true;
            autoConfigureRequested();
        }
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
        dismissedKey = noticeKey(status, message, noticeDetail);
        update();
    }

    function noticeKey(status, message, detail) {
        return status + "|" + message + "|" + detail;
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
        // Makefile puro (P0, 2026-09-17): o bear entra como ferramenta do
        // projeto porque sem ele nao ha' compile_commands.json — e a faixa
        // diz isso antes do primeiro build mudo.
        if (kind === "make") {
            return [["make", "gmake"], ["clangd"], ["clangxx", "gxx"], ["bear"]];
        }
        // PlatformIO (0.4.0e): o `pio` constroi e gera a CDB; o clangd a le'.
        if (kind === "platformIo") {
            return [["pio"], ["clangd"]];
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
            } else if (systems[index] === "cmake" || systems[index] === "platformIo") {
                systemGroups = requiredToolGroups(systems[index]);
            } else if (systems[index] === "make" && systems.indexOf("cmake") < 0) {
                // Um Makefile ao lado de um CMakeLists e' do CMake: o bear so'
                // e' exigido quando o make e' o unico build system C/C++.
                systemGroups = requiredToolGroups("make");
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

    // O menu do "Escolher preset…": um item por preset; escolher configura
    // com ele (ShellHeaderHost.executeMenuAction, prefixo "cmake.preset:").
    function presetMenuItems() {
        return presetChoices.map(name => ({ label: name, action: "cmake.preset:" + name,
                                            icon: "", enabled: true, shortcut: "" }));
    }

    function apply(newStatus, newMessage, newActionLabel, newActionTarget, detail) {
        status = newStatus;
        message = newMessage;
        actionLabel = newActionLabel;
        actionTarget = newActionTarget;
        noticeDetail = detail || "";
        active = newStatus !== "idle" && newStatus !== "ok"
                && noticeKey(newStatus, newMessage, noticeDetail) !== dismissedKey;
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
        const groups = requiredWorkspaceToolGroups();
        const hasTools = toolsList !== undefined && toolsList !== null
                && toolsList.length > 0;
        if (groups.length > 0 && !hasTools) {
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
        const heldBack = hasBuildSystem("cargo") || hasBuildSystem("platformIo")
                || (hasBuildSystem("cmake") && cmakeStatusKnown && !cmakeConfigured);
        if (!workspaceTrusted && heldBack) {
            apply("info",
                  hasBuildSystem("platformIo")
                      ? qsTr("projeto ainda não confiável: o pio, que roda os scripts do projeto, espera a sua confiança")
                      : qsTr("projeto ainda não confiável: o CMake e os build scripts não rodam sozinhos"),
                  qsTr("Confiar neste projeto"), "trust");
            return;
        }
        if (hasBuildSystem("cmake") && cmakeStatusKnown && presetChoices.length > 0
                && (!cmakeConfigured || cdbStale)) {
            apply("warning",
                  qsTr("o projeto tem %1 presets de configure (%2): escolha um")
                      .arg(presetChoices.length).arg(presetChoicesFile),
                  qsTr("Escolher preset…"), "cmakePresetChoice");
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
        if (cdbStale) {
            apply("warning",
                  cdbStaleBecause !== ""
                      ? qsTr("análise desatualizada: %1 mudou depois do último configure")
                            .arg(cdbStaleBecause)
                      : qsTr("análise desatualizada: reconfigure para atualizar as flags"),
                  qsTr("Reconfigurar"), "cmakeConfigure");
            return;
        }
        if (hasBuildSystem("cargo") && cargoMetadataFailed) {
            apply("warning",
                  qsTr("cargo metadata falhou: %1").arg(cargoMetadataError),
                  qsTr("Tentar de novo"), "cargoMetadata");
            return;
        }
        if ((compileContext.drift || []).length > 0) {
            const path = compileContext.path || "";
            const name = path.split(/[\\/]/).pop();
            apply("warning", qsTr("A configuração do kit difere do build em %1").arg(name),
                  qsTr("Inspecionar"), "compileContext", JSON.stringify([path, compileContext.drift]));
            return;
        }
        if (pythonNeedsEnvironment) {
            if (pythonCreating) {
                apply("info", pythonMessage, qsTr("Jobs"), "jobs");
                return;
            }
            apply("warning", pythonMessage, pythonActionLabel, "pythonEnvironment");
            return;
        }
        if (pythonNeedsStubs) {
            if (pythonInstallingStubs) {
                apply("info", pythonStubsMessage, qsTr("Jobs"), "jobs");
                return;
            }
            apply("info", pythonStubsMessage, qsTr("Instalar stubs"), "pythonStubs");
            return;
        }
        if (workspaceKind === "unknown" && workspaceBuildSystems.length === 0
                && !projectRecognized) {
            apply("info", qsTr("sistema de build não detectado; scripts e comandos manuais continuam disponíveis"), "", "");
            return;
        }
        apply("ok", "", "", "");
    }
}
