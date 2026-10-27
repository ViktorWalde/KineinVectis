import QtQuick
import KineinVectis

// Real controllers, routers and view; no USB device is opened by this fixture.
Item {
    id: root
    width: 360
    height: 680
    property int failures: 0
    function check(ok, message) {
        if (!ok) { failures++; console.error("DFU_FAIL " + message); }
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
        function dfuList() { return rejectSend ? -1 : ++sequence; }
        function probeList() {}
        function serialList() {}
        function projectModel() {}
    }
    EmbeddedController { id: controller; workspaceRoot: "/first" }
    EmbeddedRequestRouter { coreClient: client; embeddedController: controller }
    EmbeddedEventRouter { coreClient: client; embeddedController: controller }
    EmbeddedDfuView { id: view; width: root.width; controller: controller.dfu }
    property var result: ({complete: true, toolAvailable: true, tool: "/fixture/dfu-util", rawOutput: "raw descriptors",
        interfaces: [{mode: "dfu", vid: "0483", pid: "df11", configuration: 1, interface: 0,
                      alternate: 0, name: "@Internal Flash", path: "1-2", serial: "S1"}]})

    Component.onCompleted: {
        client.statusChanged();
        const dfu = controller.dfu;
        check(client.sequence === 0 && !dfu.measured, "discovery starts only by explicit gesture");
        dfu.refresh(); const old = client.sequence;
        dfu.refresh(); const current = client.sequence;
        client.dfuListResolved(result, old);
        client.dfuListFailed(old, "old failure");
        check(dfu.busy && dfu.interfaces.length === 0 && dfu.errorText === "", "stale success and failure ignored");
        client.dfuListResolved(result, current);
        check(!dfu.busy && dfu.complete && dfu.measured && dfu.interfaces.length === 1, "current result presented");
        check(controller.boardVerdict === dfu.verdict && controller.boardVerdict.indexOf("1 alternativa") >= 0,
              "panel header presents DFU alternatives instead of device absence");
        check(dfu.tool === result.tool && dfu.rawOutput === result.rawOutput, "diagnostic output and selected tool retained");
        check(dfu.summary(dfu.interfaces[0]).indexOf("0483:df11") >= 0, "descriptor summary comes from core result");
        client.dfuListFailed(current, "duplicate");
        client.dfuListResolved({complete: true, interfaces: []}, current);
        check(dfu.interfaces.length === 1 && dfu.errorText === "", "duplicate responses cannot replace completed results");

        dfu.refresh(); const workspace = client.sequence;
        controller.workspaceRoot = "/second";
        client.dfuListResolved(result, workspace);
        client.dfuListFailed(workspace, "old workspace");
        check(!dfu.busy && !dfu.measured && dfu.interfaces.length === 0 && dfu.errorText === "", "workspace clears state and pending identity");
        dfu.refresh();
        client.dfuListResolved({complete: false, toolAvailable: true, interfaces: result.interfaces,
            rawOutput: "Cannot open DFU device", hint: "USB access denied"}, client.sequence);
        check(!dfu.complete && dfu.toolAvailable && dfu.interfaces.length === 0 && dfu.errorText === "USB access denied", "unverified listing cannot expose partial interfaces");
        check(controller.boardVerdict === dfu.verdict && controller.boardVerdict.indexOf("não verificada") >= 0,
              "unverified discovery cannot become device absence in the header");
        dfu.refresh();
        client.dfuListFailed(client.sequence, "tool changed");
        check(!dfu.busy && !dfu.measured && dfu.errorText === "tool changed", "current failure clears pending operation");
        client.rejectSend = true;
        dfu.refresh();
        check(!dfu.busy && dfu.errorText !== "", "rejected send cannot leave busy state stuck");
        client.rejectSend = false;

        dfu.refresh(); const disconnected = client.sequence;
        client.connected = false; client.statusChanged();
        check(!dfu.busy && !dfu.measured && dfu.interfaces.length === 0 && dfu.errorText !== "", "disconnect invalidates discovery");
        check(controller.boardVerdict === "Dispositivos a verificar", "disconnect preserves the unknown device state in the panel header");
        client.dfuListResolved(result, disconnected);
        check(dfu.interfaces.length === 0, "response after disconnect ignored");
        const sent = client.sequence;
        dfu.refresh();
        check(!dfu.busy && client.sequence === sent && dfu.errorText !== "", "disconnected gesture never calls stopped bridge");
        client.connected = true; client.statusChanged();
        check(client.sequence === sent, "reconnection never repeats USB discovery");
        dfu.refresh();
        client.dfuListResolved({complete: true, toolAvailable: true, interfaces: [], hint: "No DFU interfaces"}, client.sequence);
        check(dfu.complete && dfu.measured && dfu.interfaces.length === 0 && dfu.errorText === "" && dfu.hint !== "", "complete empty query differs from failed query");
        controller.refresh();
        check(!dfu.measured && !dfu.complete && !dfu.busy && dfu.requestId < 0 && dfu.rawOutput === "",
              "global refresh invalidates previous USB observations without querying automatically");
        console.log("DFU assertions failures=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
