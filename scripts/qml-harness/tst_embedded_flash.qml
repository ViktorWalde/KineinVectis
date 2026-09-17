import QtQuick
import "../../ui/qml/embedded"

// Gravar como configuracao de execucao (E4, 2026-09-17): o
// EmbeddedFlashController REAL como filho do EmbeddedController real, com
// o roteador e os donos de execucao falsos.
//
// O que se prova: a previa e' PEDIDA com a porta, o motor escolhido e a
// flash lida (um pedido por vez); trocar o motor descarta a previa; a
// recusa do core vira erro sem previa e a de outro metodo nao mexe; "Gravar
// agora" emite EXATAMENTE a linha proposta e "Salvar" emite nome+linha —
// nada sem previa; trocar de workspace esquece tudo.
Item {
    id: root

    property var pedidos: []
    property var rodou: []
    property var salvou: []

    EmbeddedController {
        id: controller
    }

    Connections {
        target: controller.flash

        function onProposalRequested(device, engine, flashSizeBytes, firmware) {
            root.pedidos.push(device + "|" + engine + "|" + flashSizeBytes + (firmware !== "" ? "|fw=" + firmware : ""));
        }
        function onRunRequested(command) { root.rodou.push(command); }
        function onSaveRequested(name, command) { root.salvou.push(name + "=" + command); }
    }

    Component.onCompleted: {
        let failures = 0;
        const f = controller.flash;

        // Nasce vazio; sem previa nao roda nem salva.
        if (f.busy || f.found || f.engine !== "") failures += 1;
        f.run(); f.save();
        if (root.rodou.length !== 0 || root.salvou.length !== 0) failures += 2;

        // A previa e' pedida com porta, motor (vazio = o modelo decide) e flash.
        f.propose("/dev/ttyUSB0", 4194304);
        if (root.pedidos.join(",") !== "/dev/ttyUSB0||4194304" || !f.busy) failures += 4;
        f.propose("/dev/ttyUSB0", 0);
        if (root.pedidos.length !== 1) failures += 8;

        // A previa chega: comando, nome, evidencias e avisos sem "undefined".
        f.handleProposal({ name: "Gravar (esptool)", engine: "esptool",
                           command: "'/x/esptool' --chip esp32c3 --port '/dev/ttyUSB0' write-flash 0x0 '/p/a.bin'",
                           source: ["motor: esptool (sugerido pelo modelo do projeto)", "porta: /dev/ttyUSB0"],
                           warnings: [] });
        if (f.busy || !f.found || f.errorText !== "") failures += 16;
        if (f.sourceLines().length !== 2 || f.warningLines().length !== 0) failures += 32;
        f.handleProposal({ name: "Gravar (esptool)", engine: "esptool", command: "x" });
        if (f.sourceLines().length !== 0 || f.warningLines().length !== 0) failures += 64;

        // Gravar agora = a linha proposta, exata; salvar = nome + linha.
        f.handleProposal({ name: "Gravar (esptool)", engine: "esptool", command: "'/x/esptool' write-flash 0x0 '/p/o meu app.bin'" });
        f.run();
        f.save();
        if (root.rodou.join(",") !== "'/x/esptool' write-flash 0x0 '/p/o meu app.bin'") failures += 128;
        if (root.salvou.join(",") !== "Gravar (esptool)='/x/esptool' write-flash 0x0 '/p/o meu app.bin'") failures += 256;

        // Trocar o motor descarta a previa (era de outro motor) e vai no pedido;
        // clicar de novo solta (o modelo decide).
        f.selectEngine("probe-rs");
        if (f.found || f.engine !== "probe-rs") failures += 512;
        f.propose("", 0);
        if (root.pedidos[1] !== "|probe-rs|0") failures += 1024;
        f.handleProposal({ name: "Gravar (probe-rs)", engine: "probe-rs", command: "'/x/probe-rs' download --chip X '/p/fw'" });
        f.selectEngine("probe-rs");
        if (f.engine !== "" || f.found) failures += 2048;

        // A recusa do core: erro, sem previa; outro metodo nao mexe.
        f.propose("/dev/ttyUSB0", 0);
        f.handleFailed("serial.identify", "outro");
        if (!f.busy) failures += 4096;
        f.handleFailed("runConfig.flashProposal", "sem receita de gravacao: compile o projeto primeiro");
        if (f.busy || f.found || f.errorText.indexOf("compile") < 0) failures += 8192;
        f.run();
        if (root.rodou.length !== 1) failures += 16384;

        // O firmware baixado (C5): toggle, solta o motor (a pagina fixa o
        // motor), descarta a previa, e vai no pedido; escolher de novo desfaz.
        f.selectEngine("esptool");
        f.handleProposal({ name: "Gravar (esptool)", engine: "esptool", command: "e" });
        f.selectFirmware("micropython-esp32-generic");
        if (f.firmware !== "micropython-esp32-generic" || f.engine !== "" || f.found) failures += 65536;
        f.propose("/dev/ttyUSB0", 0);
        if (root.pedidos[root.pedidos.length - 1] !== "/dev/ttyUSB0||0|fw=micropython-esp32-generic") failures += 131072;
        f.handleProposal({ name: "Gravar firmware (MicroPython — ESP32_GENERIC)", engine: "esptool", command: "x" });
        f.selectFirmware("micropython-esp32-generic");
        if (f.firmware !== "" || f.found) failures += 262144;

        // Trocar de workspace esquece tudo (o pai limpa o filho).
        f.selectEngine("picotool");
        f.selectFirmware("micropython-rpi-pico");
        f.handleProposal({ name: "Gravar (picotool)", engine: "picotool", command: "p" });
        controller.workspaceRoot = "/tmp/outro";
        if (f.found || f.engine !== "" || f.firmware !== "" || f.errorText !== "" || f.busy) failures += 32768;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
