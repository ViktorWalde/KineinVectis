import QtQuick

// Espelho do EmbeddedEventRouter: leva ao core o que o controller pede.
Item {
    id: root

    property var coreClient: null
    property var embeddedController: null

    visible: false

    function available(method) {
        if (coreClient && coreClient.connected !== false) return true;
        embeddedController.handleFailed(method, qsTr("O serviço da IDE está desconectado. Aguarde a reconexão e tente novamente."));
        return false;
    }

    Connections {
        target: root.embeddedController

        function onListRequested() {
            if (root.available("probe.list")) root.coreClient.probeList();
        }

        function onSizeRequested(program) {
            if (root.available("build.size")) root.coreClient.buildSize(program);
        }

        function onSerialListRequested() {
            if (root.available("serial.list")) root.coreClient.serialList();
        }

        function onProjectRequested() {
            if (root.available("project.model")) root.coreClient.projectModel();
        }

        function onMonitorRequested(device, baud) {
            if (root.available("serial.monitor")) root.coreClient.serialMonitor(device, baud);
        }
    }

    // Identidade pelo canal (E5): o sinal e' do controller FILHO `identity`,
    // nao do EmbeddedController — ligado ao pai, o Qt avisava "no signal of
    // the target matches" e o serial.identify nunca saia (achado em
    // 2026-09-18, na foto da F7).
    Connections {
        target: root.embeddedController ? root.embeddedController.identity : null

        function onIdentifyRequested(device) {
            if (root.available("serial.identify")) root.coreClient.serialIdentify(device);
        }
    }

    // Gravar (E4): a previa nao grava nem salva; rodar e salvar vao
    // pelos donos de execucao (AppDomains), nao por aqui.
    Connections {
        target: root.embeddedController ? root.embeddedController.flash : null

        function onProposalRequested(device, engine, flashSizeBytes, firmware, nativeMode, buildDirectory) {
            const flash = root.embeddedController.flash;
            if (!root.available("runConfig.flashProposal")) {
                flash.bindProposal(-1);
                return;
            }
            flash.bindProposal(root.coreClient.runConfigFlashProposal(device, engine, flashSizeBytes, firmware, nativeMode, buildDirectory));
        }
    }

    // Permissao por canal (E2): so' o diagnostico passa por aqui; o passo vai
    // para o terminal da IDE pelo ShellEnvironmentOverlays, como o setup.
    Connections {
        target: root.embeddedController ? root.embeddedController.access : null

        function onDiagnoseRequested(device) {
            if (root.available("serial.access")) root.coreClient.serialAccess(device);
        }
    }

    // Arquivos na placa (C2): cada gesto e' um job; o que escreve ja' foi
    // confirmado no controller antes de chegar aqui.
    Connections {
        target: root.embeddedController ? root.embeddedController.files : null

        function onFilesRequested(device, action, path, local) {
            if (root.available("serial.files")) root.coreClient.serialFiles(device, action, path, local);
        }
    }
}
