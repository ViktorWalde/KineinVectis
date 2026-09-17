import QtQuick
import "../../ui/qml/embedded"

// Os arquivos na placa (C2, 2026-09-17): o EmbeddedFilesController REAL como
// filho do EmbeddedController real, com o roteador falso.
//
// O que se prova: listar e' gesto (um por vez; trocar de porta esquece a
// outra); navegar junta o caminho; o desfecho de OUTRA porta e' ignorado;
// a lista chega sem "undefined"; baixar vai para `placa/<caminho>` e abre o
// arquivo; o que ESCREVE (enviar/apagar/criar) so' sai depois do segundo
// clique e relista ao terminar; a recusa do core vira erro; trocar de
// workspace esquece tudo.
Item {
    id: root

    property var pedidos: []
    property var abertos: []
    property int enviosDoEditor: 0

    EmbeddedController {
        id: controller
    }

    Connections {
        target: controller.files

        function onFilesRequested(device, action, path, local) {
            root.pedidos.push(action + " " + device + " :" + path + " " + local);
        }
        function onOpenLocalRequested(local) { root.abertos.push(local); }
        function onUploadCurrentRequested() { root.enviosDoEditor += 1; }
    }

    Component.onCompleted: {
        let failures = 0;
        const f = controller.files;

        // Nasce vazio: nada foi perguntado, a view nem aparece.
        if (f.device !== "" || f.busy || f.listed || f.hasPending) failures += 1;

        // Listar: um por vez; porta vazia nao pede.
        f.list("", "");
        if (root.pedidos.length !== 0) failures += 2;
        f.list("/dev/ttyUSB0", "");
        if (root.pedidos.join("|") !== "list /dev/ttyUSB0 : " || !f.busy) failures += 4;
        f.list("/dev/ttyUSB0", "lib");
        if (root.pedidos.length !== 1) failures += 8;
        f.handleStarted("job_1", "mpremote connect /dev/ttyUSB0 fs ls :");
        if (f.jobId !== "job_1" || f.command.indexOf("fs ls") < 0) failures += 16;

        // Desfecho de OUTRA porta e' ignorado; o certo lista sem "undefined".
        f.handleOutcome({ device: "/dev/ttyACM9", action: "list", success: true, entries: [{ name: "x", size: 1, directory: false }], raw: "" });
        if (!f.busy || f.listed) failures += 32;
        f.handleOutcome({ device: "/dev/ttyUSB0", action: "list", path: "", success: true,
                          entries: [{ name: "boot.py", size: 139, directory: false }, { name: "lib", size: 0, directory: true }],
                          raw: "ls :\n         139 boot.py\n           0 lib/\n" });
        if (f.busy || !f.listed || f.entries.length !== 2 || f.errorText !== "") failures += 64;
        if (f.entrySummary(f.entries[0]) !== "boot.py  139 B" || f.entrySummary(f.entries[1]) !== "lib/") failures += 128;
        if (f.entrySummary({ name: "a" }) !== "a") failures += 256;

        // Navegar junta o caminho; subir volta.
        f.enterDir("lib");
        if (root.pedidos[1] !== "list /dev/ttyUSB0 :lib " || f.path !== "lib") failures += 512;
        f.handleOutcome({ device: "/dev/ttyUSB0", action: "list", success: true, entries: [], raw: "" });
        f.enterDir("sub");
        f.handleOutcome({ device: "/dev/ttyUSB0", action: "list", success: true, entries: [], raw: "" });
        if (f.path !== "lib/sub") failures += 1024;
        f.up();
        if (f.path !== "lib" || root.pedidos.length !== 4) failures += 2048;
        f.handleOutcome({ device: "/dev/ttyUSB0", action: "list", success: true, entries: [], raw: "" });

        // Baixar: `placa/<caminho>` relativo; ao chegar, abre o local resolvido.
        f.download("wifi.py");
        if (root.pedidos[4] !== "get /dev/ttyUSB0 :lib/wifi.py placa/lib/wifi.py") failures += 4096;
        f.handleOutcome({ device: "/dev/ttyUSB0", action: "get", path: "lib/wifi.py", success: true, local: "/w/placa/lib/wifi.py", raw: "" });
        if (root.abertos.join(",") !== "/w/placa/lib/wifi.py" || f.busy) failures += 8192;

        // Enviar: pendente ate' confirmar; o nome vem do arquivo; cancelar esquece.
        f.upload("", "");
        if (f.hasPending || f.errorText === "") failures += 16384;
        f.upload("/w/main.py", "");
        if (!f.hasPending || f.pending.path !== "lib/main.py" || root.pedidos.length !== 5) failures += 32768;
        if (f.pendingText().indexOf("undefined") >= 0 || f.pendingText().indexOf("/w/main.py") < 0) failures += 65536;
        f.cancelPending();
        if (f.hasPending || root.pedidos.length !== 5) failures += 131072;
        f.upload("/w/main.py", "");
        f.confirmPending();
        if (root.pedidos[5] !== "put /dev/ttyUSB0 :lib/main.py /w/main.py" || f.hasPending || !f.busy) failures += 262144;
        // Escreveu: relista a pasta atual.
        f.handleOutcome({ device: "/dev/ttyUSB0", action: "put", path: "lib/main.py", success: true, raw: "" });
        if (root.pedidos[6] !== "list /dev/ttyUSB0 :lib " || !f.busy) failures += 524288;
        f.handleOutcome({ device: "/dev/ttyUSB0", action: "list", success: true, entries: [], raw: "" });

        // Apagar e criar pasta: idem, com confirmacao.
        f.remove("old.py");
        if (f.pendingText() !== "Apagar :lib/old.py da placa?") failures += 1048576;
        f.confirmPending();
        if (root.pedidos[7] !== "rm /dev/ttyUSB0 :lib/old.py ") failures += 2097152;
        f.handleOutcome({ device: "/dev/ttyUSB0", action: "rm", success: false, error: "rm: old.py: No such file or directory.", raw: "" });
        if (f.busy || f.errorText !== "rm: old.py: No such file or directory." || root.pedidos.length !== 8) failures += 4194304;
        f.makeDir("  ");
        if (f.hasPending) failures += 8388608;
        f.makeDir("data");
        f.confirmPending();
        if (root.pedidos[8] !== "mkdir /dev/ttyUSB0 :lib/data ") failures += 16777216;
        f.handleOutcome({ device: "/dev/ttyUSB0", action: "mkdir", success: true, raw: "" });
        f.handleOutcome({ device: "/dev/ttyUSB0", action: "list", success: true, entries: [], raw: "" });

        // "Enviar o arquivo aberto" e' pedido a quem sabe (o editor).
        f.uploadCurrentRequested();
        if (root.enviosDoEditor !== 1) failures += 33554432;

        // A recusa ANTES de tocar a porta: erro; a de outro metodo nao mexe.
        f.refresh();
        f.handleFailed("serial.list", "outro");
        if (!f.busy) failures += 67108864;
        f.handleFailed("serial.files", "mpremote nao esta' nesta maquina");
        if (f.busy || f.errorText !== "mpremote nao esta' nesta maquina") failures += 134217728;

        // Trocar de porta esquece a lista da outra; trocar de workspace esquece tudo.
        f.list("/dev/ttyACM0", "");
        if (f.device !== "/dev/ttyACM0" || f.listed || f.path !== "") failures += 268435456;
        controller.workspaceRoot = "/tmp/outro";
        if (f.device !== "" || f.busy || f.entries.length !== 0 || f.hasPending) failures += 536870912;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
