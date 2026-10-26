import QtQuick
import KineinVectis

// probe-rs nativo na UI (0.4.1h4b2b4b2, roadmaps/52 §14.35): controllers e
// roteadores reais; o cliente falso registra o que iria ao core.
Item {
    id: root
    property int failures: 0
    function check(ok, message) {
        if (!ok) { failures++; console.error("FALHAS: " + message); }
    }
    width: 420
    height: 800
    QtObject {
        id: client
        property bool connected: true
        property int sequence: 0
        property int targetCalls: 0
        property var lastProposal: []
        signal statusChanged()
        signal workspaceChanged()
        signal dfuListResolved(var result, double requestId)
        signal dfuListFailed(double requestId, string message)
        signal probesResolved(var probes, bool toolAvailable, string rawOutput, string hint)
        signal buildSizeResolved(var sections, var regions, bool toolAvailable, string tool, string rawOutput)
        signal projectModelResolved(var model)
        signal projectChanged(var model)
        signal cmakeTargetsResolved(var targets, string origin)
        signal serialPortsResolved(var ports, string hint)
        signal serialMonitorOpened(string id, string command, string tool)
        signal serialIdentifyStarted(string jobId, string command)
        signal serialIdentified(var outcome)
        signal flashProposalResolved(var proposal, double requestId)
        signal flashProposalFailed(double requestId, string message)
        signal serialAccessResolved(var channels)
        signal serialFilesStarted(string jobId, string command)
        signal serialFilesResolved(var outcome)
        signal requestFailed(string method, string message, string code)
        function runConfigFlashProposal(device, engine, size, firmware, nativeMode, buildDirectory, target, probe) {
            lastProposal = [device, engine, size, firmware, nativeMode, buildDirectory, target, probe];
            return ++sequence;
        }
        function cmakeTargetsList() { targetCalls++; }
        function probeList() {}
        function serialList() {}
        function projectModel() {}
    }
    EmbeddedController { id: controller; workspaceRoot: "/ws" }
    EmbeddedRequestRouter { coreClient: client; embeddedController: controller }
    EmbeddedEventRouter { coreClient: client; embeddedController: controller }
    EmbeddedFlashDestination { id: destination; controller: controller; width: 400 }
    EmbeddedOverviewView { id: overview; controller: controller; width: 400 }

    readonly property var bareMetal: ({ embedded: true, frameworks: [], artifacts: { elf: ["/ws/.kinein/build/firmware.elf"] },
        target: { chip: "STM32F411RETx", family: "stm32", flashEngine: "probe-rs" } })
    readonly property var executables: [
        { name: "firmware", kind: "executable", sourceDir: "/ws", artifacts: ["/ws/.kinein/build/firmware.elf"] },
        { name: "bootloader", kind: "executable", sourceDir: "/ws/boot", artifacts: ["/ws/.kinein/build/boot/bootloader.elf"] },
        { name: "support", kind: "staticLibrary", sourceDir: "/ws", artifacts: ["/ws/.kinein/build/libsupport.a"] },
        { name: "foreign", kind: "executable", sourceDir: "/wsx", artifacts: ["/wsx/a.elf"] }]
    readonly property var listed: [
        { name: "STLink V2-1", vid: "0483", pid: "374b", serial: "066D", kind: "ST-LINK" },
        { name: "CMSIS-DAP", vid: "2e8a", pid: "000c", serial: "E661", kind: "CMSIS-DAP", inaccessible: true },
        { name: "Sem serial", vid: "1366", pid: "0101", kind: "J-Link" },
        { name: "Dual RS232", vid: "0403", pid: "6010", interface: 1, serial: "FT1", kind: "FTDI" }]

    Component.onCompleted: {
        client.statusChanged();
        const flash = controller.flash;
        check(!flash.probeDestination && !destination.visible, "sem modelo não há destino de sonda");
        client.projectModelResolved(root.bareMetal);
        check(flash.suggestedEngine === "probe-rs" && flash.probeDestination, "chip do kit sugere probe-rs nativo");
        check(destination.visible, "a seção de destino aparece para a sonda");
        check(controller.projectFrameworkLabel === "Bare metal · STM32F411RETx", "rótulo bare metal pelo chip do kit");
        check(client.targetCalls === 1 && flash.targetsBusy, "modelo novo pede os alvos");

        client.cmakeTargetsResolved(root.executables, "fileApi");
        check(!flash.targetsBusy && flash.targets.length === 2, "só executáveis do file-api desta raiz");
        check(flash.targets.every(item => item.kind === "executable" && item.name !== "foreign"), "biblioteca e outra raiz fora");
        client.cmakeTargetsResolved([{ name: "stray", kind: "executable", sourceDir: "/ws" }], "fileApi");
        check(flash.targets.length === 2, "resposta não pedida (Biblioteca) não substitui a lista");

        flash.requestTargets();
        client.cmakeTargetsResolved([{ name: "firmware", kind: "executable" }], "source");
        check(flash.targetsOrigin === "source" && flash.targets.length === 0, "nomes lidos da fonte não são escolhíveis");
        flash.requestTargets();
        client.requestFailed("cmake.targets.list", "kind não suportado", "INVALID_PARAMS");
        check(!flash.targetsBusy && flash.targetsError === "kind não suportado", "falha encerra a espera com a causa");
        flash.requestTargets();
        client.cmakeTargetsResolved(root.executables, "fileApi");

        client.probesResolved(root.listed, true, "", "");
        const selectable = controller.probes.filter(entry => flash.probeSelectable(entry));
        check(selectable.length === 2, "sem serial e inacessível não são destino");
        check(flash.selectorOf(root.listed[3]) === "0403:6010-1:FT1", "interface entra no seletor");
        check(overview.nextStep === "board", "sem sonda o próximo passo é escolher a sonda");

        flash.selectBuildDirectory("ignored");
        flash.selectTarget("firmware");
        flash.selectProbe("0483:374b:066D");
        check(overview.nextStep === "firmware", "sonda escolhida libera o passo de firmware");
        controller.selectedPort = "/dev/ttyACM0";
        flash.propose(controller.selectedPort, 4096);
        const sent = client.lastProposal;
        check(sent[0] === "" && sent[2] === 0 && sent[4] === true && sent[5] === "", "sem porta, tamanho ou diretório");
        check(sent[6] === "firmware" && sent[7] === "0483:374b:066D", "alvo e sonda vão ao core");
        client.flashProposalResolved({ command: "probe-rs download", flash: { version: 1 } }, client.sequence);
        check(flash.found, "prévia nativa aceita");
        flash.selectProbe("0403:6010-1:FT1");
        check(!flash.found, "trocar a sonda invalida a prévia");
        flash.selectTarget("bootloader");
        check(flash.target === "bootloader", "trocar alvo");

        flash.selectEngine("esptool");
        check(!flash.probeDestination && !destination.visible, "esptool não usa sonda");
        flash.propose(controller.selectedPort, 4096);
        check(client.lastProposal[0] === "/dev/ttyACM0" && client.lastProposal[6] === "" && client.lastProposal[7] === "",
              "outro motor nunca envia alvo/sonda");
        client.flashProposalFailed(client.sequence, "esptool");
        flash.selectEngine("");
        flash.selectMode(false);
        check(!flash.probeDestination, "modo manual não usa o destino nativo");
        flash.selectMode(true);
        check(flash.probeDestination, "volta ao nativo");

        flash.requestTargets();
        controller.workspaceRoot = "/other";
        check(flash.target === "" && flash.probe === "" && flash.targets.length === 0 && !flash.targetsBusy,
              "troca de workspace zera alvo, sonda e lista");
        client.cmakeTargetsResolved(root.executables, "fileApi");
        check(flash.targets.length === 0, "resposta do workspace anterior descartada");
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
