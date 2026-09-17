// Embarcados: o que a UI mostra da sonda e o que ela pede (roadmaps/35 §5.7,
// fatia 1 — o fio).
//
// Por que existe: o `probe.list` foi roteado no core em 2026-09-03 e ate'
// 2026-09-11 nenhuma tela o pedia. O `EmbeddedController` e' o consumidor que
// faltava, e a traducao dele — sonda reconhecida, ferramenta ausente, dica,
// saida crua — e' o que separa "nenhuma sonda" (verdade) de "nenhuma sonda"
// com uma plugada (mentira). Sem harness, essa traducao quebra sem compilador
// que reclame — a classe do `cdbStale`.
import QtQuick
import "../../ui/qml/embedded"

Item {
    id: root

    property int consultas: 0
    property int consultasSerial: 0
    property var monitores: []
    property int projetos: 0

    EmbeddedController {
        id: controller

        onListRequested: root.consultas += 1
        onSerialListRequested: root.consultasSerial += 1
        onMonitorRequested: function(device, baud) { root.monitores.push(device + "@" + baud); }
        onProjectRequested: root.projetos += 1
    }

    Component.onCompleted: {
        let failures = 0;

        // Abrir PERGUNTA: sonda e' coisa que se pluga e despluga.
        controller.workspaceRoot = "/tmp/projeto";
        controller.open();
        if (!controller.panelVisible) failures += 1;
        if (root.consultas !== 1) failures += 2;
        if (!controller.busy) failures += 4;

        // Uma sonda reconhecida: a linha diz nome, familia, VID:PID e serial.
        controller.handleProbes(
            [{ name: "STLink V3", vid: "0483", pid: "374e", serial: "0031",
               kind: "ST-LINK" }],
            true, "irrelevante", "");
        if (!controller.probeFound) failures += 8;
        if (controller.busy) failures += 16;
        if (controller.probeSummary(controller.probes[0])
                !== "STLink V3 · ST-LINK · 0483:374e · 0031") failures += 32;

        // Campo ausente NAO vira "undefined" na tela.
        if (controller.probeSummary({ name: "CMSIS-DAP", vid: "0d28", pid: "0204" })
                !== "CMSIS-DAP · 0d28:0204") failures += 64;

        // Nada reconhecido: a dica e a saida crua ficam, e' o que o usuario le.
        controller.refresh();
        if (root.consultas !== 2) failures += 128;
        controller.handleProbes([], true, "No debug probes were found.",
                                "nenhuma sonda conectada.");
        if (controller.probeFound) failures += 256;
        if (controller.hint !== "nenhuma sonda conectada.") failures += 512;
        if (controller.rawOutput !== "No debug probes were found.") failures += 1024;

        // Ferramenta ausente e' outro estado, nao "nenhuma sonda".
        controller.handleProbes([], false, "", "instale o probe-rs");
        if (controller.toolAvailable) failures += 2048;

        // Erro de OUTRO dominio nao acende o erro deste painel.
        controller.refresh();
        controller.handleFailed("git.status", "nada a ver");
        if (controller.errorText !== "") failures += 4096;
        if (!controller.busy) failures += 8192;
        controller.handleFailed("probe.list", "probe-rs saiu com 1");
        if (controller.errorText !== "probe-rs saiu com 1") failures += 16384;
        if (controller.busy) failures += 32768;

        // --- Portas seriais (E1 do integracoes/38 §6) ------------------------
        // Abrir/atualizar pergunta as portas JUNTO com a sonda: sao dois
        // pedidos, e o segundo nao pode depender do primeiro voltar.
        if (root.consultasSerial !== root.consultas) failures += 65536;
        if (!controller.portsBusy) failures += 131072;

        // A porta medida em 2026-09-11: o resumo diz no', produto, VID:PID e
        // driver — e a familia fala do ELO, nunca do chip.
        const cp2102 = {
            device: "/dev/ttyUSB0", kind: "usbUartBridge", vid: "10c4", pid: "ea60",
            product: "CP2102 USB to UART Bridge Controller", driver: "cp210x",
            family: "ponte USB-UART CP210x",
            access: { readableWritable: true, mode: "crw-rw----", group: "dialout" },
            modemManager: { candidate: true, ignored: false, running: true }
        };
        controller.handleSerialPorts([cp2102], "");
        if (!controller.portFound) failures += 262144;
        if (controller.portsBusy) failures += 524288;
        if (controller.portSummary(cp2102)
                !== "/dev/ttyUSB0 · CP2102 USB to UART Bridge Controller · 10c4:ea60 · cp210x")
            failures += 1048576;
        // Campo ausente NAO vira "undefined".
        if (controller.portSummary({ device: "/dev/ttyACM0", vid: "303a", pid: "1001" })
                !== "/dev/ttyACM0 · 303a:1001") failures += 2097152;

        // O aviso do ModemManager exige as TRES condicoes: vivo, candidata, sem
        // regra de ignorar. Estado desconhecido nao acusa.
        if (!controller.modemManagerWarns(cp2102)) failures += 4194304;
        if (controller.modemManagerWarns({ modemManager: { candidate: true, ignored: true, running: true } }))
            failures += 8388608;
        if (controller.modemManagerWarns({ modemManager: { candidate: true, ignored: false, running: false } }))
            failures += 16777216;
        if (controller.modemManagerWarns({ device: "/dev/ttyACM0" })) failures += 33554432;

        // O monitor e' PEDIDO (baud 0 = o padrao do core), nunca rodado aqui;
        // e a recusa do serial.monitor acende o erro deste painel.
        controller.openMonitor("/dev/ttyUSB0");
        if (root.monitores.join(",") !== "/dev/ttyUSB0@0") failures += 4294967296;
        controller.handleFailed("serial.monitor", "nenhum monitor serial nesta maquina");
        if (controller.errorText !== "nenhum monitor serial nesta maquina") failures += 8589934592;

        // --- O modelo do projeto (pilar 0 do roadmaps/42) --------------------
        // Abrir pergunta o modelo junto com sonda e portas.
        if (root.projetos !== root.consultas) failures += 17179869184;
        controller.handleProject({ embedded: true,
            frameworks: [{ framework: "espIdf", evidence: "CMakeLists.txt", detail: "IDF_TARGET esp32c3" }],
            sdks: [{ id: "esp-idf", label: "ESP-IDF", found: false, hint: "install.sh" },
                   { id: "esptool", label: "esptool", found: true, path: "/x/esptool" }],
            target: { chip: "esp32c3", family: "espressif", flashEngine: "esptool", monitor: "espflash",
                      debugAdapter: "probe-rs", evidence: ["chip: CMakeLists.txt (CONFIG_IDF_TARGET)"] },
            hints: ["falta ESP-IDF: install.sh"] });
        if (!controller.projectEmbedded || controller.projectBusy) failures += 34359738368;
        if (controller.frameworkSummary(controller.projectFrameworks[0])
                !== "ESP-IDF · CMakeLists.txt · IDF_TARGET esp32c3") failures += 68719476736;
        if (controller.targetSummary(controller.projectTarget)
                !== "esp32c3 · espressif · gravar: esptool, monitor: espflash, debug: probe-rs") failures += 137438953472;
        // Campo ausente nao vira "undefined": um alvo vazio e' uma linha vazia.
        if (controller.targetSummary({}) !== "") failures += 274877906944;
        // A receita e as particoes LIDAS aparecem; a app e' a "flash" do ESP32.
        if (controller.artifactsSummary({ elf: ["a.elf"],
                flashRecipe: { files: [{}, {}, {}], flashSize: "4MB", flashMode: "dio" },
                partitions: { entries: [{ kind: "data", size: 24576, offset: 36864 },
                                        { kind: "app", size: 1048576, offset: 65536 }] } })
                !== "1 ELF · receita: 3 imagens, flash 4MB dio · 2 partições, app 1 MB em 0x10000") failures += 2199023255552;
        if (controller.artifactsSummary({}) !== "") failures += 4398046511104;
        // O evento substitui o modelo inteiro (nao mescla).
        controller.handleProject({ embedded: false, frameworks: [], sdks: [], target: {}, hints: [] });
        if (controller.projectEmbedded || controller.projectFrameworks.length !== 0) failures += 549755813888;

        // Lista vazia guarda a dica do core; a falha do serial.list nao apaga a
        // sonda que ja' veio.
        controller.handleSerialPorts([], "nenhuma porta serial USB apareceu.");
        if (controller.portFound) failures += 67108864;
        if (controller.portsHint !== "nenhuma porta serial USB apareceu.") failures += 134217728;
        controller.handleProbes([{ name: "x", vid: "1", pid: "2" }], true, "", "");
        controller.handleFailed("serial.list", "sysfs indisponivel");
        if (controller.errorText !== "sysfs indisponivel") failures += 268435456;
        if (!controller.probeFound) failures += 536870912;

        // --- A porta ESCOLHIDA do Executar (C3 do roadmaps/41, 2026-09-17) ---
        // Nasce vazia: o campo `device` nao vai e o mpremote escolhe.
        const acm0 = { device: "/dev/ttyACM0", vid: "303a", pid: "1001",
                       access: { readableWritable: true, mode: "crw-rw----" } };
        controller.handleSerialPorts([cp2102, acm0], "");
        if (controller.selectedPort !== "") failures += 8796093022208;
        // Escolher e' apontar para uma porta DA LISTA; fora dela, nada muda.
        controller.selectPort("/dev/ttyACM0");
        if (controller.selectedPort !== "/dev/ttyACM0") failures += 17592186044416;
        controller.selectPort("/dev/ttyS99");
        if (controller.selectedPort !== "/dev/ttyACM0") failures += 35184372088832;
        // Trocar de porta substitui; clicar na escolhida desfaz (toggle).
        controller.selectPort("/dev/ttyUSB0");
        if (controller.selectedPort !== "/dev/ttyUSB0") failures += 70368744177664;
        controller.selectPort("/dev/ttyUSB0");
        if (controller.selectedPort !== "") failures += 140737488355328;
        // A lista nova decide: a escolhida continua se ainda esta' la', e cai
        // se a placa foi desplugada — sem `device` fantasma no proximo Executar.
        controller.selectPort("/dev/ttyACM0");
        controller.handleSerialPorts([acm0], "");
        if (controller.selectedPort !== "/dev/ttyACM0") failures += 281474976710656;
        controller.handleSerialPorts([cp2102], "");
        if (controller.selectedPort !== "") failures += 562949953421312;
        // A falha do serial.list nao mexe na escolha (a lista nao mudou).
        controller.selectPort("/dev/ttyUSB0");
        controller.handleFailed("serial.list", "sysfs indisponivel");
        if (controller.selectedPort !== "/dev/ttyUSB0") failures += 1125899906842624;

        // Trocar de workspace FECHA e esquece: a lista da ultima vez e'
        // exatamente o que nao se pode mostrar — sonda E porta, e a escolha.
        controller.handleSerialPorts([cp2102], "");
        controller.selectPort("/dev/ttyUSB0");
        controller.handleProject({ embedded: true, frameworks: [{ framework: "picoSdk", evidence: "CMakeLists.txt" }],
                                   sdks: [], target: {}, hints: [] });
        controller.workspaceRoot = "/tmp/outro";
        if (controller.panelVisible) failures += 1073741824;
        if (controller.probes.length !== 0 || controller.ports.length !== 0) failures += 2147483648;
        if (controller.projectEmbedded) failures += 1099511627776;
        if (controller.selectedPort !== "") failures += 2251799813685248;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
