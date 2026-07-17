import QtQuick

// Espelho do GitEventRouter: aquele traz o que o core responde (status, log,
// diff), este leva ao core o que o GitController pede.
//
// POR QUE ESTE ROUTER CONHECE O EDITOR, e os outros nao. Quatro operacoes
// (checkout, criar branch, pull, stash) mexem na arvore de trabalho e nao podem
// rodar com arquivo modificado no editor — o git sobrescreveria edicao nao
// salva. Essa guarda e a UNICA aresta cross-domain do git, e ela fica aqui, num
// lugar so, em vez de espalhada por quem chama. `editorController` entra como
// dependencia declarada: o router pergunta "tem arquivo sujo?" e mais nada.
Item {
    id: root

    property var coreClient: null
    property var gitController: null
    property var editorController: null

    visible: false

    // A guarda, num lugar so. Devolve `true` quando a operacao deve PARAR.
    function bloqueadoPorArquivoSujo() {
        if (root.editorController.hasModifiedFiles()) {
            root.gitController.rejectDirtyOperation();
            return true;
        }
        return false;
    }

    Connections {
        target: root.gitController

        function onStatusRequested() {
            root.coreClient.gitStatus();
        }

        function onBranchesRequested() {
            root.coreClient.gitBranches();
        }

        function onCheckoutRequested(branch) {
            if (root.bloqueadoPorArquivoSujo()) {
                return;
            }
            root.coreClient.gitCheckout(branch);
        }

        function onBranchCreateRequested(name) {
            if (root.bloqueadoPorArquivoSujo()) {
                return;
            }
            root.coreClient.gitCreateBranch(name, true);
        }

        function onPullRequested() {
            if (root.bloqueadoPorArquivoSujo()) {
                return;
            }
            root.coreClient.gitPull();
        }

        function onStashRequested(action, message) {
            if (root.bloqueadoPorArquivoSujo()) {
                return;
            }
            root.coreClient.gitStash(action, message);
        }

        function onPushRequested() {
            root.coreClient.gitPush();
        }

        function onFileDiffRequested(path) {
            root.coreClient.gitFileDiff(path);
        }

        function onStageRequested(paths) {
            root.coreClient.gitStage(paths);
        }

        function onUnstageRequested(paths) {
            root.coreClient.gitUnstage(paths);
        }

        function onDiscardRequested(paths) {
            root.coreClient.gitDiscard(paths);
        }

        function onCommitRequested(message) {
            root.coreClient.gitCommit(message);
        }

        function onBlameRequested(path) {
            root.coreClient.gitBlame(path);
        }

        function onLogRequested() {
            root.coreClient.gitLog();
        }

        function onCommitDiffRequested(sha) {
            root.coreClient.gitCommitDiff(sha);
        }
    }
}
