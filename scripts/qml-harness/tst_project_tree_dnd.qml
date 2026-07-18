import QtQuick
// Carrega os controllers REAIS da arvore: regras de arraste e teclado.
import "../../ui/qml/project"

Item {
    id: root
    width: 100
    height: 100

    property var listed: []
    property string renameFrom: ""
    property string renameTo: ""
    property string opened: ""

    function resetSpies() {
        listed = [];
        renameFrom = "";
        renameTo = "";
        opened = "";
    }

    function listedHas(path) {
        return listed.indexOf(path) >= 0;
    }

    Item {
        visible: false

        ProjectTreeController {
            id: tree

            workspaceRoot: "/ws"
            onListDirRequested: function(path) { root.listed.push(path); }
            onRenamePathRequested: function(from, to) {
                root.renameFrom = from;
                root.renameTo = to;
            }
            onReadFileRequested: function(path) { root.opened = path; }
        }

        ProjectTreeGestures {
            id: gestures

            tree: tree
        }
    }

    Component.onCompleted: {
        let failures = 0;

        // /ws/src (aberta) > /ws/src/lib.rs ; /ws/main.rs ; /ws/docs
        tree.setDirectoryListing("/ws", [
            { name: "src", kind: "directory" },
            { name: "main.rs", kind: "file" },
            { name: "docs", kind: "directory" }
        ]);
        tree.setDirectoryListing("/ws/src", [{ name: "lib.rs", kind: "file" }]);
        if (tree.entriesModel.count !== 4) failures += 1;

        // --- regras de arraste -------------------------------------------
        // Soltar num ARQUIVO significa soltar na pasta dele.
        if (gestures.dropTargetDir("/ws/src/lib.rs", "file") !== "/ws/src") {
            failures += 2;
        }
        // Alvo valido: arquivo da raiz para uma subpasta.
        if (!gestures.canDropOn("/ws/main.rs", "/ws/src", "directory")) {
            failures += 4;
        }
        // Sobre si mesmo.
        if (gestures.canDropOn("/ws/src", "/ws/src", "directory")) failures += 8;
        // Pasta para dentro de si mesma — o EINVAL que nunca deve chegar ao core.
        if (gestures.canDropInto("/ws/src", "/ws/src/deep/deeper")) failures += 16;
        // Para o proprio pai: no-op disfarcado de sucesso.
        if (gestures.canDropOn("/ws/main.rs", "/ws/main.rs", "file")) failures += 32;
        // Voltar para a raiz pelo no do projeto (alvo "" = workspaceRoot).
        if (!gestures.canDropOn("/ws/src/lib.rs", "", "directory")) failures += 64;
        // Ja esta na raiz: soltar no no do projeto nao acende.
        if (gestures.canDropOn("/ws/main.rs", "", "directory")) failures += 128;

        // --- o arraste efetivo -------------------------------------------
        root.resetSpies();
        gestures.dropOn("/ws/main.rs", "/ws/src", "directory");
        if (root.renameFrom !== "/ws/main.rs"
                || root.renameTo !== "/ws/src/main.rs") failures += 256;
        if (!tree.moveInFlight) failures += 512;

        // Drop invalido nao pode chegar ao core.
        root.resetSpies();
        gestures.dropOn("/ws/src", "/ws/src/lib.rs", "file");
        if (root.renameFrom !== "") failures += 1024;

        // --- falha de MOVER nao abre o dialogo de renomear ----------------
        root.resetSpies();
        gestures.dropOn("/ws/main.rs", "/ws/src", "directory");
        tree.handleRequestFailed("fs.rename", "destino ja existe");
        if (tree.entryRenameVisible) failures += 2048;
        if (tree.moveError !== "destino ja existe") failures += 4096;
        if (tree.moveInFlight) failures += 8192;

        // ...mas falha de RENOMEAR continua abrindo o dialogo.
        tree.moveError = "";
        tree.handleRequestFailed("fs.rename", "nome invalido");
        if (!tree.entryRenameVisible || tree.entryRenameError !== "nome invalido") {
            failures += 16384;
        }
        tree.entryRenameVisible = false;

        // --- mover atualiza AS DUAS pastas; renomear, so uma --------------
        root.resetSpies();
        gestures.dropOn("/ws/main.rs", "/ws/src", "directory");
        tree.handlePathRenamed("/ws/main.rs", "/ws/src/main.rs");
        if (!root.listedHas("/ws/src") || !root.listedHas("/ws")) failures += 32768;
        if (tree.moveInFlight) failures += 65536;

        root.resetSpies();
        tree.handlePathRenamed("/ws/docs", "/ws/documentos");
        if (root.listed.length !== 1 || !root.listedHas("/ws")) failures += 131072;

        // --- navegacao por teclado ---------------------------------------
        tree.selectEntry("/ws/src", "directory");
        gestures.moveSelection(1);
        if (tree.selectedPath !== "/ws/src/lib.rs") failures += 262144;
        gestures.moveSelection(-1);
        if (tree.selectedPath !== "/ws/src") failures += 524288;

        // Seta esquerda num item folha sobe para a pasta-mae.
        tree.selectEntry("/ws/src/lib.rs", "file");
        gestures.collapseSelected();
        if (tree.selectedPath !== "/ws/src") failures += 1048576;

        // Seta esquerda numa pasta aberta fecha e engole os filhos.
        gestures.collapseSelected();
        if (tree.entriesModel.count !== 3
                || tree.entriesModel.get(0).expanded) failures += 2097152;

        // Seta direita numa pasta fechada pede a listagem.
        root.resetSpies();
        gestures.expandSelected();
        if (!root.listedHas("/ws/src")) failures += 4194304;

        // Enter num arquivo abre.
        root.resetSpies();
        tree.selectEntry("/ws/main.rs", "file");
        gestures.activateSelected();
        if (root.opened !== "/ws/main.rs") failures += 8388608;

        // Selecao fora do modelo nao pode explodir nem mover nada.
        tree.selectEntry("/ws/inexistente", "file");
        gestures.activateSelected();
        gestures.collapseSelected();
        gestures.expandSelected();

        // O codigo de saida de um processo tem 8 BITS: o bitmask vai para a
        // SAIDA (onde nao trunca) e o exit so diz passou/falhou.
        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
