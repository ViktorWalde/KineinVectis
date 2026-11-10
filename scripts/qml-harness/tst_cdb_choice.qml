// C-05 (consolidação C §16): várias configurações de presets com
// compile_commands.json e nenhuma escolhida. O ProjectHealthController e a
// ProjectHealthBanner REAIS: a faixa pergunta com o motivo do core, o menu é
// o de configuração do C1-23 (um item por preset, com a pasta, e
// "Automático"), e nada configura sozinho enquanto há escolha pendente ou uma
// CDB de outra configuração em uso.
import QtQuick
import "../../ui/qml/workspace"
import "../../ui/qml/shell"

Item {
    id: root
    width: 900
    height: 200

    property int configureRequests: 0
    property var menuPoint: null

    ProjectHealthController {
        id: controller

        workspaceRoot: "/tmp/proj"
        workspaceTrusted: true
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

    ProjectHealthBanner {
        id: banner

        y: 40
        width: parent.width
        menuSpace: root
        active: controller.active
        status: controller.status
        message: controller.message
        actionLabel: controller.actionLabel
        onActionRequested: (x, y) => root.menuPoint = { x: x, y: y }
    }

    function check(condition, label) {
        if (!condition) {
            console.error("FALHOU: " + label);
            return 1;
        }
        return 0;
    }

    readonly property string reason: "2 configurações de presets têm compile_commands.json (gcc-debug, clang-release): escolha uma"
    readonly property var candidates: [
        { directory: "build/gcc-debug", origin: "preset", preset: "gcc-debug" },
        { directory: "/tmp/proj saídas/clang release", origin: "preset", preset: "clang-release" }
    ]
    // O que o core manda junto (0.210.0): vários presets e nenhuma escolha.
    readonly property var several: ({
        refused: "o projeto tem 2 presets de configure em CMakePresets.json (gcc-debug, clang-release): escolha um",
        choices: [
            { preset: "gcc-debug", buildDir: "build/gcc-debug", configured: true },
            { preset: "clang-release", buildDir: "/tmp/proj saídas/clang release", configured: true }
        ]
    })

    Timer {
        interval: 50
        running: true
        onTriggered: {
            let failures = 0;
            // Escolha pendente, sem configure da IDE: pergunta, não configura.
            controller.handleCmakeStatus(false, false, "", "", ["gcc-debug", "clang-release"],
                                         "CMakePresets.json",
                                         { choiceRequired: true, absentBecause: root.reason,
                                           candidates: root.candidates }, root.several);
            failures += check(root.configureRequests === 0, "escolha pendente: o configure automatico espera");
            failures += check(controller.actionTarget === "cmakeConfigurationChoice", "o alvo e' a escolha: " + controller.actionTarget);
            failures += check(controller.message === root.reason, "a faixa diz o motivo do core: " + controller.message);
            failures += check(controller.actionLabel === "Escolher configuração…", "o botao: " + controller.actionLabel);
            const items = controller.menuItems();
            failures += check(items.length === 4, "um item por preset, o separador e o automatico: " + items.length);
            failures += check(items.length === 4 && items[0].action === "cmake.configuration:gcc-debug"
                              && items[0].label === "gcc-debug" && items[0].detail === "build/gcc-debug"
                              && items[1].action === "cmake.configuration:clang-release"
                              && items[1].detail === "/tmp/proj saídas/clang release"
                              && items[1].tooltip === "/tmp/proj saídas/clang release"
                              && items[1].enabled === true && items[2].separator === true
                              && items[3].action === "cmake.configuration:",
                              "pasta e preset em cada item: " + JSON.stringify(items));
            failures += check(items.every(item => (item.icon || "") === ""),
                              "nenhuma escolhida: nada marcado: " + JSON.stringify(items));
            const button = banner.children.find(c => c.text === "Escolher configuração…");
            failures += check(button !== undefined, "o botao existe na faixa");
            if (button !== undefined) {
                button.clicked();
                failures += check(root.menuPoint !== null && root.menuPoint.y > banner.y, "o ponto abaixo do botao: " + JSON.stringify(root.menuPoint));
            }
            // A escolha de preset do configure (0.4.0b) continua com o seu menu.
            controller.handleCmakeStatus(false, false, "", "", ["debug", "release"], "CMakePresets.json", {});
            failures += check(controller.actionTarget === "cmakePresetChoice"
                              && controller.menuItems()[0].action === "cmake.preset:debug", "o menu de preset: " + controller.actionTarget);
            // A CDB de um preset em uso, sem configure da IDE: análise ok, sem
            // reconfigurar em `.kinein/build` nem aviso de modo degradado.
            controller.handleCmakeStatus(false, false, "", "", [], "",
                                         { directory: "build/gcc-debug", origin: "preset", preset: "gcc-debug" });
            failures += check(root.configureRequests === 0, "CDB em uso: nada de configure automatico");
            failures += check(controller.status === "ok", "CDB em uso: a faixa nao avisa: " + controller.status + " " + controller.message);
            failures += check(controller.menuItems().length === 0, "sem menu fora das escolhas");
            // A CDB escolhida velha: o aviso de sempre, com o culpado.
            controller.handleCmakeStatus(false, true, "CMakeLists.txt", "", [], "",
                                         { directory: "build/gcc-debug", origin: "configuration", preset: "gcc-debug",
                                           selectedConfiguration: "gcc-debug" });
            failures += check(controller.actionTarget === "cmakeConfigure"
                              && controller.message.indexOf("CMakeLists.txt") >= 0, "CDB escolhida velha: " + controller.message);
            // Sem CDB e sem escolha (o mapa ausente das versões antigas): o
            // configure automatico roda uma vez.
            controller.handleCmakeStatus(false, false, "", "", [], "");
            failures += check(root.configureRequests === 1, "sem CDB: configura sozinho");
            Qt.exit(failures === 0 ? 0 : 1);
        }
    }
}
