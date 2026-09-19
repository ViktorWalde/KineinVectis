// A saida dos testes chega a tela (41 A6, fechado na fatia 3 da cadeia Python,
// 2026-09-13): o JobsController guarda a saida BRUTA do runner
// (event.test.started/output) num modelo proprio, o `error` de um runner que
// nem correu vira o resumo, e comecar uma rodada nova limpa a anterior.
// E o ProjectTreeController aceita `.py` como executavel (o core roda com o
// interpretador do projeto).
//
// Por que existe: o sinal testOutput existia no C++ desde o test.run e ninguem
// ouvia — o pytest explicava a falha para ninguem. Nenhum compilador acusa um
// sinal sem ouvinte.
import QtQuick
import "../../ui/qml/jobs"
import "../../ui/qml/project"

Item {
    id: root

    property var abas: []
    property var descobertas: []
    property var umSo: []

    JobsController {
        id: jobs

        workspaceRoot: "/tmp/proj"
        onShowTabRequested: function(tab) { root.abas.push(tab); }
        onDiscoverTestsRequested: function(buildSystem) { root.descobertas.push(buildSystem); }
        onRunOneTestRequested: function(testId, buildSystem) { root.umSo.push(testId); }
    }

    ProjectTreeController {
        id: tree
    }

    Component.onCompleted: {
        let failures = 0;
        // Antes do catalogo do core (run.capabilities), NADA e' executavel nem
        // depuravel: uma lista escrita a mao aqui responderia "sim" sem o core
        // ter falado — o defeito que o format.capabilities corrigiu em 0.61.
        if (tree.isRunnableScript("/tmp/proj/tools/gera.py", "file")) failures += 1 << 28;
        if (tree.isDebuggableScript("/tmp/proj/tools/gera.py", "file")) failures += 1 << 29;
        tree.applyRunCapabilities(["sh", "bash", "zsh", "py"], ["py"]);

        // Comecar limpa e abre a aba.
        jobs.handleTestOutput("lixo de antes");
        jobs.startTests("");
        if (jobs.testOutputModel.count !== 0 || jobs.testModel.count !== 0) failures += 1;
        if (root.abas.indexOf("tests") < 0) failures += 2;
        if (jobs.testSummary.indexOf("rodando") < 0) failures += 4;

        // O comando abre a saida; cada linha entra na ordem.
        jobs.handleTestStarted(".venv/bin/python -m pytest -v");
        jobs.handleTestOutput("tests/test_a.py::test_soma PASSED [ 50%]");
        jobs.handleTestOutput("E   assert 1 == 2");
        if (jobs.testOutputModel.count !== 3) failures += 8;
        if (jobs.testOutputModel.get(0).line !== "$ .venv/bin/python -m pytest -v") failures += 16;
        if (jobs.testOutputModel.get(2).line !== "E   assert 1 == 2") failures += 32;

        // Casos e totais continuam como antes.
        jobs.handleTestCase("tests/test_a.py::test_soma", "passed");
        jobs.handleTestFinished(false, 1, 1, 0, "");
        if (jobs.testModel.count !== 1) failures += 64;
        if (jobs.testSummary.indexOf("FALHOU") < 0 || jobs.testSummary.indexOf("falhou: 1") < 0) failures += 128;
        // O placar da aba (HUD do painel de baixo, 2026-09-18): passou/total.
        if (jobs.testsBadge !== "1/2" || jobs.testsFailed !== 1) failures += 4096;

        // Um runner que nem correu: o `error` do core E' o resumo, e vai para a saida.
        jobs.handleTestFinished(false, 0, 0, 0, "pytest ausente: instale-o NO ambiente do projeto");
        if (jobs.testSummary.indexOf("nao rodou") < 0 || jobs.testSummary.indexOf("pytest ausente") < 0) failures += 256;
        if (jobs.testOutputModel.get(jobs.testOutputModel.count - 1).line.indexOf("pytest ausente") < 0) failures += 512;

        // A saida tem teto (nao cresce sem fim numa suite grande).
        for (let i = 0; i < 2100; i++) jobs.handleTestOutput("linha " + i);
        if (jobs.testOutputModel.count > 2000) failures += 1024;

        // clear() esquece a saida junto com o resto.
        jobs.clear();
        if (jobs.testOutputModel.count !== 0 || jobs.testSummary !== "" || jobs.testsBadge !== "") failures += 2048;

        // A ARVORE (test.discover, 2026-09-13): listar pede UMA vez; o resultado
        // vira linhas com status vazio; um caso que roda com o MESMO id pinta a
        // linha em vez de virar caso solto; rodar um pede o id exato; um run
        // novo apaga os status mas mantem a arvore; a falha da listagem vira o
        // resumo e nao deixa arvore.
        jobs.discoverTests("");
        jobs.discoverTests("");
        if (root.descobertas.length !== 1 || !jobs.discovering) failures += 131072;
        jobs.handleTestsDiscovered({ success: true, runner: "pytest", command: "python -m pytest --collect-only -q",
                                     tests: [ { id: "tests/test_a.py::test_soma", name: "test_soma", file: "tests/test_a.py" },
                                              { id: "tests/test_a.py::test_x[a b]", name: "test_x[a b]", file: "tests/test_a.py" } ] });
        if (jobs.discovering || jobs.discoveredModel.count !== 2 || jobs.discoverRunner !== "pytest") failures += 262144;
        if (jobs.testSummary.indexOf("2 teste(s) listado(s)") !== 0) failures += 524288;
        jobs.handleTestCase("tests/test_a.py::test_x[a b]", "failed");
        jobs.handleTestCase("tests/outro.py::solto", "passed");
        if (jobs.discoveredModel.get(1).status !== "failed" || jobs.discoveredModel.get(0).status !== "") failures += 1048576;
        if (jobs.testModel.count !== 1 || jobs.testModel.get(0).name !== "tests/outro.py::solto") failures += 2097152;
        jobs.runOneTest("tests/test_a.py::test_x[a b]", "");
        if (root.umSo.length !== 1 || root.umSo[0] !== "tests/test_a.py::test_x[a b]") failures += 4194304;
        if (jobs.discoveredModel.count !== 2 || jobs.discoveredModel.get(1).status !== "" || jobs.testModel.count !== 0) failures += 8388608;
        jobs.runOneTest("", "");
        if (root.umSo.length !== 1) failures += 16777216;
        jobs.handleTestsDiscovered({ success: false, error: "pytest ausente: instale-o NO ambiente do projeto" });
        if (jobs.discoveredModel.count !== 0 || jobs.testSummary.indexOf("nao listou: pytest ausente") !== 0) failures += 33554432;

        // Executar: shells e .py; nao pastas, nao .txt.
        if (!tree.isRunnableScript("/tmp/proj/tools/gera.py", "file")) failures += 4096;
        if (!tree.isRunnableScript("/tmp/proj/build.sh", "file")) failures += 8192;
        if (tree.isRunnableScript("/tmp/proj/tools", "directory")) failures += 16384;
        if (tree.isRunnableScript("/tmp/proj/notas.txt", "file")) failures += 32768;
        if (tree.isRunnableScript("/tmp/proj/pyproject.toml", "file")) failures += 65536;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
