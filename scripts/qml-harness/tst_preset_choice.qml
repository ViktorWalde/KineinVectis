// 0.4.0b (roadmaps/52 §14.2): com varios presets e nada anotado, a faixa de
// saude PERGUNTA em vez de configurar sozinha. O ProjectHealthController e a
// ProjectHealthBanner REAIS: nada de configure automatico, a faixa com o botao
// "Escolher preset…", o menu com um item por preset, e o ponto do menu logo
// abaixo do botao, no espaco pedido.
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

    Timer {
        interval: 50
        running: true
        onTriggered: {
            let failures = 0;
            // Nao configurado, dois presets: nada de configure sozinho.
            controller.handleCmakeStatus(false, false, "", "", ["debug", "release"], "CMakePresets.json");
            failures += check(root.configureRequests === 0, "com escolhas, o configure automatico espera");
            failures += check(controller.actionTarget === "cmakePresetChoice", "o alvo e' a escolha: " + controller.actionTarget);
            failures += check(controller.message.indexOf("2 presets") >= 0
                              && controller.message.indexOf("CMakePresets.json") >= 0, "a faixa diz quantos e de onde: " + controller.message);
            failures += check(controller.actionLabel === "Escolher preset…", "o botao: " + controller.actionLabel);
            const items = controller.presetMenuItems();
            failures += check(items.length === 2 && items[0].action === "cmake.preset:debug"
                              && items[1].label === "release" && items[1].enabled === true, "um item por preset");
            // O botao pede o menu logo abaixo dele, no espaco de quem pediu.
            const button = banner.children.find(c => c.text === "Escolher preset…");
            failures += check(button !== undefined, "o botao existe na faixa");
            if (button !== undefined) {
                button.clicked();
                failures += check(root.menuPoint !== null && root.menuPoint.y > banner.y, "o ponto abaixo do botao: " + JSON.stringify(root.menuPoint));
            }
            // Configurado e com a CDB velha, ainda com escolhas: a faixa pergunta
            // (o reconfigure seria recusado); sem escolhas, o aviso de sempre.
            controller.handleCmakeStatus(true, true, "CMakeLists.txt", "", ["debug", "release"], "CMakePresets.json");
            failures += check(controller.actionTarget === "cmakePresetChoice", "CDB velha com escolhas: pergunta");
            controller.handleCmakeStatus(true, true, "CMakeLists.txt", "release", [], "");
            failures += check(controller.actionTarget === "cmakeConfigure", "anotado: o reconfigure de sempre: " + controller.actionTarget);
            // Nao configurado e sem escolhas: o configure automatico roda.
            controller.handleCmakeStatus(false, false, "", "", [], "");
            failures += check(root.configureRequests === 1, "um preset so': configura sozinho");
            Qt.exit(failures === 0 ? 0 : 1);
        }
    }
}
