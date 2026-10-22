pragma ComponentBehavior: Bound
import QtQuick

// Estado apresentado de probe/serial/project: diagnóstico e identidade vêm do
// core, sem abrir porta na descoberta ou deduzir chip por ponte USB.
// selectedPort é intenção do usuário, repassada ao Executar do MicroPython.
// identity/flash/access/files mantêm as operações em seus donos existentes.
// A conexão invalida resultados acionáveis, preservando intenção e navegação.
Item {
    id: root

    property string workspaceRoot: ""
    property bool panelVisible: false
    property var probes: []
    // `true` ate' o core dizer o contrario: a tela nao acusa ferramenta ausente
    // antes de perguntar.
    property bool toolAvailable: true
    property string rawOutput: ""
    property string hint: ""
    property bool busy: false
    property string errorText: ""

    // Tamanho do ELF (build.size): regioes do linker script com uso, e as
    // secoes cruas. `sizeMeasured` separa "ainda nao medi" de "medi e deu zero".
    property var sizeRegions: []
    property var sizeSections: []
    property string sizeTool: ""
    property bool sizeToolAvailable: true
    property bool sizeMeasured: false
    property bool sizeBusy: false

    // Portas seriais (serial.list). `portsHint` e' a dica do core para a lista
    // VAZIA; a dica de PERMISSAO vem dentro de cada porta.
    property var ports: []
    property string portsHint: ""
    property bool portsBusy: false
    property bool portsMeasured: false
    // A porta que o Executar usa (`device` de run.start/run.script). Vazio =
    // nenhuma escolhida: o campo nao vai, e o mpremote pega a primeira que
    // acha. So' aponta para uma porta da LISTA ATUAL — ver handleSerialPorts.
    property string selectedPort: ""

    // O MODELO do projeto (project.model / event.project.changed): o que o
    // projeto E'. A tela so' mostra; quem deduz e' o core, com evidencia.
    property var project: ({})
    property bool projectBusy: false
    readonly property bool projectEmbedded: project.embedded === true
    readonly property var projectFrameworks: project.frameworks !== undefined ? project.frameworks : []
    readonly property bool projectMicroPython: projectFrameworks.length > 0 && projectFrameworks[0].framework === "microPython"
    readonly property var projectPlatformIoInfo: projectFrameworks.find(info => info.framework === "platformIo") || null
    readonly property bool projectPlatformIo: projectPlatformIoInfo !== null
    readonly property bool projectLinux: projectTarget.family === "linux"
    readonly property bool projectManagedLinux: projectLinux && projectFrameworks.length > 0
        && ["yocto", "buildroot"].indexOf(projectFrameworks[0].framework) >= 0
    readonly property string projectFrameworkLabel: projectFrameworks.length > 0
        ? frameworkSummary(projectFrameworks[0]).split(" · ")[0] : projectLinux ? qsTr("Linux embarcado")
        : projectEmbedded && projectTarget.chip ? qsTr("Bare metal · %1").arg(projectTarget.chip) : ""
    readonly property var projectSdks: project.sdks !== undefined ? project.sdks : []
    readonly property var projectHints: project.hints !== undefined ? project.hints : []
    readonly property var projectTarget: project.target !== undefined ? project.target : ({})
    readonly property var projectArtifacts: project.artifacts !== undefined ? project.artifacts : ({})

    readonly property bool probeFound: probes.length > 0
    readonly property bool portFound: ports.length > 0
    // A aba do painel (E3-5): "board" | "project" | "flash" | "kit".
    property string tab: "project"
    onSelectedPortChanged: flashController.invalidate()
    // O veredito do cabecalho: a placa identificada, senao o que ha' no USB.
    readonly property string boardVerdict: identityController.found
        ? qsTr("%1%2 em %3").arg(identityController.errorText !== "" ? qsTr("Identificação parcial: ") : "")
            .arg(identityController.identity.chipDescription || identityController.identity.chip)
            .arg(identityController.device)
        : portFound ? qsTr("%1 porta(s) serial(is); nenhuma placa identificada ainda").arg(ports.length)
        : probeFound ? qsTr("sonda %1").arg(probes[0].name)
        : portsMeasured ? qsTr("Nenhum dispositivo listado") : qsTr("Dispositivos a verificar")

    // A identidade pelo canal (E5) e' dono proprio, filho deste: quem tem o
    // EmbeddedController alcanca `identity` sem propriedade de repasse.
    readonly property alias identity: identityController
    // Gravar como configuracao de execucao (E4): idem, filho.
    readonly property alias flash: flashController
    // Permissao por canal (E2): idem, filho.
    readonly property alias access: accessController
    // Arquivos na placa MicroPython (C2): idem, filho.
    readonly property alias files: filesController

    EmbeddedIdentityController {
        id: identityController

        onIdentityChanged: {
            if (device === root.selectedPort) flashController.invalidate();
        }
    }

    EmbeddedFlashController {
        id: flashController

        frameworks: root.projectFrameworks
        suggestedEngine: root.projectTarget.flashEngine || ""
        workspaceRoot: root.workspaceRoot
    }

    EmbeddedAccessController {
        id: accessController
        onRefreshRequested: { root.errorText = ""; root.portsBusy = true; root.serialListRequested(); }
    }

    EmbeddedFilesController {
        id: filesController
    }

    signal listRequested()
    signal serialListRequested()
    signal projectRequested()
    signal monitorRequested(string device, int baud)
    signal sizeRequested(string program)
    signal windowRequested()
    signal commandRequested(string commandId)

    visible: false

    onWorkspaceRootChanged: {
        probes = [];
        toolAvailable = true;
        rawOutput = "";
        hint = "";
        busy = false;
        errorText = "";
        sizeRegions = [];
        sizeSections = [];
        sizeTool = "";
        sizeToolAvailable = true;
        sizeMeasured = false;
        sizeBusy = false;
        ports = [];
        portsHint = "";
        portsBusy = false;
        portsMeasured = false;
        selectedPort = "";
        identityController.clear();
        flashController.clear();
        accessController.clear();
        filesController.clear();
        project = ({});
        projectBusy = false;
        panelVisible = false;
        tab = "project";
    }

    function open() {
        panelVisible = true;
        refresh();
        windowRequested();
    }

    function close() {
        panelVisible = false;
    }

    // Toda abertura PERGUNTA de novo: sonda e porta sao coisa que se pluga e
    // despluga, e a lista da ultima vez e' exatamente o que nao se pode mostrar.
    function refresh() {
        busy = true;
        portsBusy = true;
        projectBusy = true;
        accessController.clear();
        errorText = "";
        listRequested();
        serialListRequested();
        projectRequested();
    }

    // O modelo chega por evento (abrir workspace, fim de configure/build) e
    // por pedido: os dois caem aqui. Um modelo novo substitui o anterior.
    function handleProject(model) {
        flashController.invalidate();
        project = model === undefined || model === null ? ({}) : model;
        projectBusy = false;
        if (flashController.probeDestination) flashController.requestTargets();
    }

    function refreshProbes() {
        busy = true;
        listRequested();
    }

    // Uma linha por framework: nome, o arquivo que o provou e o detalhe.
    function frameworkSummary(info) {
        const nomes = { espIdf: "ESP-IDF", zephyr: "Zephyr", picoSdk: "pico-sdk", platformIo: "PlatformIO",
                        stm32Cube: "STM32Cube", cargoEmbedded: "Rust embarcado", microPython: "MicroPython",
                        yocto: "Yocto", buildroot: "Buildroot" };
        const partes = [nomes[info.framework] !== undefined ? nomes[info.framework] : String(info.framework)];
        partes.push(info.evidence);
        if (info.detail !== undefined && info.detail !== "") {
            partes.push(info.detail);
        }
        return partes.join(" · ");
    }

    // Os artefatos em uma linha: quantos ELF, a receita de gravacao LIDA
    // (imagens, flash) e a tabela de particoes (a "flash" de um ESP32 e' a
    // particao app, nao o .ld). Vazio quando o build ainda nao existe.
    function artifactsSummary(art) {
        const partes = [];
        if (art.elf !== undefined && art.elf.length > 0) partes.push(art.elf.length + " ELF");
        if (art.uf2 !== undefined && art.uf2.length > 0) partes.push(art.uf2.length + " UF2");
        if (art.flashRecipe !== undefined) {
            const r = art.flashRecipe;
            let receita = r.files.length + " imagens";
            if (r.flashSize !== undefined) receita += ", flash " + r.flashSize;
            if (r.flashMode !== undefined) receita += " " + r.flashMode;
            partes.push("receita: " + receita);
        }
        if (art.partitions !== undefined) {
            const apps = art.partitions.entries.filter(p => p.kind === "app");
            let texto = art.partitions.entries.length + " partições";
            if (apps.length > 0) texto += ", app " + formatSize(apps[0].size) + " em 0x" + apps[0].offset.toString(16);
            partes.push(texto);
        }
        if (art.memoryX !== undefined) partes.push("memory.x");
        return partes.join(" · ");
    }

    // Bytes -> MB/kB, como no painel de containers.
    function formatSize(n) {
        if (n >= 1048576) return (n / 1048576).toFixed(n % 1048576 === 0 ? 0 : 1) + " MB";
        if (n >= 1024) return (n / 1024).toFixed(0) + " kB";
        return n + " B";
    }

    // O alvo em uma linha: chip, familia e os motores sugeridos — cada um
    // com a evidencia disponivel no tooltip do painel.
    function targetSummary(target) {
        const partes = [];
        if (target.chip !== undefined) partes.push(target.chip);
        if (target.family !== undefined) partes.push(target.family);
        if (target.triple !== undefined) partes.push(target.triple);
        const motores = [];
        if (target.flashEngine !== undefined) motores.push("gravar: " + target.flashEngine);
        if (target.monitor !== undefined) motores.push("monitor: " + target.monitor);
        if (target.debugAdapter !== undefined) motores.push("debug: " + target.debugAdapter);
        if (motores.length > 0) partes.push(motores.join(", "));
        return partes.join(" · ");
    }

    // O monitor e' um PROCESSO numa aba de terminal (decisao do autor,
    // 2026-09-11): a IDE so' escolhe a porta e pede; a ferramenta vem do kit
    // (papel serialMonitor) e a aba e' do RuntimeController.
    function openMonitor(device) {
        errorText = "";
        monitorRequested(device, 0);
    }

    // A escolha e' um TOGGLE: clicar na escolhida desfaz. Uma porta que nao
    // esta' na lista nao se escolhe — a lista e' o que o core mediu.
    function selectPort(device) {
        if (device === selectedPort) {
            selectedPort = "";
            return;
        }
        if (indexOfPort(device) >= 0) {
            selectedPort = device;
        }
    }

    function indexOfPort(device) {
        for (let i = 0; i < ports.length; i++) {
            if (ports[i].device === device) {
                return i;
            }
        }
        return -1;
    }

    // Lista nova substitui a anterior; a escolha so' sobrevive se a porta
    // continua la'. Placa desplugada nao deixa um `device` fantasma no
    // proximo Executar — o campo simplesmente deixa de ir.
    function handleSerialPorts(newPorts, newHint) {
        ports = newPorts === undefined ? [] : newPorts;
        portsHint = newHint === undefined ? "" : newHint;
        portsBusy = false;
        portsMeasured = true;
        if (selectedPort !== "" && indexOfPort(selectedPort) < 0) {
            selectedPort = "";
        }
    }

    function handleProbes(newProbes, newToolAvailable, newRawOutput, newHint) {
        probes = newProbes === undefined ? [] : newProbes;
        toolAvailable = newToolAvailable === undefined ? true : newToolAvailable;
        rawOutput = newRawOutput === undefined ? "" : newRawOutput;
        hint = newHint === undefined ? "" : newHint;
        busy = false;
    }

    function handleFailed(method, message) {
        identityController.handleFailed(method, message);
        accessController.handleFailed(method, message);
        filesController.handleFailed(method, message);
        if (method === "cmake.targets.list") flashController.handleTargetsFailed(message);
        const flags = { "build.size": "sizeBusy", "serial.list": "portsBusy",
                        "project.model": "projectBusy", "probe.list": "busy" };
        const flag = flags[method];
        if (flag !== undefined) root[flag] = false;
        if (method === "serial.list") {
            portsMeasured = false;
            ports = [];
        }
        if (flag !== undefined || method === "serial.monitor") errorText = message;
    }

    function handleCoreDisconnected() {
        const message = qsTr("A conexão com o serviço da IDE foi interrompida. Atualize os dispositivos após reconectar; nenhuma ação será repetida automaticamente.");
        busy = false;
        portsBusy = false;
        portsMeasured = false;
        projectBusy = false;
        sizeBusy = false;
        ports = [];
        probes = [];
        project = ({});
        sizeMeasured = false;
        sizeRegions = [];
        sizeSections = [];
        identityController.clear();
        filesController.clear();
        flashController.disconnected();
        accessController.clear(true);
        errorText = message;
    }

    // O core resolve o ELF sozinho (como o debug.start); por isso o programa
    // vai vazio. Sobrescrever nao e' desta fatia.
    function measureSize() {
        sizeBusy = true;
        errorText = "";
        sizeRequested("");
    }

    function handleBuildSize(sections, regions, toolAvailable, tool, rawOutput) {
        sizeSections = sections === undefined ? [] : sections;
        sizeRegions = regions === undefined ? [] : regions;
        sizeToolAvailable = toolAvailable === undefined ? true : toolAvailable;
        sizeTool = tool === undefined ? "" : tool;
        sizeMeasured = true;
        sizeBusy = false;
    }

    // Fracao usada de uma regiao, 0..1. Regiao sem capacidade (nao deveria
    // ocorrer) nao divide por zero.
    function fracaoUsada(region) {
        return region.size > 0 ? region.used / region.size : 0;
    }

    // Uma linha por porta: o no', o produto que o USB declarou, VID:PID e o
    // driver — na ordem em que se confere no `lsusb`/`dmesg`. Campo ausente nao
    // vira "undefined" na tela.
    function portSummary(port) {
        const partes = [port.device];
        if (port.product !== undefined && port.product !== "") {
            partes.push(port.product);
        }
        partes.push(port.vid + ":" + port.pid);
        if (port.driver !== undefined && port.driver !== "") {
            partes.push(port.driver);
        }
        return partes.join(" · ");
    }

    // O aviso do ModemManager so' vale quando ele esta' RODANDO, a porta e'
    // candidata e nenhuma regra ja' o mandou ignorar. Estado desconhecido
    // (sem udevadm) nao acusa nada.
    function modemManagerWarns(port) {
        const mm = port.modemManager;
        return mm !== undefined && mm !== null && mm.running && mm.candidate && !mm.ignored;
    }

    // Uma linha por sonda: nome, familia, VID:PID e serial — o que o probe-rs
    // reportou, na ordem em que se procura no `lsusb`. Campo ausente nao vira
    // "undefined" na tela.
    function probeSummary(probe) {
        const partes = [probe.name];
        if (probe.kind !== undefined && probe.kind !== "") {
            partes.push(probe.kind);
        }
        partes.push(probe.vid + ":" + probe.pid);
        if (probe.serial !== undefined && probe.serial !== "") {
            partes.push(probe.serial);
        }
        return partes.join(" · ");
    }
}
