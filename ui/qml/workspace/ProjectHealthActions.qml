import QtQuick

// O QUE CADA ACAO DA FAIXA DE SAUDE FAZ: um gesto de um clique por alvo. A
// faixa e o "proximo passo" dos Problemas usam a mesma. Saiu do Main.qml em
// 2026-10-16 (16.4b, roadmaps/59 §7.1.4), que estava no limite e ganhava o
// alvo "trust" — o despacho e' responsabilidade propria, e assim um harness
// o exercita com dubles.
QtObject {
    id: root

    property var coreClient: null
    property var shellController: null
    property var pythonController: null

    function perform(target) {
        if (target === "scan") {
            root.coreClient.scanEnvironment();
        } else if (target === "cmakeConfigure") {
            root.shellController.showTab("jobs");
            root.coreClient.cmakeConfigure();
        } else if (target === "cargoMetadata") {
            root.coreClient.cargoMetadata();
        } else if (target === "trust") {
            // O gesto que libera o CMake automatico e os build scripts.
            root.coreClient.trustWorkspace();
        } else if (target === "pythonEnvironment") {
            root.shellController.showTab("jobs");
            root.pythonController.createEnvironment();
        } else if (target === "pythonStubs") {
            root.shellController.showTab("jobs");
            root.pythonController.installStubs();
        } else if (target !== "") {
            root.shellController.showTab(target);
        }
    }
}
