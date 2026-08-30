// Aviso de saude do projeto — em especial a CDB DESATUALIZADA (roadmap 29 §5c).
//
// Por que existe: o core ja media `cdbStale` desde o protocolo 0.62.0 e a UI
// simplesmente descartava o campo (o sinal C++ nem o carregava). Um dado medido
// que nao chega a tela e' indistinguivel de nao ter sido medido — e nada
// reprovava, porque nenhum teste exercia este controller.
import QtQuick
import "../../ui/qml/workspace"

Item {
    id: root

    property int configureRequests: 0

    ProjectHealthController {
        id: controller

        workspaceRoot: "/tmp/proj"
        workspaceKind: "cmake"
        workspaceBuildSystems: ["cmake"]
        toolsList: [
            { id: "cmake", status: "detected" },
            { id: "ninja", status: "detected" },
            { id: "clangd", status: "detected" },
            { id: "gxx", status: "detected" }
        ]

        onAutoConfigureRequested: root.configureRequests += 1
    }

    Component.onCompleted: {
        let failures = 0;

        // Configurado e CDB fresca: nenhum aviso, e nenhum auto-configure.
        controller.handleCmakeStatus(true, false, "");
        if (controller.status !== "ok" || controller.active) failures += 1;
        if (root.configureRequests !== 0) failures += 2;

        // CDB velha: aviso ACIONAVEL, e ele nomeia o arquivo que a invalidou.
        controller.handleCmakeStatus(true, true, "CMakeLists.txt");
        if (controller.status !== "warning" || !controller.active) failures += 4;
        if (controller.message.indexOf("CMakeLists.txt") < 0) failures += 8;
        if (controller.actionTarget !== "cmakeConfigure") failures += 16;

        // Sem o nome do arquivo o aviso ainda tem que ser acionavel.
        controller.handleCmakeStatus(true, true, "");
        if (controller.status !== "warning"
                || controller.actionTarget !== "cmakeConfigure") failures += 32;

        // Volta a ficar fresca depois do reconfigure: o aviso SAI da tela.
        controller.handleCmakeStatus(true, false, "");
        if (controller.status !== "ok" || controller.active) failures += 64;

        // Sem configure nenhum o aviso maior VENCE: falar de CDB velha ali
        // seria ruido em cima de um problema que ja esta na tela. Aqui o
        // controller dispara o auto-configure (uma vez) e diz isso; o que este
        // check trava e' que o texto NAO e' o de CDB desatualizada.
        controller.handleCmakeStatus(false, true, "CMakeLists.txt");
        if (controller.message.indexOf("desatualizada") >= 0) failures += 128;
        if (root.configureRequests !== 1) failures += 256;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
