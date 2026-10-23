import QtQuick
import KineinVectis

// probe-rs (0.4.1h4b2b4b3, roadmaps/52 §14.36): a sonda escolhida em Gravar
// acompanha o debug.start; controllers e roteador reais, ponte falsa.
Item {
    id: root
    property var requests: []
    property int failures: 0
    function check(ok, message) {
        if (!ok) { failures++; console.error("FALHAS: " + message); }
    }
    QtObject {
        id: bridge
        function debugStart(program, connect, probe) {
            root.requests.push({ program: program, connect: connect, probe: probe });
        }
        function debugStop() {}
    }
    DebugController { id: debug; workspaceRoot: "/ws" }
    EmbeddedController { id: embedded; workspaceRoot: "/ws" }
    DebugRequestRouter { coreClient: bridge; debugController: debug; embeddedController: embedded }

    Component.onCompleted: {
        embedded.handleProject({ embedded: true, frameworks: [], artifacts: {},
                                 target: { chip: "STM32F411RETx", flashEngine: "probe-rs" } });
        embedded.flash.selectProbe("0483:374b:066D");
        debug.startDebug("/ws/fw.elf");
        check(requests.length === 1 && requests[0].probe === "0483:374b:066D", "a sonda de Gravar vai no debug.start");
        debug.starting = false;
        embedded.flash.requestTargets();
        embedded.flash.handleTargets([
            { name: "firmware", kind: "executable", sourceDir: "/ws", artifacts: ["/ws/.kinein/build/firmware.elf"] },
            { name: "bootloader", kind: "executable", sourceDir: "/ws", artifacts: ["/ws/.kinein/build/bootloader.elf"] }], "fileApi");
        embedded.flash.selectTarget("firmware");
        debug.startDebug("");
        check(requests[requests.length - 1].program === "/ws/.kinein/build/firmware.elf", "sem programa, o alvo de Gravar");
        debug.starting = false;
        debug.startDebug("/ws/outro.elf");
        check(requests[requests.length - 1].program === "/ws/outro.elf", "programa explícito prevalece");
        debug.starting = false;
        requests = [requests[0]];
        debug.startDebug("", { host: "localhost", port: 5678 });
        check(requests.length === 2 && requests[1].probe === "", "attach nunca leva sonda");
        debug.starting = false;
        embedded.flash.selectEngine("esptool");
        debug.startDebug("/ws/fw.elf");
        check(requests.length === 3 && requests[2].probe === "", "fora do destino sonda nada vai");
        debug.starting = false;
        embedded.flash.selectEngine("");
        embedded.flash.selectProbe("");
        debug.startDebug("/ws/fw.elf");
        check(requests.length === 4 && requests[3].probe === "", "sem sonda escolhida nada muda");
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
