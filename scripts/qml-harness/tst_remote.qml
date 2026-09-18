import QtQuick
import "../../ui/qml/remote"

// O alvo Linux por SSH (P6 fatia 1, 2026-09-17): o RemoteController REAL com
// o roteador falso.
//
// O que se prova: abrir o workspace pede a lista; salvar manda so' os campos
// do protocolo (nunca senha); o alvo salvo fica selecionado; sondar/enviar
// so' com alvo SALVO; o evento da sonda vira arquitetura/kernel/ferramentas;
// a falha vira texto com o proximo passo; cada `remote.command` vai ao dono
// certo (run config, kit, terminal); trocar de workspace esquece tudo.
Item {
    id: root

    property var pedidos: []

    RemoteController {
        id: c

        onListRequested: root.pedidos.push("list")
        onSaveRequested: function(target) { root.pedidos.push("save:" + JSON.stringify(target)); }
        onRemoveRequested: function(name) { root.pedidos.push("remove:" + name); }
        onProbeRequested: function(name) { root.pedidos.push("probe:" + name); }
        onDeployRequested: function(name, source, dest) { root.pedidos.push("deploy:" + name + ":" + source + ":" + dest); }
        onCommandRequested: function(name, kind, program, port) { root.pedidos.push("command:" + name + ":" + kind + ":" + program + ":" + port); }
        onRunConfigRequested: function(name, command) { root.pedidos.push("runconfig:" + name + "=" + command); }
        onKitRemoteRequested: function(remoteTarget, debugServer) { root.pedidos.push("kit:" + remoteTarget + "=" + debugServer); }
        onShellRequested: function(command) { root.pedidos.push("shell:" + command); }
    }

    Component.onCompleted: {
        let failures = 0;
        c.workspaceRoot = "/w";
        if (root.pedidos.join(",") !== "list") failures += 1;

        // Sondar/enviar sem alvo salvo: nada sai.
        c.probe(); c.deploy(); c.requestCommand("run");
        if (root.pedidos.length !== 1) failures += 2;

        // Editar e salvar: so' os campos do protocolo; a porta vira numero.
        c.editDraft("name", "pi");
        c.editDraft("host", "192.168.0.42");
        c.editDraft("user", "pi");
        c.editDraft("port", "2222");
        c.editDraft("password", "nunca");
        c.save();
        const salvo = root.pedidos[1];
        if (salvo.indexOf("save:") !== 0 || salvo.indexOf("password") >= 0 || salvo.indexOf('"port":2222') < 0) failures += 4;

        // A resposta seleciona o alvo salvo.
        c.handleTargets([{ name: "pi", host: "192.168.0.42", user: "pi", port: 2222 }]);
        if (c.selectedName !== "pi" || !c.selectedSaved || c.targets.length !== 1) failures += 8;

        // Sondar: o pedido sai, e o evento preenche o veredito.
        c.probe();
        if (!c.probing || root.pedidos[2] !== "probe:pi") failures += 16;
        c.handleJobAccepted("remote.probe", "j1", "ssh -o BatchMode=yes pi@192.168.0.42 uname");
        c.handleProbed({ name: "pi", success: true, arch: "aarch64", kernel: "Linux 6.6", tools: [{ id: "gdbserver", found: true }, { id: "rsync", found: false }] });
        if (c.probing || !c.probeOk || c.probeArch !== "aarch64" || c.probeTools.length !== 2 || c.lastCommand.indexOf("ssh") !== 0) failures += 32;
        c.handleProbed({ name: "pi", success: false, error: "o alvo recusou a chave: ssh-copy-id pi@192.168.0.42" });
        if (c.probeOk || c.probeMessage.indexOf("ssh-copy-id") < 0) failures += 64;

        // Enviar com a origem digitada; o desfecho vira texto.
        c.deploySource = "build/app";
        c.deploy();
        if (!c.deploying || root.pedidos[3] !== "deploy:pi:build/app:") failures += 128;
        c.handleDeployed({ name: "pi", success: true, dest: "~/kinein/w" });
        if (c.deploying || c.deployMessage.indexOf("~/kinein/w") < 0) failures += 256;

        // Cada comando vai ao dono certo.
        c.program = "app";
        c.requestCommand("run");
        if (root.pedidos[4] !== "command:pi:run:app:0") failures += 512;
        c.handleCommand({ name: "Rodar em pi", command: "ssh -tt pi@192.168.0.42 '~/kinein/w/app'" });
        if (root.pedidos[5] !== "runconfig:Rodar em pi=ssh -tt pi@192.168.0.42 '~/kinein/w/app'") failures += 1024;
        c.requestCommand("debugServer");
        c.handleCommand({ name: "gdbserver em pi", command: "ssh -tt pi@192.168.0.42 'gdbserver :2345 ~/kinein/w/app'", remoteTarget: "192.168.0.42:2345" });
        if (root.pedidos[7] !== "kit:192.168.0.42:2345=ssh -tt pi@192.168.0.42 'gdbserver :2345 ~/kinein/w/app'") failures += 2048;
        c.requestCommand("shell");
        c.handleCommand({ name: "Shell em pi", command: "ssh pi@192.168.0.42" });
        if (root.pedidos[9] !== "shell:ssh pi@192.168.0.42" || c.lastOutcome === "" || c.pendingKind !== "") failures += 4096;

        // A falha de um pedido remote.* vira texto e destrava.
        c.probe();
        c.handleFailed("remote.probe", "nao achei `ssh` no PATH");
        if (c.probing || c.errorText.indexOf("ssh") < 0) failures += 8192;
        c.handleFailed("build.run", "outra coisa");
        if (c.errorText !== "nao achei `ssh` no PATH") failures += 16384;

        // Remover; trocar de workspace esquece tudo.
        c.remove();
        if (root.pedidos[root.pedidos.length - 1] !== "remove:pi") failures += 32768;
        c.handleTargets([]);
        if (c.selectedName !== "" || c.selectedSaved) failures += 65536;
        c.workspaceRoot = "/outro";
        if (c.targets.length !== 0 || c.lastCommand !== "" || c.errorText !== "") failures += 131072;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
