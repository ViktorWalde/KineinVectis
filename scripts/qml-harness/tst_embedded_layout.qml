import QtQuick
import KineinVectis

// Regressão da bancada: ações acessíveis no dock estreito, navegação fixa.
Item {
    id: root
    width: 400
    height: 520
    property int failures: 0
    property var commands: []

    EmbeddedController {
        id: controller
    }
    ToolchainController {
        id: toolchain
    }
    Connections {
        target: controller
        function onCommandRequested(commandId) { root.commands.push(commandId); }
    }
    EmbeddedPanelHost {
        id: panel
        anchors.fill: parent
        controller: controller
        toolchainController: toolchain
    }

    function find(item, name) {
        if (item.objectName === name) return item;
        for (const child of item.children) {
            const result = find(child, name);
            if (result) return result;
        }
        return null;
    }
    function check(ok, message) {
        if (!ok) { root.failures++; console.error(message); }
    }
    function inside(item) {
        const position = item.mapToItem(panel, 0, 0);
        return position.x >= 0 && position.x + item.width <= panel.width;
    }

    function checkEnvironmentAction() {
        const environment = find(panel, "embeddedStage_environment");
        const firmware = find(panel, "embeddedStage_firmware");
        check(environment.enabled && !firmware.enabled, "SDK ausente impede compilar e oferece preparação");
        environment.clicked();
        check(root.commands.join(",") === "setup.list", "Preparação abre o dono do ambiente");
        controller.handleProject({ embedded: true, frameworks: [{ framework: "espIdf", evidence: "CMakeLists.txt" }],
            sdks: [{ id: "esp-idf", label: "ESP-IDF", found: true }] });
        Qt.callLater(root.checkCompileAction);
    }

    function checkCompileAction() {
        const firmware = find(panel, "embeddedStage_firmware");
        check(firmware.enabled, "Ambiente detectado permite compilar sem artefato");
        firmware.clicked();
        check(root.commands.join(",") === "setup.list,build.run", "Compilar chama a orquestração existente");
        controller.handleProject({ embedded: true, frameworks: [{ framework: "espIdf", evidence: "CMakeLists.txt" }],
            artifacts: { elf: ["/tmp/firmware.elf"] } });
        Qt.callLater(root.checkReviewAction);
    }

    function checkReviewAction() {
        find(panel, "embeddedStage_firmware").clicked();
        check(controller.tab === "flash" && root.commands.length === 2, "Artefato leva à revisão, sem gravar");
        controller.tab = "project";
        controller.handleProject({ embedded: true, frameworks: [{ framework: "microPython", evidence: "main.py" }] });
        Qt.callLater(root.checkMicroPythonAction);
    }

    function checkMicroPythonAction() {
        const script = find(panel, "embeddedStage_script");
        check(script && script.enabled && !find(panel, "embeddedStage_firmware"), "MicroPython oferece scripts, sem compilar firmware");
        if (script) script.clicked();
        check(root.commands[2] === "view.returnToEditor", "MicroPython leva à execução por arquivo no editor");
        controller.tab = "kit";
        Qt.callLater(root.checkAdvancedKit);
    }

    function checkAdvancedKit() {
        const target = find(panel, "embeddedTarget");
        const toggle = find(panel, "embeddedAdvancedKit");
        check(!target.visible, "Campos avançados começam recolhidos");
        toggle.clicked();
        kitLayoutCheck.start();
    }

    Timer {
        id: kitLayoutCheck
        interval: 50
        onTriggered: {
            const target = root.find(panel, "embeddedTarget");
            const toggle = root.find(panel, "embeddedAdvancedKit");
            root.check(target.visible && target.mapToItem(panel, 0, 0).y > toggle.mapToItem(panel, 0, 0).y,
                  "Opções avançadas abrem abaixo do controle");
            root.checkNextWorkspace();
        }
    }

    function checkNextWorkspace() {
        controller.workspaceRoot = "/tmp/embedded-next-project";
        Qt.callLater(function() {
            check(controller.panelVisible && controller.portsBusy && controller.projectBusy,
                  "Troca de projeto com dock aberto renova a descoberta");
            Qt.exit(root.failures === 0 ? 0 : 1);
        });
    }

    Component.onCompleted: {
        controller.handleProject({ embedded: true, frameworks: [{ framework: "espIdf", evidence: "CMakeLists.txt" }] });
        controller.handleSerialPorts([{ device: "/dev/ttyUSB0", vid: "10c4", pid: "ea60", product: "CP2102", driver: "cp210x",
            access: { readableWritable: true }, family: "Espressif" }], "");
        controller.tab = "board";
        controller.selectedPort = "/dev/ttyUSB0";
        Qt.callLater(function() {
            const identify = find(panel, "embeddedIdentify");
            const monitor = find(panel, "embeddedMonitor");
            const select = find(panel, "embeddedSelectPort");
            const files = find(panel, "embeddedFiles");
            check(identify && monitor && select, "Ações da porta presentes");
            check(inside(identify) && inside(monitor) && inside(select), "Ações dentro da largura do dock");
            check(!files.visible, "ESP-IDF não oferece arquivos MicroPython");
            check(monitor.y > select.y || identify.y > select.y, "Ações quebram linha na largura reduzida");
            const body = find(panel, "embeddedBody");
            const navigation = find(panel, "embeddedNavigation");
            const navY = navigation.mapToItem(panel, 0, 0).y;
            body.contentY = 60;
            check(navigation.mapToItem(panel, 0, 0).y === navY, "Navegação fixa ao rolar");
            controller.tab = "flash";
            Qt.callLater(function() {
                check(body.contentY === 0, "Nova aba começa no topo");
                check(navigation.width <= panel.width, "Abas cabem no dock");
                controller.tab = "project";
                controller.handleProject({ embedded: true, frameworks: [{ framework: "espIdf", evidence: "CMakeLists.txt" }],
                    sdks: [{ id: "esp-idf", label: "ESP-IDF", found: false }] });
                Qt.callLater(root.checkEnvironmentAction);
            });
        });
    }
}
