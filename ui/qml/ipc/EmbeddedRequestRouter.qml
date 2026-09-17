import QtQuick

// Espelho do EmbeddedEventRouter: leva ao core o que o controller pede.
Item {
    id: root

    property var coreClient: null
    property var embeddedController: null

    visible: false

    Connections {
        target: root.embeddedController

        function onListRequested() {
            root.coreClient.probeList();
        }

        function onSizeRequested(program) {
            root.coreClient.buildSize(program);
        }

        function onSerialListRequested() {
            root.coreClient.serialList();
        }

        function onProjectRequested() {
            root.coreClient.projectModel();
        }

        function onMonitorRequested(device, baud) {
            root.coreClient.serialMonitor(device, baud);
        }

        function onIdentifyRequested(device) {
            root.coreClient.serialIdentify(device);
        }
    }

    // Gravar (E4): a previa e' um pedido puro ao core; rodar e salvar vao
    // pelos donos de execucao (AppDomains), nao por aqui.
    Connections {
        target: root.embeddedController ? root.embeddedController.flash : null

        function onProposalRequested(device, engine, flashSizeBytes, firmware) {
            root.coreClient.runConfigFlashProposal(device, engine, flashSizeBytes, firmware);
        }
    }

    // Permissao por canal (E2): so' o diagnostico passa por aqui; o passo vai
    // para o terminal da IDE pelo ShellEnvironmentOverlays, como o setup.
    Connections {
        target: root.embeddedController ? root.embeddedController.access : null

        function onDiagnoseRequested(device) {
            root.coreClient.serialAccess(device);
        }
    }

    // Arquivos na placa (C2): cada gesto e' um job; o que escreve ja' foi
    // confirmado no controller antes de chegar aqui.
    Connections {
        target: root.embeddedController ? root.embeddedController.files : null

        function onFilesRequested(device, action, path, local) {
            root.coreClient.serialFiles(device, action, path, local);
        }
    }
}
