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
        controller.handleCmakeStatus(true, false, "", "linux-clang");
        // O preset com que a IDE configurou (P0, 0.115.0) fica visivel;
        // ausente/nulo vira vazio, nao "undefined".
        if (controller.cmakePreset !== "linux-clang") failures += 1 << 20;
        controller.handleCmakeStatus(true, false, "", null);
        if (controller.cmakePreset !== "") failures += 1 << 21;
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

        // Python sem ambiente (bloco B do roadmaps/41): a faixa e' ACIONAVEL,
        // com o rotulo da ferramenta e o alvo do gesto; enquanto cria, vira
        // informativa apontando para os jobs; com ambiente, nada.
        controller.handleCmakeStatus(true, false, "");
        controller.pythonMessage = "o projeto usa o Python do SISTEMA: crie um ambiente";
        controller.pythonActionLabel = "Criar .venv com uv";
        controller.pythonNeedsEnvironment = true;
        if (controller.status !== "warning" || controller.actionTarget !== "pythonEnvironment"
                || controller.actionLabel !== "Criar .venv com uv") failures += 512;
        controller.pythonCreating = true;
        controller.pythonMessage = "criando o ambiente Python…";
        if (controller.status !== "info" || controller.actionTarget !== "jobs") failures += 1024;
        controller.pythonCreating = false;
        controller.pythonNeedsEnvironment = false;
        if (controller.status !== "ok" || controller.active) failures += 2048;
        // Os stubs da placa (C4, 2026-09-17): faixa INFORMATIVA, depois do
        // ambiente (que e' aviso); instalando, aponta para os jobs.
        controller.pythonStubsMessage = "MicroPython sem os stubs da placa (micropython-esp32-stubs)";
        controller.pythonNeedsStubs = true;
        if (controller.status !== "info" || controller.actionTarget !== "pythonStubs"
                || controller.actionLabel !== "Instalar stubs") failures += 33554432;
        controller.pythonInstallingStubs = true;
        if (controller.actionTarget !== "jobs") failures += 67108864;
        controller.pythonInstallingStubs = false;
        controller.pythonNeedsEnvironment = true;
        if (controller.actionTarget !== "pythonEnvironment") failures += 134217728;
        controller.pythonNeedsEnvironment = false;
        controller.pythonNeedsStubs = false;
        if (controller.status !== "ok") failures += 268435456;
        // A CDB velha do C++ vem ANTES do Python: um aviso de cada vez, o mais
        // grave primeiro.
        controller.pythonNeedsEnvironment = true;
        controller.handleCmakeStatus(true, true, "CMakeLists.txt");
        if (controller.actionTarget !== "cmakeConfigure") failures += 4096;

        // Makefile puro (P0, 2026-09-17): sem bear, a faixa nomeia o bear
        // como ferramenta ausente (com make, clangd e g++ presentes); com
        // bear, nada. Um Makefile AO LADO do CMake nao exige bear.
        controller.pythonNeedsEnvironment = false;
        controller.handleCmakeStatus(true, false, "");
        controller.workspaceKind = "make";
        controller.workspaceBuildSystems = ["make"];
        controller.toolsList = [
            { id: "make", status: "detected" },
            { id: "clangd", status: "detected" },
            { id: "gxx", status: "detected" }
        ];
        if (controller.status !== "warning" || controller.message.indexOf("bear") < 0
                || controller.actionTarget !== "tools") failures += 1 << 22;
        controller.toolsList = [
            { id: "make", status: "detected" },
            { id: "clangd", status: "detected" },
            { id: "gxx", status: "detected" },
            { id: "bear", status: "detected" }
        ];
        if (controller.status !== "ok") failures += 1 << 23;
        controller.workspaceKind = "cmake";
        controller.workspaceBuildSystems = ["cmake", "make"];
        controller.toolsList = [
            { id: "cmake", status: "detected" },
            { id: "ninja", status: "detected" },
            { id: "clangd", status: "detected" },
            { id: "gxx", status: "detected" }
        ];
        if (controller.status !== "ok") failures += 1 << 24;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
