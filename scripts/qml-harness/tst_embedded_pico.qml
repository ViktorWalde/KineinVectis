import QtQuick
import KineinVectis

// Real controllers, routers and view; no USB device is opened by this fixture.
Item {
    id: root
    width: 360
    height: 680
    property int failures: 0
    function check(ok, message) {
        if (!ok) { failures++; console.error("PICO_FAIL " + message); }
    }
    QtObject {
        id: client
        property bool connected: true
        property int sequence: 0
        property bool rejectSend: false
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
        function picotoolList() { return rejectSend ? -1 : ++sequence; }
        function dfuList() { return ++sequence; }
        function probeList() {}
        function serialList() {}
        function projectModel() {}
    }
    EmbeddedController { id: controller; workspaceRoot: "/first" }
    EmbeddedRequestRouter { coreClient: client; embeddedController: controller }
    EmbeddedEventRouter { coreClient: client; embeddedController: controller }
    EmbeddedPicoView { id: view; width: root.width; controller: controller.usb.pico }
    property var result: ({complete: true, toolAvailable: true, tool: "/fixture/picotool", version: "2.3.1", rawOutput: "confirmed",
        devices: [{chip: "rp2040", serialSelector: "E660583883355B21", flashSizeBytes: 2097152}]})
    Component.onCompleted: {
        client.statusChanged();
        const pico=controller.usb.pico;
        check(!pico.measured && client.sequence===0, "no automatic USB claim on open");
        pico.refresh(); const old=client.sequence;
        pico.refresh(); const current=client.sequence;
        client.picotoolListResolved(result,old);client.picotoolListFailed(old,"old");
        check(pico.busy && pico.devices.length===0, "old responses cannot finish latest query");
        client.picotoolListResolved(result,current);
        check(pico.complete && pico.measured && !pico.busy && pico.devices.length===1,"typed confirmed devices presented");
        check(controller.boardVerdict===pico.verdict,"USB owner presents queried RP2040 state");
        check(pico.summary(pico.devices[0]).indexOf("E660583883355B21")>=0,"selector comes from confirmed result");
        client.picotoolListFailed(current,"duplicate");check(pico.complete,"duplicate errors ignored");
        pico.refresh();client.picotoolListResolved({complete:false,toolAvailable:true,devices:result.devices,hint:"permission",rawOutput:"denied"},client.sequence);
        check(!pico.complete && pico.devices.length===0 && pico.errorText==="permission","failed query never exposes partial devices");
        pico.refresh();const previous=client.sequence;controller.workspaceRoot="/second";
        client.picotoolListResolved(result,previous);check(!pico.busy && !pico.measured && pico.devices.length===0,"workspace invalidates identity");
        pico.refresh();const same=client.sequence;client.workspaceChanged();client.picotoolListResolved(result,same);
        check(!pico.measured && pico.devices.length===0,"same-root activation clears USB query");
        pico.refresh();client.picotoolListFailed(client.sequence,"tool changed");
        check(!pico.busy && pico.errorText==="tool changed","current failure visible");
        client.rejectSend=true;pico.refresh();check(!pico.busy && pico.errorText!=="","rejected transport clears busy");client.rejectSend=false;
        pico.refresh();const pending=client.sequence;client.connected=false;client.statusChanged();
        client.picotoolListResolved(result,pending);check(!pico.measured && pico.devices.length===0 && !pico.busy,"disconnect forbids stale data");
        check(controller.boardVerdict==="Dispositivos a verificar","disconnected header keeps unknown state");
        const sent=client.sequence;pico.refresh();check(client.sequence===sent && !pico.busy,"no request while disconnected");
        console.log("PICO failures="+failures);Qt.exit(failures===0?0:1);
    }
}
