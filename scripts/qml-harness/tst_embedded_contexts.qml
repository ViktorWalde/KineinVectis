import QtQuick
import KineinVectis

Item {
    id: root
    width: 400
    height: 520
    property int failures: 0
    property int proposals: 0
    property var commands: []
    TopHeaderBar {
        id: header
        width: 1400
        visible: false
        workspaceOpen: true
        workspaceKind: "cmake"
        workspaceBuildSystems: ["cmake"]
    }
    EmbeddedController {
        id: controller
    }
    ToolchainController {
        id: toolchain
    }
    EmbeddedPanelHost {
        id: panel
        anchors.fill: parent
        controller: controller
        toolchainController: toolchain
    }
    Connections {
        target: controller
        function onCommandRequested(commandId) { root.commands.push(commandId); }
    }
    Connections {
        target: controller.flash
        function onProposalRequested() { root.proposals++; }
        function onRunRequested() { root.proposals++; }
        function onSaveRequested() { root.proposals++; }
    }
    function find(item, name) {
        if (item.objectName === name) return item;
        for (const child of item.children) {
            const found = find(child, name);
            if (found) return found;
        }
        return null;
    }
    function findRole(item, key) {
        if (item.roleKey === key) return item;
        for (const child of item.children) {
            const found = findRole(child, key);
            if (found) return found;
        }
        return null;
    }
    function check(ok, message) {
        if (ok) return;
        root.failures++;
        console.error(message);
    }
    function platformIo(available) {
        controller.handleProject({ embedded: true,
            frameworks: [{ framework: "platformIo", evidence: "platformio.ini", detail: "bench (native)" },
                         { framework: "microPython", evidence: "examples/main.py" }],
            sdks: [{ id: "pio", found: available }], target: {} });
        controller.tab = "project";
    }
    function countItems(item) {
        let count = 1;
        for (const child of item.children) count += countItems(child);
        return count;
    }
    function checkPlatformIo() {
        console.log("initial_visual_items", root.countItems(panel));
        const loader = find(panel, "embeddedEnvironmentLoader");
        if (loader) check(loader.item === null, "Ambiente ainda não criado antes do primeiro acesso");
        check(controller.projectPlatformIo && !controller.projectMicroPython, "PlatformIO não muda de runtime por um exemplo MicroPython");
        check(controller.projectFrameworkLabel === "PlatformIO", "Nome principal do contexto PlatformIO");
        if (header.projectFrameworkLabel !== undefined) header.projectFrameworkLabel = controller.projectFrameworkLabel;
        check(header.workspaceSystemLabel() === "PlatformIO", "Cabeçalho mostra o wrapper em vez do CMake interno");
        const build = find(panel, "embeddedPlatformIoBuild");
        check(build !== null, "Destaque PlatformIO presente no painel");
        if (!build) { Qt.exit(1); return; }
        build.clicked();
        check(root.commands.join(",") === "build.run", "Compilar PlatformIO chega ao dono de build");
        find(panel, "embeddedPlatformIoUpload").clicked();
        check(controller.tab === "flash" && controller.flash.engine === "platformio", "Upload abre a revisão no motor correto");
        check(root.proposals === 0, "Abrir revisão não grava nem gera proposta implicitamente");
        controller.tab = "kit";
        Qt.callLater(root.checkPlatformIoKit);
    }
    function checkPlatformIoKit() {
        check(!findRole(panel, "cCompiler"), "PlatformIO gerencia compiladores sem oferecer o seletor CMake");
        root.platformIo(false);
        Qt.callLater(root.checkMissingPlatformIo);
    }
    function checkMissingPlatformIo() {
        check(!find(panel, "embeddedPlatformIoUpload").enabled, "Sem ferramenta, upload indisponível");
        find(panel, "embeddedPlatformIoBuild").clicked();
        check(root.commands[1] === "setup.list", "Sem PlatformIO, próxima ação prepara ferramenta");
        controller.handleProject({ embedded: true, frameworks: [{ framework: "yocto", evidence: "conf/local.conf" }],
            target: { family: "linux" }, sdks: [] });
        controller.handleSerialPorts([{ device: "/dev/ttyUSB0", access: { readableWritable: true } }], "");
        Qt.callLater(root.checkLinux);
    }
    function checkLinux() {
        const navigation = find(panel, "embeddedNavigation");
        check(navigation.options[1].label === "Conexões" && navigation.options[2].label === "Implantar", "Linux usa rótulos de conexão e implantação");
        check(!find(panel, "embeddedStage_firmware"), "Linux não exige firmware/porta de microcontrolador");
        const remote = find(panel, "embeddedLinuxRemote");
        check(remote && remote.enabled && remote.visible, "Linux oferece acesso ao alvo SSH");
        remote.clicked();
        check(root.commands[2] === "remote.list" && root.proposals === 0, "SSH navega para o dono sem implantar");
        controller.tab = "board";
        Qt.callLater(root.checkLinuxConsole);
    }
    function checkLinuxConsole() {
        check(!find(panel, "embeddedIdentify").visible, "Console Linux não oferece identificação ROM pelo esptool");
        check(find(panel, "embeddedMonitor").visible && !find(panel, "embeddedFiles").visible,
            "Console Linux mantém monitor e dispensa arquivos mpremote");
        controller.tab = "kit";
        Qt.callLater(root.checkLinuxKit);
    }
    function checkLinuxKit() {
        check(find(panel, "embeddedTarget").visible, "Alvo do SDK Linux acessível sem abrir depuração avançada");
        check(!findRole(panel, "cCompiler"), "Imagem Linux gerencia seus compiladores");
        controller.handleProject({ embedded: true, frameworks: [], target: { family: "linux", triple: "aarch64-linux-gnu" } });
        toolchain.targetTriple = "aarch64-linux-gnu";
        toolchain.handleAdvice("", [], true);
        Qt.callLater(root.checkLinuxApplicationKit);
    }
    function checkLinuxApplicationKit() {
        check(findRole(panel, "cCompiler") !== null, "Aplicação Linux CMake mantém escolha do compilador do kit");
        const rustHint = find(panel, "embeddedRustTargetHint");
        check(rustHint && rustHint.text === "", "Aplicação CMake não pede instalar um alvo Rust");
        check(controller.projectFrameworkLabel === "Linux embarcado", "Aplicação sem framework mantém nome do contexto Linux");
        controller.handleProject({ embedded: true, frameworks: [{ framework: "buildroot", evidence: ".config" }],
            target: { family: "linux" } });
        check(controller.projectLinux, "Buildroot usa a jornada Linux");
        controller.handleProject({ embedded: true, frameworks: [{ framework: "espIdf", evidence: "CMakeLists.txt" },
            { framework: "microPython", evidence: "examples/main.py" }] });
        check(!controller.projectMicroPython, "ESP-IDF não vira MicroPython por um exemplo aninhado");
        const loader = find(panel, "embeddedEnvironmentLoader");
        const original = loader.item;
        const target = find(panel, "embeddedTarget");
        const input = target.children.find(child => child.placeholder !== undefined);
        input.text = "draft-target";
        controller.tab = "project";
        controller.tab = "kit";
        check(loader.item === original && input.text === "draft-target", "Trocar de aba conserva a instância e o rascunho");
        controller.workspaceRoot = "/tmp/embedded-context-next";
        Qt.callLater(root.checkWorkspaceReset);
    }
    function checkWorkspaceReset() {
        check(find(panel, "embeddedEnvironmentLoader").item === null, "Outro workspace descarta o formulário anterior");
        if (root.failures) console.error("FALHAS", root.failures);
        Qt.exit(root.failures === 0 ? 0 : 1);
    }
    Component.onCompleted: {
        root.platformIo(true);
        Qt.callLater(root.checkPlatformIo);
    }
}
