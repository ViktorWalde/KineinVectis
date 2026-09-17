import QtQuick
// Carrega o RuntimeController.qml REAL (arquivo do projeto, sem copia).
import "../../ui/qml/runtime"

// A porta escolhida no Executar de MicroPython (C3 do roadmaps/41, protocolo
// 0.110.0): o RuntimeController recebe `serialDevice` por binding (a escolha
// mora no EmbeddedController) e a REPASSA nos gestos de executar — nunca a
// guarda, nunca a valida.
//
// Por que existe: a fiacao selecao -> estado -> roteador -> `device` e' QML
// puro. Um sinal que perde o segundo parametro, ou um `startRun` que manda a
// porta junto com um comando digitado (que o core RECUSA), compila e passa no
// lint; so' se ve quando o mpremote conecta na porta errada — na placa.
Item {
    id: root
    width: 100
    height: 100

    property var pedidos: []

    RuntimeController {
        id: runtime

        workspaceRoot: "/tmp/workspace"
        onRunStartRequested: function(command, device) {
            root.pedidos.push("start:" + command + ":" + device);
        }
        onRunScriptRequested: function(path, device) {
            root.pedidos.push("script:" + path + ":" + device);
        }
    }

    Component.onCompleted: {
        let failures = 0;

        // Sem escolha: o campo vai vazio (= ausente na ponte C++).
        runtime.startRun("");
        runtime.startScript("/tmp/workspace/main.py");
        if (root.pedidos.join(",") !== "start::,script:/tmp/workspace/main.py:") failures += 1;

        // Com escolha: o botao Executar e o "Executar" num .py levam a porta.
        root.pedidos = [];
        runtime.serialDevice = "/dev/ttyUSB0";
        runtime.startRun("");
        runtime.startScript("/tmp/workspace/main.py");
        if (root.pedidos.join(",")
                !== "start::/dev/ttyUSB0,script:/tmp/workspace/main.py:/dev/ttyUSB0") failures += 2;

        // Um comando DIGITADO roda como foi escrito: a porta NAO vai junto
        // (o core recusa command + device).
        root.pedidos = [];
        runtime.startRun("python tools/gera.py");
        runtime.submitRunInput("echo oi");
        if (root.pedidos.join(",") !== "start:python tools/gera.py:,start:echo oi:") failures += 4;

        // A escolha e' de quem a fez: trocar/desfazer no painel chega aqui
        // pelo binding e vale no proximo gesto.
        root.pedidos = [];
        runtime.serialDevice = "/dev/ttyACM1";
        runtime.startScript("/tmp/workspace/util.py");
        runtime.serialDevice = "";
        runtime.startScript("/tmp/workspace/util.py");
        if (root.pedidos.join(",")
                !== "script:/tmp/workspace/util.py:/dev/ttyACM1,script:/tmp/workspace/util.py:") failures += 8;

        // Processo em curso ou sem workspace: nada vai, com ou sem porta.
        root.pedidos = [];
        runtime.serialDevice = "/dev/ttyUSB0";
        runtime.running = true;
        runtime.startRun("");
        runtime.startScript("/tmp/workspace/main.py");
        runtime.running = false;
        runtime.workspaceRoot = "";
        runtime.startRun("");
        if (root.pedidos.length !== 0) failures += 16;

        // A recusa do core ao run.script aparece na sessao de execucao, como
        // a do run.start — e' a aba que o gesto acabou de abrir.
        runtime.workspaceRoot = "/tmp/workspace";
        runtime.handleRequestFailed("run.script", "projeto MicroPython: o mpremote nao esta' nesta maquina");
        if (runtime.runModel.count !== 1
                || runtime.runModel.get(0).kind !== "stderr") failures += 32;
        runtime.handleRequestFailed("git.status", "outro dominio");
        if (runtime.runModel.count !== 1) failures += 64;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
