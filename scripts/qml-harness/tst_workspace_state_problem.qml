import QtQuick
import KineinVectis

// O `.kinein` que o core nao usa (INF-13, 0.221.0, 2026-11-12): o
// WorkspaceEventRouter e o ProjectHealthController REAIS, com um cliente
// falso que emite na ordem do CoreClient: o workspaceRoot muda, o
// workspaceChanged, e logo depois o workspaceStateProblem com o
// `stateProblem` da resposta do `workspace.open` (vazio quando ausente).
// Antes, com o `.kinein` ligado a uma pasta de fora, o core gravava la' o
// workspace.json e o banco dos rascunhos, calado.
//
// O que se prova: o motivo vai para a faixa, sem botao, depois dos avisos de
// build e antes do settings.json que o mesmo link recusa; reabrir o mesmo
// projeto sem o motivo limpa a faixa; trocar de projeto e fechar zeram o
// motivo; dispensar esconde a faixa daquele motivo, e outro a traz de volta.
Item {
    id: root
    property int failures: 0

    QtObject {
        id: bridge
        property string workspaceRoot: ""
        signal workspaceChanged()
        signal workspaceStateProblem(string problem)
        // Os outros sinais que o roteador ouve (sem eles, o Connections avisa).
        signal statusChanged()
        signal cmakeStatusResolved(bool configured, bool hasCompileCommands, bool cdbStale,
                                   string cdbStaleBecause, string preset, var presetChoices,
                                   string presetChoicesFile, var compileDatabase,
                                   var configuration)
        signal cmakeConfigureFinished(bool success)
        signal fileSaved(string path)
        signal cargoMetadataResolved(var packages)
        signal dirListed(string path, var entries)
        signal fileCreated(string path)
        signal directoryCreated(string path)
        signal pathRenamed(string from, string to)
        signal pathCopied(string from, string to)
        signal pathBatchTransferred(var result)
        signal pathCopyFailed(string to, string message)
        signal pathDeleted(string path)
        signal filesChanged(var changes)
        signal workspaceBrowseListed(string path, string parent, var entries, var places)
        signal projectEnvironmentsResolved(var environments)
        signal workspaceFolderCreated(string path)
        signal recentWorkspacesResolved(var workspaces)
        signal toolsListed(var tools)
        signal environmentTool(var tool)
        signal requestFailed(string method, string message)
        property bool workspaceTrusted: false
        property var workspaceBuildSystems: []
        function detectTools() {}
    }
    ProjectHealthController {
        id: health
        workspaceRoot: bridge.workspaceRoot
        workspaceKind: "unknown"
        projectRecognized: true
    }
    WorkspaceEventRouter {
        coreClient: bridge
        folderPicker: QtObject { function close() {} }
        workspaceController: QtObject { function handleWorkspaceChanged() {} }
        projectHealthController: health
        recentWorkspacesController: QtObject {
            function handleRequestFailed(method, message) {}
        }
    }

    readonly property string folderLink: ".kinein é um link simbólico, que a IDE não segue no projeto"
    readonly property string databaseLink: ".kinein/kinein.db é um link simbólico, que a IDE não segue no projeto"

    function check(ok, message) {
        if (!ok) { root.failures++; console.error("FALHOU: " + message); }
    }
    // O que o CoreClient faz no `workspace.open`.
    function open(path, problem) {
        bridge.workspaceRoot = path;
        bridge.workspaceChanged();
        bridge.workspaceStateProblem(problem);
    }

    Component.onCompleted: {
        root.open("/tmp/proj", "");
        check(health.status === "ok" && !health.active, "sem motivo, a faixa fica quieta: " + health.status);

        // O motivo do `.kinein`: a faixa avisa, sem botao.
        root.open("/tmp/proj-ligado", root.folderLink);
        check(health.stateProblem === root.folderLink, "o motivo no controller: " + health.stateProblem);
        check(health.active && health.status === "warning", "a faixa avisa: " + health.status);
        check(health.message === "estado do projeto não gravado: " + root.folderLink,
              "o texto da faixa: " + health.message);
        check(health.actionLabel === "" && health.actionTarget === "",
              "a faixa nao tem botao: " + health.actionLabel + " " + health.actionTarget);

        // A ordem: o ambiente Python que falta vem antes; o settings.json que
        // o mesmo link recusa, depois.
        health.settingsProblem = root.folderLink;
        check(health.message === "estado do projeto não gravado: " + root.folderLink,
              "o motivo do .kinein vem antes do settings.json: " + health.message);
        health.pythonNeedsEnvironment = true;
        health.pythonMessage = "falta o ambiente";
        health.pythonActionLabel = "Criar";
        check(health.actionTarget === "pythonEnvironment", "o ambiente Python vem antes: " + health.actionTarget);
        health.pythonNeedsEnvironment = false;
        health.settingsProblem = "";

        // Reabrir o mesmo projeto, com o link trocado por pasta, limpa.
        root.open("/tmp/proj-ligado", "");
        check(health.stateProblem === "" && health.status === "ok" && !health.active,
              "reaberto sem o motivo, a faixa fica quieta: " + health.message);

        // Trocar de projeto zera o motivo antes da resposta nova chegar.
        root.open("/tmp/proj-ligado", root.folderLink);
        bridge.workspaceRoot = "/tmp/outro";
        check(health.stateProblem === "" && health.status === "ok",
              "trocar de projeto zera o motivo: " + health.stateProblem);
        bridge.workspaceChanged();
        bridge.workspaceStateProblem("");

        // Fechar o projeto (o root vazio) tambem.
        root.open("/tmp/proj-ligado", root.folderLink);
        bridge.workspaceRoot = "";
        check(health.stateProblem === "" && health.status === "idle" && !health.active,
              "fechar zera o motivo: " + health.stateProblem + " " + health.status);

        // Dispensar esconde a faixa daquele motivo; outro motivo a traz.
        root.open("/tmp/proj-ligado", root.folderLink);
        health.dismiss();
        check(!health.active && health.status === "warning", "dispensado, a faixa some: " + health.active);
        bridge.workspaceStateProblem(root.databaseLink);
        check(health.active && health.message === "estado do projeto não gravado: " + root.databaseLink,
              "um motivo novo volta a faixa: " + health.message);

        Qt.exit(root.failures === 0 ? 0 : 1);
    }
}
