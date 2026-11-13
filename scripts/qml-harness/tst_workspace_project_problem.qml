import QtQuick
import KineinVectis

// O arquivo do projeto que o core nao le (INF-18, 0.223.0, 2026-11-13): o
// ProjectHealthController REAL recebe o `projectProblem` da resposta do
// `workspace.open` e de cada `cmake.status`, como o roteador o entrega. Antes,
// um FIFO no lugar do `CMakeLists.txt` ou do `CMakeCache.txt` prendia o core
// na abertura, e a IDE ficava sem resposta. Sem cliente falso (R-TEST-06): o
// caminho do sinal ate' o controller e' provado no tst_workspace_event_router.
//
// O que se prova: o motivo vai para a faixa, sem botao, depois do pedido de
// confianca e antes dos avisos do CMake; o `.kinein` que nao grava vem
// depois; o status sem o motivo limpa a faixa; trocar de projeto e fechar
// zeram o motivo; dispensar esconde a faixa daquele motivo, e outro a traz.
Item {
    id: root
    property int failures: 0

    ProjectHealthController {
        id: health
        workspaceKind: "unknown"
        projectRecognized: true
    }

    readonly property string fifo: "CMakeLists.txt não é um arquivo regular"
    readonly property string folder: ".kinein/build/CMakeCache.txt não é um arquivo regular"

    function check(ok, message) {
        if (!ok) { root.failures++; console.error("FALHOU: " + message); }
    }
    // O que chega ao controller no `workspace.open`: a raiz, o `stateProblem`
    // e o `projectProblem` da resposta.
    function open(path, problem) {
        health.workspaceRoot = path;
        health.stateProblem = "";
        health.projectProblem = problem;
    }

    Component.onCompleted: {
        root.open("/tmp/proj", "");
        check(health.status === "ok" && !health.active, "sem motivo, a faixa fica quieta: " + health.status);

        // O arquivo recusado: a faixa avisa, sem botao.
        root.open("/tmp/proj-fifo", root.fifo);
        check(health.projectProblem === root.fifo, "o motivo no controller: " + health.projectProblem);
        check(health.active && health.status === "warning", "a faixa avisa: " + health.status);
        check(health.message === "arquivo do projeto não lido: " + root.fifo,
              "o texto da faixa: " + health.message);
        check(health.actionLabel === "" && health.actionTarget === "",
              "a faixa nao tem botao: " + health.actionLabel + " " + health.actionTarget);

        // A ordem: o pedido de confianca vem antes; a escolha da CDB do
        // CMake e o `.kinein` que nao grava, depois.
        health.toolsList = ["cmake", "ninja", "clangd", "gxx"].map(
            function (id) { return { id: id, status: "detected" }; });
        health.workspaceBuildSystems = ["cmake"];
        health.handleCmakeStatus(false, false, "", "", [], "",
                                 { choiceRequired: true, absentBecause: "escolha" }, {});
        check(health.actionTarget === "trust", "a confianca vem antes: " + health.actionTarget);
        health.handleTrust(true);
        check(health.message === "arquivo do projeto não lido: " + root.fifo,
              "o arquivo recusado vem antes da escolha da CDB: " + health.message);
        health.stateProblem = ".kinein é um link simbólico, que a IDE não segue no projeto";
        check(health.message === "arquivo do projeto não lido: " + root.fifo,
              "o arquivo recusado vem antes do .kinein: " + health.message);
        health.stateProblem = "";
        health.workspaceBuildSystems = [];

        // O `cmake.status` sem o motivo, com o arquivo trocado, limpa.
        health.projectProblem = "";
        check(health.projectProblem === "" && health.status === "ok" && !health.active,
              "o status sem o motivo deixa a faixa quieta: " + health.message);

        // Trocar de projeto zera o motivo antes da resposta nova chegar.
        root.open("/tmp/proj-fifo", root.fifo);
        health.workspaceRoot = "/tmp/outro";
        check(health.projectProblem === "" && health.status === "ok",
              "trocar de projeto zera o motivo: " + health.projectProblem);

        // Fechar o projeto (o root vazio) tambem.
        root.open("/tmp/proj-fifo", root.fifo);
        health.workspaceRoot = "";
        check(health.projectProblem === "" && health.status === "idle" && !health.active,
              "fechar zera o motivo: " + health.projectProblem + " " + health.status);

        // Dispensar esconde a faixa daquele motivo; outro motivo a traz.
        root.open("/tmp/proj-fifo", root.fifo);
        health.dismiss();
        check(!health.active && health.status === "warning", "dispensado, a faixa some: " + health.active);
        health.projectProblem = root.folder;
        check(health.active && health.message === "arquivo do projeto não lido: " + root.folder,
              "um motivo novo volta a faixa: " + health.message);

        Qt.exit(root.failures === 0 ? 0 : 1);
    }
}
