import QtQuick
import KineinVectis

Item {
    id: root
    width: 380
    height: 700
    property int failures: 0
    property var payload: ({})
    property int runs: 0
    function check(ok, message) {
        if (!ok) { failures++; console.error("DFU_FLASH_FAIL " + message); }
    }
    QtObject {
        id: client
        property bool connected: true
        property int sequence: 0
        signal statusChanged()
        signal workspaceChanged()
        signal picotoolListResolved(var result, double requestId)
        signal picotoolListFailed(double requestId, string message)
        signal dfuListResolved(var result, double requestId)
        signal dfuListFailed(double requestId, string message)
        signal probesResolved(var probes, bool toolAvailable, string rawOutput, string hint)
        signal buildSizeResolved(var sections, var regions, bool toolAvailable, string tool, string rawOutput)
        signal projectModelResolved(var model)
        signal cmakeTargetsResolved(var targets, string origin)
        signal projectChanged(var model)
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
        function dfuList() { return ++sequence; }
        function cmakeTargetsList() { return ++sequence; }
        function runConfigFlashProposal(device, engine, size, firmware, nativeMode, buildDirectory, target, probe, dfu) {
            root.payload = {device: device, engine: engine, size: size, firmware: firmware, nativeMode: nativeMode,
                buildDirectory: buildDirectory, target: target, probe: probe, dfu: dfu};
            return ++sequence;
        }
    }
    EmbeddedController { id: controller; workspaceRoot: "/first" }
    EmbeddedRequestRouter { coreClient: client; embeddedController: controller }
    EmbeddedEventRouter { coreClient: client; embeddedController: controller }
    EmbeddedDfuFlashView { width: parent.width; controller: controller; flash: controller.flash }
    Connections {
        target: controller.flash
        function onRunRequested(command, flash) { root.runs++; }
    }
    property var entry: ({mode: "dfu", vid: "0483", pid: "df11", serial: "S1", path: "1-2",
        configuration: 1, interface: 0, alternate: 0, name: "@Internal Flash"})
    function listed() {
        controller.dfu.refresh();
        client.dfuListResolved({complete: true, toolAvailable: true, interfaces: [entry]}, client.sequence);
    }
    Component.onCompleted: {
        client.statusChanged();
        const flash = controller.flash;
        const selected = flash.dfu;
        flash.selectEngine("dfu-util");
        check(flash.dfuDestination && selected.discovery === controller.dfu, "native USB destination uses current discovery owner");
        check(!selected.selectable({mode: "runtime", vid: "0483", pid: "df11", serial: "S1", configuration: 1, interface: 0}),
              "Runtime mode cannot be selected for native download");
        selected.select(entry);
        check(!selected.destination.serial, "unlisted destination cannot be selected");
        listed(); selected.setImage("firmware a'b.dfu"); selected.select(entry);
        check(selected.destination.serial === "S1", "listed alternative is explicitly chosen");
        flash.selectBuildDirectory("foreign-build"); flash.selectTarget("foreign-target"); flash.selectProbe("foreign-probe");
        flash.propose("/dev/unused", 123);
        check(payload.device === "" && payload.size === 0 && payload.buildDirectory === "" && payload.target === "" && payload.probe === "",
              "USB proposal cannot inherit serial or probe fields");
        check(payload.dfu.image === "firmware a'b.dfu" && payload.dfu.destination.serial === "S1", "typed image and USB selection reach bridge");
        const old = client.sequence;
        selected.setImage("changed.dfu");
        client.flashProposalResolved({command: "stale", flash: {version: 1}}, old);
        check(!flash.found && !flash.busy, "editing image invalidates pending preview");
        flash.propose("", 0);
        client.flashProposalResolved({command: "literal argv", flash: {version: 1}}, client.sequence);
        flash.run(); check(runs === 1 && flash.found, "only reviewed native intent can request execution");
        controller.dfu.refresh();
        check(!flash.found && !selected.destination.serial && selected.image === "changed.dfu", "new listing invalidates destination and preview while preserving image draft");
        client.dfuListResolved({complete: true, toolAvailable: true, interfaces: [entry]}, client.sequence);
        selected.select(entry); selected.select(entry);
        check(!selected.destination.serial, "selected alternative toggles off");
        selected.select(entry);
        flash.selectMode(false); flash.propose("", 0);
        check(Object.keys(payload.dfu).length === 0, "manual mode never transports native USB intent");
        client.flashProposalResolved({command: "explicit manual command"}, client.sequence);
        listed();
        check(flash.found, "USB discovery does not invalidate an unrelated manual preview");
        flash.selectMode(true);
        selected.selectBinary(true);
        selected.setImage(".kinein/build/firmware.bin");
        listed(); selected.select(entry);
        flash.requestTargets();
        client.cmakeTargetsResolved([{name: "firmware", kind: "executable", sourceDir: "/first"}], "source");
        check(flash.targets.length === 0, "source parsing cannot offer a BIN target");
        flash.requestTargets();
        client.cmakeTargetsResolved([{name: "firmware", kind: "executable", sourceDir: "/first", artifacts: ["/first/.kinein/build/firmware.elf"]}], "fileApi");
        flash.selectTarget("firmware");
        flash.propose("/dev/ignored", 1);
        check(payload.dfu.binaryTarget === "firmware" && payload.target === "" && payload.device === "",
              "BIN uses nested target and never inherits another destination");
        client.flashProposalResolved({command: "BIN literal", flash: {version: 1}}, client.sequence);
        check(flash.found, "BIN native preview accepted");
        flash.requestTargets();
        check(!flash.found, "refreshing build targets invalidates BIN preview");
        client.cmakeTargetsResolved([], "fileApi");
        check(flash.target === "", "vanished CMake target cannot remain selected");
        selected.selectBinary(false); flash.propose("", 0);
        check(payload.dfu.binaryTarget === undefined, "DfuSe never inherits BIN target");
        client.flashProposalResolved({command: "DfuSe literal", flash: {version: 1}}, client.sequence);
        selected.selectBinary(true);
        check(!flash.found, "changing the file format invalidates preview");
        controller.workspaceRoot = "/second";
        check(selected.image === "" && !selected.binaryMode && !selected.destination.serial && !flash.found, "workspace clears image, selection and preview");
        flash.selectEngine("dfu-util"); selected.setImage("second.dfu"); listed(); selected.select(entry);
        flash.propose("", 0);
        client.connected = false; client.statusChanged();
        check(!selected.destination.serial && !flash.found && !flash.busy && controller.dfu.interfaces.length === 0,
              "disconnect cannot leave actionable USB selection or preview");
        console.log("DFU_FLASH failures=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
