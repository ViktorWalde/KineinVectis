import QtQuick
import KineinVectis

// Real controllers/routers; replies can arrive out of order across workspaces.
Item {
    id: root
    property int failures: 0
    property int executions: 0
    function check(ok, message) {
        if (!ok) { failures++; console.error("FALHAS: " + message); }
    }
    QtObject {
        id: client
        property bool connected: true
        property int sequence: 0
        property bool rejectSend: false
        signal statusChanged()
        signal workspaceChanged()
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
        function runConfigFlashProposal(device, engine, size, firmware) {
            return rejectSend ? -1 : ++sequence;
        }
    }
    EmbeddedController { id: controller; workspaceRoot: "/first" }
    EmbeddedRequestRouter { coreClient: client; embeddedController: controller }
    EmbeddedEventRouter { coreClient: client; embeddedController: controller }
    Connections {
        target: controller.flash
        function onRunRequested(command) { root.executions++; }
        function onSaveRequested(name, command) { root.executions++; }
    }
    Component.onCompleted: {
        client.statusChanged();
        const flash = controller.flash;
        flash.selectMode(false); flash.propose("/first-port", 0);
        const first = client.sequence;
        controller.workspaceRoot = "/second";
        flash.selectMode(false); flash.propose("/second-port", 0);
        const second = client.sequence;
        client.flashProposalResolved({command: "old-workspace"}, first);
        check(flash.busy && !flash.found, "resposta antiga não conclui pedido de outro workspace");
        flash.run(); flash.save();
        check(executions === 0, "prévia antiga não habilita execução ou salvamento");
        client.requestFailed("runConfig.flashProposal", "generic old failure", "INVALID_REQUEST");
        client.flashProposalFailed(first, "old failure");
        check(flash.busy && flash.errorText === "", "falhas genéricas ou antigas não encerram o pedido atual");
        client.flashProposalResolved({command: "current-workspace"}, second);
        check(!flash.busy && flash.proposal.command === "current-workspace", "resposta atual conclui sua própria prévia");
        client.flashProposalResolved({command: "duplicate"}, second);
        client.flashProposalFailed(second, "duplicate failure");
        check(flash.proposal.command === "current-workspace" && flash.errorText === "", "duplicatas não substituem o resultado concluído");

        flash.selectMode(false); flash.propose("/second-port", 0);
        const changed = client.sequence;
        controller.selectedPort = "/third-port";
        client.flashProposalResolved({command: "old-port"}, changed);
        check(!flash.busy && !flash.found, "id correto não valida contexto alterado");
        flash.selectMode(false); flash.propose(controller.selectedPort, 0);
        client.flashProposalFailed(client.sequence, "versão desconhecida: confira o binário");
        check(!flash.busy && !flash.found && flash.errorText.indexOf("versão") >= 0, "recusa atual chega com causa integral");

        client.rejectSend = true;
        flash.selectMode(false); flash.propose(controller.selectedPort, 0);
        check(!flash.busy && !flash.found && flash.errorText !== "", "envio recusado não deixa espera presa");
        client.rejectSend = false;
        flash.selectMode(false); flash.propose(controller.selectedPort, 0);
        const disconnected = client.sequence;
        client.connected = false;
        client.statusChanged();
        check(!flash.busy && !flash.found, "desconexão invalida espera e prévia");
        const count = client.sequence;
        client.connected = true;
        client.statusChanged();
        check(client.sequence === count, "reconexão não repete consulta");
        flash.selectMode(false); flash.propose(controller.selectedPort, 0);
        client.flashProposalResolved({command: "old-connection"}, disconnected);
        check(flash.busy && !flash.found, "resposta de conexão antiga não conclui pedido novo");
        client.flashProposalResolved({command: "new-connection"}, client.sequence);
        check(!flash.busy && flash.proposal.command === "new-connection", "consulta explícita após reconectar funciona");
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
