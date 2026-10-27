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
        if (!ok) { failures++; console.error("PICO_FLASH_FAIL " + message); }
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
        function picotoolList() { return ++sequence; }
        function cmakeTargetsList() { return ++sequence; }
        function runConfigFlashProposal(device, engine, size, firmware, nativeMode, buildDirectory, target, probe, dfu, pico) {
            root.payload = {device: device, engine: engine, size: size, firmware: firmware, nativeMode: nativeMode,
                buildDirectory: buildDirectory, target: target, probe: probe, dfu: dfu, pico: pico};
            return ++sequence;
        }
    }
    EmbeddedController { id: controller; workspaceRoot: "/first" }
    EmbeddedRequestRouter { coreClient: client; embeddedController: controller }
    EmbeddedEventRouter { coreClient: client; embeddedController: controller }
    EmbeddedPicoFlashView { width: parent.width; controller: controller; flash: controller.flash }
    Connections {
        target: controller.flash
        function onRunRequested(command, flash) { root.runs++; }
    }
    property var entry: ({chip:"rp2040", serialSelector:"E660583883355B21", flashSizeBytes:2097152})
    function listed() {
        controller.usb.pico.refresh();
        client.picotoolListResolved({complete:true,toolAvailable:true,devices:[entry]},client.sequence);
    }
    Component.onCompleted: {
        client.statusChanged();
        const flash=controller.flash;
        const selected=flash.pico;
        flash.selectEngine("picotool");
        check(flash.picoDestination && flash.usbDestination && selected.discovery===controller.usb.pico,"native Pico shares USB owner");
        selected.select(entry);check(!selected.destination.serialSelector,"unlisted cannot be selected");
        listed();selected.setImage(".kinein/build/firmware a'b.uf2");selected.select(entry);
        check(selected.destination.serialSelector===entry.serialSelector,"confirmed destination chosen");
        flash.selectBuildDirectory("foreign");flash.selectProbe("foreign");
        flash.requestTargets();
        client.cmakeTargetsResolved([{name:"firmware",kind:"executable",sourceDir:"/first"}],"source");
        check(flash.targets.length===0,"parsed source is not a build target");
        flash.requestTargets();
        client.cmakeTargetsResolved([{name:"firmware",kind:"executable",sourceDir:"/first",artifacts:["/first/.kinein/build/firmware.elf"]}],"fileApi");
        flash.selectTarget("firmware");flash.propose("/dev/ignored",999);
        check(payload.device==="" && payload.size===0 && payload.probe==="" && payload.buildDirectory==="" && Object.keys(payload.dfu).length===0,"Pico cannot inherit other destination fields");
        check(payload.target==="firmware" && payload.pico.image===selected.image && payload.pico.destination.serialSelector===entry.serialSelector,"typed UF2 target and USB destination reach bridge");
        const pending=client.sequence;selected.setImage(".kinein/build/new.uf2");
        client.flashProposalResolved({command:"stale",flash:{version:1}},pending);
        check(!flash.found && !flash.busy,"image edit invalidates old reply");
        flash.propose("",0);client.flashProposalResolved({command:"literal argv",flash:{version:1}},client.sequence);
        flash.run();check(runs===1 && flash.found,"reviewed native intent can request execution");
        flash.requestTargets();check(!flash.found,"target refresh invalidates preview");client.cmakeTargetsResolved([],"fileApi");
        check(flash.target==="","removed target cannot stay selected");
        flash.propose("",0);client.flashProposalResolved({command:"another",flash:{version:1}},client.sequence);
        controller.usb.pico.refresh();
        check(!flash.found && !selected.destination.serialSelector && selected.image!=="","refresh clears destination and preview, preserves image draft");
        client.picotoolListResolved({complete:true,toolAvailable:true,devices:[entry]},client.sequence);
        selected.select(entry);selected.select(entry);check(!selected.destination.serialSelector,"selection toggles off");selected.select(entry);
        flash.selectMode(false);flash.propose("",0);check(Object.keys(payload.pico).length===0,"manual mode omits native Pico intent");
        client.flashProposalResolved({command:"manual"},client.sequence);listed();check(flash.found,"discovery does not invalidate manual preview");
        flash.selectMode(true);selected.select(entry);flash.propose("",0);const old=client.sequence;
        client.workspaceChanged();client.flashProposalResolved({command:"old workspace",flash:{version:1}},old);
        check(!flash.found && !selected.destination.serialSelector,"same-root activation cannot restore old preview");
        controller.workspaceRoot="/second";check(selected.image==="" && !flash.found,"workspace clears image");
        flash.selectEngine("picotool");selected.setImage(".kinein/build/second.uf2");listed();selected.select(entry);flash.propose("",0);
        client.connected=false;client.statusChanged();check(!selected.destination.serialSelector && !flash.found && !flash.busy,"disconnect clears actionable native state");
        console.log("PICO_FLASH failures="+failures);Qt.exit(failures===0?0:1);
    }
}
