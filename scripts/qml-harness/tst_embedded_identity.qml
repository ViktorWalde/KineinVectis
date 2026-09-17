import QtQuick
import "../../ui/qml/embedded"

// A identidade pelo canal (E5, 2026-09-17): o EmbeddedIdentityController
// REAL como filho do EmbeddedController real, com o roteador falso.
//
// O que se prova: identificar e' gesto (um por vez, so' porta com nome);
// o desfecho de OUTRA porta nao sobrescreve; o resumo nao mostra
// "undefined"; a recusa do core (sem esptool/permissao) vira erro sem
// resultado; aplicar ao kit emite o chip sugerido e nada mais; trocar de
// workspace esquece tudo.
Item {
    id: root

    property var pedidos: []
    property var chips: []

    EmbeddedController {
        id: controller
    }

    Connections {
        target: controller.identity

        function onIdentifyRequested(device) { root.pedidos.push(device); }
        function onKitChipRequested(chip) { root.chips.push(chip); }
    }

    Component.onCompleted: {
        let failures = 0;
        const id = controller.identity;

        // Nasce vazio: nada foi perguntado, a view nem aparece.
        if (id.device !== "" || id.busy || id.found) failures += 1;

        // Pedir: um por vez, e a porta vazia nao pede.
        id.identify("");
        if (root.pedidos.length !== 0) failures += 2;
        id.identify("/dev/ttyUSB0");
        if (root.pedidos.join(",") !== "/dev/ttyUSB0" || !id.busy || id.device !== "/dev/ttyUSB0") failures += 4;
        id.identify("/dev/ttyACM0");
        if (root.pedidos.length !== 1) failures += 8;
        id.handleStarted("job_3", "esptool --port /dev/ttyUSB0 flash-id");
        if (id.jobId !== "job_3" || id.command.indexOf("flash-id") < 0) failures += 16;

        // Desfecho de OUTRA porta (pedido antigo) e' ignorado.
        id.handleIdentified({ device: "/dev/ttyACM9", success: true, identity: { chip: "esp32" }, raw: "" });
        if (!id.busy || id.found) failures += 32;

        // O desfecho certo: identidade, sugestao, resumo sem "undefined".
        id.handleIdentified({
            device: "/dev/ttyUSB0", success: true, command: "esptool ... flash-id",
            identity: { chip: "esp32c3", chipDescription: "ESP32-C3 (QFN32) (revision v0.4)",
                        features: ["Wi-Fi", "BLE"], flashSize: "4MB", flashSizeBytes: 4194304,
                        mac: "34:b4:72:0a:1b:2c" },
            target: { chip: "esp32c3", family: "espressif", flashEngine: "esptool",
                      monitor: "espflash", debugAdapter: "probe-rs", evidence: ["chip: esptool flash-id"] },
            raw: "Chip type: ..."
        });
        if (id.busy || !id.found || id.errorText !== "") failures += 64;
        if (id.summary(id.identity) !== "ESP32-C3 (QFN32) (revision v0.4) · flash 4MB · MAC 34:b4:72:0a:1b:2c") failures += 128;
        if (id.summary({ chip: "esp32" }) !== "esp32") failures += 256;
        if (id.targetSummary(id.target) !== "chip esp32c3 · espressif · gravar: esptool, monitor: espflash, debug: probe-rs") failures += 512;
        if (id.targetSummary({}) !== "") failures += 1024;

        // Aplicar ao kit: o chip sugerido, uma vez; sem sugestao, nada.
        id.applyToKit();
        if (root.chips.join(",") !== "esp32c3") failures += 2048;

        // Falha do esptool: erro com a mensagem, sem identidade nem sugestao,
        // e a saida crua fica para a tela.
        id.identify("/dev/ttyUSB0");
        id.handleIdentified({ device: "/dev/ttyUSB0", success: false, error: "A fatal error occurred: Failed to connect",
                              raw: "Connecting......\nA fatal error occurred: Failed to connect" });
        if (id.busy || id.found || id.errorText !== "A fatal error occurred: Failed to connect") failures += 4096;
        if (id.target.chip !== undefined || id.rawOutput.indexOf("Connecting") < 0) failures += 8192;
        id.applyToKit();
        if (root.chips.length !== 1) failures += 16384;

        // A recusa ANTES de abrir a porta (sem esptool): erro, e a de outro
        // metodo nao mexe.
        id.identify("/dev/ttyUSB0");
        id.handleFailed("serial.monitor", "outro");
        if (!id.busy) failures += 32768;
        id.handleFailed("serial.identify", "esptool nao esta' nesta maquina: pipx install esptool");
        if (id.busy || id.errorText.indexOf("pipx install esptool") < 0) failures += 65536;

        // Trocar de workspace esquece tudo (o pai limpa o filho).
        id.identify("/dev/ttyUSB0");
        controller.workspaceRoot = "/tmp/outro";
        if (id.device !== "" || id.busy || id.errorText !== "") failures += 131072;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
