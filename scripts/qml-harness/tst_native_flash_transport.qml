import QtQuick
import KineinVectis

Item {
    id: root
    property int failures: 0
    property var intent: ({version: 1, workspace: "/project", program: "/bin/esptool",
                           args: ["write-flash", "image.bin"], deviceKey: "channel",
                           fingerprint: "snapshot", selection: {native: true, device: "/port"}})
    function check(condition, message) {
        if (!condition) { failures++; console.error("FALHAS: " + message); }
    }
    // O transporte pode reordenar chaves; o contrato e' o conteudo inteiro.
    function sameValue(actual, expected) {
        // Sequencia Qt tambem serializa como array, sem ser um Array JS.
        return sameJson(JSON.parse(JSON.stringify(actual)), JSON.parse(JSON.stringify(expected)));
    }
    function sameJson(actual, expected) {
        if (actual === expected) return true;
        if (!actual || !expected || typeof actual !== "object" || typeof expected !== "object") return false;
        if (Array.isArray(actual) !== Array.isArray(expected)) return false;
        const keys = Object.keys(expected);
        return Object.keys(actual).length === keys.length
                && keys.every(key => Object.prototype.hasOwnProperty.call(actual, key) && sameJson(actual[key], expected[key]));
    }
    QtObject {
        id: client
        property bool connected: true
        property int sequence: 0
        property var proposalRequest: ({})
        property var saved: ({})
        property var launched: ({})
        function runConfigFlashProposal(device, engine, size, firmware, nativeMode, directory) {
            proposalRequest = {native: nativeMode, directory: directory}; return ++sequence;
        }
        function runConfigSave(id, name, command, flash) {
            saved = {id: id, name: name, command: command, flash: flash}; return ++sequence;
        }
        function runStart(command, device, flash) {
            launched = {command: command, device: device, flash: flash}; return ++sequence;
        }
    }
    EmbeddedController { id: embedded; workspaceRoot: "/project" }
    RunConfigController { id: configs }
    RunConfigDialog { id: editor; nativeConfiguration: configs.editingNative }
    RuntimeController { id: runtime; workspaceRoot: "/project"; serialDevice: "/other-port" }
    EmbeddedRequestRouter { coreClient: client; embeddedController: embedded }
    RuntimeRequestRouter { coreClient: client; runtimeController: runtime; runConfigController: configs }
    Connections {
        target: embedded.flash
        function onRunRequested(command, flash) { runtime.startRun(command, flash); }
        function onSaveRequested(name, command, flash) { configs.saveConfiguration(name, command, flash); }
    }
    Component.onCompleted: {
        const flash = embedded.flash;
        flash.propose("/port", 0);
        check(client.proposalRequest.native === true, "prévia solicita intenção nativa por padrão");
        flash.handleProposal({name: "Flash", command: "reviewed command", flash: intent}, client.sequence);
        flash.run();
        check(sameValue(client.launched.flash, intent), "run transporta intenção inteira");
        check(client.launched.command === "" && client.launched.device === "", "execução nativa não leva shell nem outra porta");
        flash.save();
        check(sameValue(client.saved.flash, intent), "save transporta intenção inteira");
        configs.handleMutationFinished(client.sequence);
        configs.handleRunConfigs([{id:"cfg-1",name:"Flash",command:"reviewed command",flash:intent}],"cfg-1");
        configs.openEditConfigDialog();
        editor.openWith("Flash", "reviewed command");
        check(configs.editingNative && find(editor, "runConfigCommand").readOnly
              && find(editor, "runConfigCommand").cursorPosition === 0,
              "editor real apresenta o início da prévia nativa sem permitir editá-la");
        configs.handleRunConfigs([{id:"cfg-1",name:"Concurrent",command:"other",flash:{version:2}}],"cfg-1");
        configs.confirmConfigDialog("Renamed", "reviewed command");
        check(client.saved.id === "cfg-1" && client.saved.name === "Renamed"
              && sameValue(client.saved.flash, intent), "renomear preserva objeto aninhado sem ListModel");
        configs.handleMutationFinished(client.sequence);
        configs.clear();
        check(!configs.editingNative && Object.keys(configs.flashById).length === 0, "novo projeto descarta intenções anteriores");
        runtime.clear();
        const before = client.sequence;
        flash.selectBuildDirectory("/external/build"); flash.run(); flash.save();
        check(!flash.found && client.sequence === before, "trocar build invalida ações");
        flash.propose("/port", 0);
        check(client.proposalRequest.native && client.proposalRequest.directory === "/external/build", "build explícito chega ao pedido");
        flash.handleProposal({command: "missing native intent"}, client.sequence);
        flash.run(); flash.save();
        check(!flash.found && flash.errorText !== "" && client.sequence === before + 1,
              "resposta sem intenção é recusada sem fallback manual");
        flash.propose("/port", 0); const empty = client.sequence;
        flash.handleProposal({command:"must not run the default launcher",flash:{}},empty);
        flash.run(); flash.save();
        check(!flash.found && client.sequence === empty, "intenção vazia não executa lançador padrão nem salva configuração manual");
        flash.selectMode(false); flash.propose("/port", 0);
        check(client.proposalRequest.native === false && client.proposalRequest.directory === "", "modo manual é escolha explícita sem build nativo");
        flash.handleProposal({command:"echo manual"}, client.sequence);
        flash.run();
        check(client.launched.command === "echo manual" && Object.keys(client.launched.flash).length === 0,
              "manual mantém comando e não inventa intenção");
        flash.selectMode(true);
        check(!flash.found, "trocar modo invalida prévia manual");
        Qt.exit(failures === 0 ? 0 : 1);
    }
    function find(item, name) {
        if (item.objectName === name) return item;
        for (const child of item.children) {
            const result = find(child, name);
            if (result) return result;
        }
        return null;
    }
}
