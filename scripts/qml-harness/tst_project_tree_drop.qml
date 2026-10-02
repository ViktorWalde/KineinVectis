// A mesma interpretação de MIME usada por linhas e espaço vazio da árvore.
// O gesto nativo X11 foi provado em 2026-09-30; Wayland e AppImage seguem no V8.
import QtQuick
import KineinVectis

Item {
    id: root
    property int failures: 0
    property var received: []

    ProjectTreeDropArea {
        id: destination
        destination: "/w/dst"
        urlDecoder: ({localFilePathsFromUrls: function(urls) {
            if (urls.length === 1 && urls[0] === "file:///tmp/a%C3%A7%C3%A3o%20%231.txt")
                // A ponte Qt devolve uma sequência, não um Array JS.
                return {0: "/tmp/ação #1.txt", length: 1};
            return [];
        }})
        onFilesDropped: function(paths, target, copy, external) {
            root.received = [paths, target, copy, external];
        }
    }

    function check(condition, message) {
        if (!condition) { console.error("FALHOU: " + message); failures += 1; }
    }

    Component.onCompleted: {
        let acceptedAction = Qt.IgnoreAction;
        const valid = {
            formats: ["application/x-kinein-project-paths"],
            source: {dragPaths: ["/w/a.bin", "/w/b com espaço.bin"]},
            proposedAction: Qt.CopyAction,
            accepted: false,
            getDataAsString: function() {
                return JSON.stringify(["/w/a.bin", "/w/b com espaço.bin"]);
            },
            accept: function(action) { acceptedAction = action; }
        };
        destination.acceptDrop(valid);
        check(received.length === 4 && received[0].length === 2
              && received[0][1] === "/w/b com espaço.bin"
              && received[1] === "/w/dst" && received[2] && !received[3]
              && acceptedAction === Qt.CopyAction, "drop perdeu paths, destino ou ação");

        received = [];
        const external = {
            formats: [], hasUrls: true,
            urls: ["file:///tmp/a%C3%A7%C3%A3o%20%231.txt"],
            proposedAction: Qt.MoveAction, accepted: false,
            accept: function(action) { acceptedAction = action; }
        };
        destination.acceptDrop(external);
        check(received.length === 4 && received[0][0] === "/tmp/ação #1.txt"
              && received[1] === "/w/dst" && received[2] && received[3]
              && acceptedAction === Qt.CopyAction,
              "drop externo deve copiar URL local sem mover origem");

        received = [];
        const remote = { formats: [], hasUrls: true,
            urls: ["https://site.test/file.txt"], accepted: true,
            accept: function() { failures += 1; }
        };
        destination.acceptDrop(remote);
        check(received.length === 0 && !remote.accepted,
              "URL remota não pode virar importação local");

        received = [];
        const invalid = {
            formats: ["application/x-kinein-project-paths"],
            source: {dragPaths: ["/w/a.bin"]},
            proposedAction: Qt.MoveAction,
            accepted: true,
            getDataAsString: function() { return "{inválido"; },
            accept: function() { failures += 1; }
        };
        destination.acceptDrop(invalid);
        check(received.length === 0 && !invalid.accepted,
              "MIME inválido produziu mutação");

        received = [];
        const forged = {
            formats: ["application/x-kinein-project-paths"], source: null,
            hasUrls: true, urls: ["file:///tmp/a%C3%A7%C3%A3o%20%231.txt"],
            proposedAction: Qt.MoveAction, accepted: false,
            accept: function(action) { acceptedAction = action; },
            getDataAsString: function() { return JSON.stringify(["/w/a.bin"]); }
        };
        destination.acceptDrop(forged);
        check(received.length === 4 && received[3] && received[2]
              && received[0][0] === "/tmp/ação #1.txt"
              && acceptedAction === Qt.CopyAction,
              "MIME interno forjado tentou mover arquivo da árvore");

        // Soltar uma pasta nela mesma ou num filho e' recusado (0.3.9); o
        // irmao de nome parecido ("/w/src2") nao conta como filho.
        check(ProjectDragRules.insideAny("/w/src", ["/w/src"]), "a propria pasta");
        check(ProjectDragRules.insideAny("/w/src/lib", ["/w/a", "/w/src"]), "um filho");
        check(!ProjectDragRules.insideAny("/w/src2", ["/w/src"]), "irmao de nome parecido");
        check(!ProjectDragRules.insideAny("/w", ["/w/src"]), "o pai pode receber");
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
