// "Depurar" num .py da arvore (fatia 4 da cadeia Python, 2026-09-13): o
// ProjectTreeController so' oferece Depurar para `.py` (Executar continua
// valendo para shells), o pedido leva o CAMINHO do arquivo, e o DebugController
// repassa o programa explicito ao core — com as mesmas guardas do botao
// (sem workspace ou com sessao viva, nada sai).
//
// Por que existe: uma guarda a menos e o duplo clique sobe duas sessoes; um
// caminho perdido no meio e o core depura o alvo automatico em vez do arquivo
// que o usuario apontou. Nenhum compilador acusa isso.
import QtQuick
import "../../ui/qml/project"
import "../../ui/qml/debug"

Item {
    id: root

    property var depurarPedidos: []
    property var programas: []
    property var abas: []

    ProjectTreeController {
        id: tree

        workspaceRoot: "/tmp/proj"
        onDebugScriptRequested: function(path) { root.depurarPedidos.push(path); }
    }

    DebugController {
        id: dbg

        onStartRequested: function(program) { root.programas.push(program); }
        onShowTabRequested: function(tab) { root.abas.push(tab); }
    }

    Component.onCompleted: {
        let failures = 0;

        // Depurar: so' .py; Executar: shells e .py.
        if (!tree.isDebuggableScript("/tmp/proj/tools/gera.py", "file")) failures += 1;
        if (tree.isDebuggableScript("/tmp/proj/build.sh", "file")) failures += 2;
        if (tree.isDebuggableScript("/tmp/proj/pacote", "directory")) failures += 4;
        if (tree.isDebuggableScript("/tmp/proj/stubs.pyi", "file")) failures += 8;

        // O menu aberto num .py liga as duas acoes; num .sh so' Executar.
        tree.openEntryMenu("/tmp/proj/tools/gera.py", "file", "gera.py", 10, 10);
        if (!tree.entryMenuDebuggable || !tree.entryMenuRunnable) failures += 16;
        tree.debugEntryScript();
        if (root.depurarPedidos.length !== 1 || root.depurarPedidos[0] !== "/tmp/proj/tools/gera.py") failures += 32;
        if (tree.entryMenuVisible) failures += 64;
        tree.openEntryMenu("/tmp/proj/build.sh", "file", "build.sh", 10, 10);
        if (tree.entryMenuDebuggable || !tree.entryMenuRunnable) failures += 128;
        tree.debugEntryScript();
        if (root.depurarPedidos.length !== 1) failures += 256;

        // Sem workspace, Depurar nao sai.
        tree.workspaceRoot = "";
        tree.openEntryMenu("/tmp/proj/tools/gera.py", "file", "gera.py", 10, 10);
        tree.debugEntryScript();
        if (root.depurarPedidos.length !== 1) failures += 512;

        // O DebugController: sem workspace nada; com workspace o programa vai
        // como veio e a aba de debug abre; com sessao viva, nada.
        dbg.startDebugProgram("/tmp/proj/tools/gera.py");
        if (root.programas.length !== 0) failures += 1024;
        dbg.workspaceRoot = "/tmp/proj";
        dbg.startDebugProgram("/tmp/proj/tools/gera.py");
        if (root.programas.length !== 1 || root.programas[0] !== "/tmp/proj/tools/gera.py") failures += 2048;
        if (root.abas.indexOf("debug") < 0) failures += 4096;
        dbg.startDebug();
        if (root.programas.length !== 2 || root.programas[1] !== "") failures += 8192;
        dbg.sessionActive = true;
        dbg.startDebugProgram("/tmp/proj/tools/gera.py");
        if (root.programas.length !== 2) failures += 16384;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
