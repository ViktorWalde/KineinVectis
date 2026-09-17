import QtQuick
import "../../ui/qml/embedded"

// Permissao por canal (E2, 2026-09-17): o EmbeddedAccessController REAL como
// filho do EmbeddedController real, com o roteador e o terminal falsos.
//
// O que se prova: o diagnostico e' pedido (um por vez); os canais chegam
// e a contagem de problemas e' a dos `ok: false`; o rotulo nao mostra
// "undefined"; o passo emite EXATAMENTE o comando (e nada com comando
// vazio); a recusa do core vira erro e a de outro metodo nao mexe; trocar
// de workspace esquece tudo.
Item {
    id: root

    property var pedidos: []
    property var escritos: []

    EmbeddedController {
        id: controller
    }

    Connections {
        target: controller.access

        function onDiagnoseRequested(device) { root.pedidos.push("[" + device + "]"); }
        function onCommandRequested(command) { root.escritos.push(command); }
    }

    Component.onCompleted: {
        let failures = 0;
        const a = controller.access;

        if (a.busy || a.diagnosed || a.problems !== 0) failures += 1;
        a.diagnose("");
        a.diagnose("/dev/ttyUSB0");
        if (root.pedidos.join(",") !== "[]" || !a.busy) failures += 2;

        // Os canais como o core os manda (o caso REAL medido nesta maquina
        // em 2026-09-17: serial ok por uaccess, ModemManager com passo,
        // sondas da distro).
        a.handleChannels([
            { kind: "serial", device: "/dev/ttyUSB0", ok: true,
              detail: "/dev/ttyUSB0 e' crw-rw---- do grupo `dialout`; voce nao faz parte do grupo",
              distroDidIt: "o udev deu acesso ao usuario da sessao pela tag `uaccess`" },
            { kind: "modemManager", device: "/dev/ttyUSB0", ok: false,
              detail: "ModemManager esta' rodando; a porta e' candidata e nao tem regra de ignorar",
              problem: "o ModemManager pode abrir esta porta por alguns segundos",
              fix: { steps: [
                       { explanation: "regra udev", command: "printf 'ATTRS...' | sudo tee /etc/udev/rules.d/77-mm-kinein-10c4-ea60.rules" },
                       { explanation: "recarrega", command: "sudo udevadm control --reload && sudo udevadm trigger" } ],
                     sourceUrl: "/usr/lib/udev/rules.d/80-mm-candidate.rules", checkedOn: "2026-09-17" } },
            { kind: "probe", ok: true, detail: "regras udev de sonda: 60-openocd.rules",
              distroDidIt: "a distro ja' instalou: 60-openocd.rules" }
        ]);
        if (a.busy || !a.diagnosed || a.problems !== 1) failures += 4;
        if (a.channelTitle(a.channels[0]) !== "porta serial · /dev/ttyUSB0") failures += 8;
        if (a.channelTitle(a.channels[2]) !== "sondas (regra udev)") failures += 16;
        if (a.channelTitle({ kind: "outro" }) !== "outro") failures += 32;
        if (a.stepsOf(a.channels[0]).length !== 0 || a.stepsOf(a.channels[1]).length !== 2) failures += 64;

        // O passo escreve EXATAMENTE o comando; vazio nao escreve.
        a.runStep(a.stepsOf(a.channels[1])[1].command);
        a.runStep("");
        if (root.escritos.join(",") !== "sudo udevadm control --reload && sudo udevadm trigger") failures += 128;

        // A recusa: erro, e a de outro metodo nao mexe.
        a.diagnose("");
        a.handleFailed("serial.list", "outro");
        if (!a.busy) failures += 256;
        a.handleFailed("serial.access", "sysfs indisponivel");
        if (a.busy || a.errorText !== "sysfs indisponivel") failures += 512;

        // Trocar de workspace esquece.
        controller.workspaceRoot = "/tmp/outro";
        if (a.diagnosed || a.busy || a.errorText !== "" || a.problems !== 0) failures += 1024;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
