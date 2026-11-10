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
    // A pessoa escolheu a configuração no menu (C1-23): escolher não
    // configura, nem libera o configure automático que esperava a escolha.
    property bool configurationChosen: false
    // 16.4b (roadmaps/59 §7.1.4): so' com a confianca da pessoa o configure
    // automatico roda o CMake do projeto. Vem do core (fora do projeto).
    property bool workspaceTrusted: false
    property bool cargoMetadataFailed: false
    property string cargoMetadataError: ""
    // A recusa do `cmake.configure` antes do CMake (consolidação C §15: o
    // compilador fixado sumiu). Sai com o próximo configure bem-sucedido.
    property string cmakeConfigureRefusal: ""
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
    // A CDB em uso (0.206.0, consolidação C §16): a pasta, a origem e se a
    // escolha entre várias configurações de presets é obrigatória.
    property string cdbDirectory: ""
    property bool cdbChoiceRequired: false
    property string cdbAbsentBecause: ""
    // A configuração ativa e o menu dela (0.210.0, C1-23): o chip do
    // cabeçalho e a faixa abrem o mesmo menu.
    readonly property CmakeConfigurationChoices configuration: CmakeConfigurationChoices {}
    property var compileContext: ({})
    onCompileContextChanged: update()
    // O que o projeto exige e o que a máquina tem.
    readonly property ProjectToolRequirements requirements: ProjectToolRequirements {}

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
        configurationChosen = false;
        cargoMetadataFailed = false;
        cargoMetadataError = "";
        cmakeConfigureRefusal = "";
        cdbStale = false;
        cdbStaleBecause = "";
        presetChoices = [];
        presetChoicesFile = "";
        cdbDirectory = "";
        cdbChoiceRequired = false;
        cdbAbsentBecause = "";
        configuration.reset();
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

    function handleCmakeStatus(configured, stale, staleBecause, preset, choices, choicesFile,
                               database, activeConfiguration) {
        cmakeStatusKnown = true;
        cmakeConfigured = configured;
        cmakePreset = preset === undefined || preset === null ? "" : preset;
        presetChoices = choices === undefined || choices === null ? [] : choices;
        presetChoicesFile = choicesFile === undefined || choicesFile === null ? "" : choicesFile;
        cdbStale = stale === true;
        cdbStaleBecause = cdbStale && staleBecause !== undefined
                && staleBecause !== null ? staleBecause : "";
        const cdb = database === undefined || database === null ? ({}) : database;
        cdbDirectory = cdb.directory || "";
        cdbChoiceRequired = cdb.choiceRequired === true;
        cdbAbsentBecause = cdb.absentBecause || "";
        configuration.handle(activeConfiguration, cdb.selectedConfiguration);
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
    // confianca, so' o "Configurar" da faixa (um gesto) o roda. Com uma CDB
    // em uso (de um preset, do CLI, de outra IDE) ou uma escolha pendente, ele
    // nao roda: configuraria a pasta da configuracao ativa, que pode nao ser a
    // dessa CDB, e trocaria a base calado (C1-19). Sem configuracao ativa, o
    // core recusaria: a tentativa espera a escolha (C1-23).
    function configureIfTrusted() {
        if (workspaceTrusted && hasBuildSystem("cmake") && cmakeStatusKnown
                && !cmakeConfigured && !autoConfigureAttempted && presetChoices.length === 0
                && cdbDirectory === "" && !cdbChoiceRequired && configuration.refused === ""
                && !configurationChosen) {
            autoConfigureAttempted = true;
            autoConfigureRequested();
        }
    }

    function handleConfigurationChosen() {
        configurationChosen = true;
    }

    function handleCmakeFinished(success) {
        if (!success && autoConfigureAttempted) {
            autoConfigureFailed = true;
        }
        if (success) cmakeConfigureRefusal = "";
        update();
    }

    function handleCmakeConfigureFailed(message) {
        cmakeConfigureRefusal = message;
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

    // O dono de "esta ferramenta foi detectada" para o RailFacts.
    function hasAnyTool(toolIds) {
        return requirements.hasAnyTool(toolsList, toolIds);
    }

    function hasBuildSystem(buildSystem) {
        const systems = workspaceBuildSystems !== undefined
                && workspaceBuildSystems !== null
                ? workspaceBuildSystems : [];
        return systems.indexOf(buildSystem) >= 0;
    }

    // O menu do "Escolher preset…": um item por preset; escolher configura
    // com ele (ShellHeaderHost.executeMenuAction, prefixo "cmake.preset:").
    function presetMenuItems() {
        return presetChoices.map(name => ({ label: name, action: "cmake.preset:" + name,
                                            icon: "", enabled: true, shortcut: "" }));
    }

    // O menu da faixa atual; vazio quando a ação não abre menu. O
    // "Escolher configuração…" é o menu do chip do cabeçalho.
    function menuItems() {
        if (actionTarget === "cmakePresetChoice") return presetMenuItems();
        if (actionTarget === "cmakeConfigurationChoice") return configuration.items();
        return [];
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
        const groups = requirements.requiredWorkspaceToolGroups(workspaceKind, workspaceBuildSystems);
        const hasTools = toolsList !== undefined && toolsList !== null
                && toolsList.length > 0;
        if (groups.length > 0 && !hasTools) {
            apply("info", qsTr("ambiente ainda não verificado para este projeto"),
                  qsTr("Verificar"), "scan");
            return;
        }
        const missing = requirements.missingGroups(toolsList, groups);
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
        if (hasBuildSystem("cmake") && cmakeStatusKnown && cdbChoiceRequired) {
            apply("warning", cdbAbsentBecause, qsTr("Escolher configuração…"),
                  "cmakeConfigurationChoice");
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
        // Sem configuração ativa e sem presets a configurar: o menu tem a
        // volta ao automático (o registro órfão, o ilegível).
        if (hasBuildSystem("cmake") && cmakeStatusKnown && configuration.refused !== "") {
            apply("warning", configuration.refused, qsTr("Escolher configuração…"),
                  "cmakeConfigurationChoice");
            return;
        }
        if (hasBuildSystem("cmake") && cmakeConfigureRefusal !== "") {
            apply("warning", qsTr("configure recusado: %1").arg(cmakeConfigureRefusal),
                  qsTr("Tentar de novo"), "cmakeConfigure");
            return;
        }
        // Com a CDB de outra configuração em uso, a análise não está degradada.
        if (hasBuildSystem("cmake") && cmakeStatusKnown && !cmakeConfigured
                && cdbDirectory === "") {
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
                      ? qsTr("análise desatualizada: %1; reconfigure para atualizar as flags")
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
