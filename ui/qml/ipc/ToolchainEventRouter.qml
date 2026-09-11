import QtQuick

// O que o core responde de `toolchain.*` -> ToolchainController.
//
// Inclui o `requestFailed` porque aqui a recusa e informacao de produto:
// "g++ nao foi detectado nesta maquina" e a resposta a pergunta do usuario,
// nao ruido de log.
Item {
    id: root

    property var coreClient: null
    property var toolchainController: null

    visible: false

    Connections {
        target: root.coreClient

        function onToolchainResolved(selections, candidates, preset, sysroot, targetTriple,
                                     chip, presetToolchainFile) {
            root.toolchainController.handleResolved(selections, candidates, preset, sysroot,
                                                    targetTriple, chip, presetToolchainFile);
        }

        function onRequestFailed(method, message) {
            root.toolchainController.handleFailed(method, message);
        }
    }
}
