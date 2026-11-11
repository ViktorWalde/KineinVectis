import QtQuick
import KineinVectis

// A arvore da aba Testes ao rodar (C1-39 e C1-44, 2026-11-10): o JobsController
// e o TestsPanel REAIS. Uma rodada nova zera a saida antes da primeira linha
// dela; se a lista crescesse nesse instante, caberia inteira e voltaria ao
// topo, e o caso que rodou sairia da vista. E o ponto do ignorado e' ambar,
// e nao a cor do que nao rodou.
Item {
    id: root
    width: 800
    height: 420
    property int failures: 0
    property var ran: []

    JobsController {
        id: jobs
        workspaceRoot: "/tmp/proj"
        onRunOneTestRequested: function(testId, buildSystem) { root.ran.push(testId); }
    }
    TestsPanel {
        id: panel
        anchors.fill: parent
        controller: jobs
    }

    function find(item, name) {
        if (item.objectName === name) return item;
        for (const child of item.children) {
            const result = find(child, name);
            if (result) return result;
        }
        return null;
    }
    function check(ok, message) {
        if (!ok) { root.failures++; console.error("FALHOU: " + message); }
    }
    function discover() {
        const tests = [];
        for (let i = 0; i < 10; i++) tests.push({ id: "t" + i, name: "t" + i });
        for (let i = 0; i < 7; i++) tests.push({ id: "ns::unit::Math.Case" + i, name: "ns::unit::Math.Case" + i });
        jobs.handleTestsDiscovered({ success: true, runner: "ctest", command: "ctest -N", tests: tests });
    }

    Component.onCompleted: {
        const tree = find(panel, "testsTree");
        check(tree !== null, "a arvore existe");

        // Listada, sem saida: a arvore usa o painel inteiro.
        discover();
        const whole = tree.height;
        check(jobs.discoveredModel.count === 17, "17 linhas: " + jobs.discoveredModel.count);

        // Com saida, a arvore divide o painel com ela.
        jobs.handleTestStarted("ctest --test-dir /b --output-on-failure");
        jobs.handleTestOutput("100% tests passed");
        const shared = tree.height;
        check(shared < whole, "a saida toma parte do painel: " + shared + " < " + whole);

        // C1-44: o run novo zera a saida, e a arvore NAO cresce.
        jobs.runOneTest("ns::unit::Math.Case6", "");
        check(root.ran.length === 1 && root.ran[0] === "ns::unit::Math.Case6", "rodou o id: " + root.ran);
        check(jobs.testOutputModel.count === 0, "a saida foi zerada");
        check(tree.height === shared, "a arvore fica do mesmo tamanho ao rodar: " + tree.height + " e " + shared);
        jobs.handleTestStarted("'/b/ge' --gtest_filter=Math.Case6");
        check(tree.height === shared, "e continua quando a saida volta: " + tree.height);

        // Uma listagem nova com a saida na tela tambem nao mexe no tamanho, nem
        // o run seguinte a ela.
        discover();
        check(tree.height === shared, "relistar com saida: " + tree.height);
        jobs.runOneTest("t3", "");
        check(tree.height === shared, "rodar depois de relistar: " + tree.height);

        // Sem arvore, a lista sao os casos do run e volta a seguir as linhas:
        // o run sem arvore zera a saida e usa o painel inteiro.
        jobs.clear();
        jobs.handleTestStarted("ctest --test-dir /b --output-on-failure");
        check(tree.height === shared, "sem arvore, com saida: " + tree.height);
        jobs.startTests("");
        check(tree.height === whole, "sem arvore, o run usa o painel inteiro: " + tree.height + " e " + whole);

        // Sem arvore e sem saida, a trava solta: a proxima listagem usa o
        // painel inteiro de novo.
        jobs.clear();
        discover();
        check(tree.height === whole, "depois do clear, o painel inteiro: " + tree.height + " e " + whole);

        // C1-39: o ignorado e' ambar; o que nao rodou fica apagado.
        check(Qt.colorEqual(panel.statusColor("ignored"), Theme.warningSoft), "ignorado em warningSoft");
        check(Qt.colorEqual(panel.statusColor(""), Theme.textMuted), "nao rodou em textMuted");
        check(Qt.colorEqual(panel.statusColor("passed"), Theme.successSoft), "passou em successSoft");
        check(Qt.colorEqual(panel.statusColor("failed"), Theme.errorSoft), "falhou em errorSoft");

        Qt.exit(root.failures === 0 ? 0 : 1);
    }
}
