import QtQuick
// Carrega o JobsController REAL — o mesmo que a IDE instancia.
import "../../ui/qml/jobs"

Item {
    id: root
    width: 100
    height: 100

    property int pedidos: 0
    property string ultimaAba: ""

    JobsController {
        id: jobs

        onRunCoverageRequested: root.pedidos += 1
        onShowTabRequested: function(tab) { root.ultimaAba = tab; }
    }

    Component.onCompleted: {
        let failures = 0;

        // Sem workspace aberto, medir cobertura e no-op COMPLETO: nao pede
        // nada ao core e nao acende o "medindo". Um job disparado sem raiz
        // rodaria a ferramenta no diretorio de quem abriu a IDE.
        jobs.workspaceRoot = "";
        jobs.startCoverage("");
        if (root.pedidos !== 0) failures += 1;
        if (jobs.coverage.measuring) failures += 2;

        // Com workspace: pede uma vez, acende o estado e traz a aba de testes
        // para a frente (a cobertura mora com os testes).
        jobs.workspaceRoot = "/ws";
        jobs.startCoverage("");
        if (root.pedidos !== 1) failures += 4;
        if (!jobs.coverage.measuring) failures += 8;
        if (jobs.coverage.summary === "") failures += 16;
        if (root.ultimaAba !== "tests") failures += 32;

        // Guarda de reentrancia: clicar de novo enquanto mede NAO dispara um
        // segundo job. Dois `cargo llvm-cov` no mesmo alvo brigam pelo lock.
        jobs.startCoverage("");
        if (root.pedidos !== 1) failures += 64;

        // Sucesso: apaga o "medindo", resume os totais e lista os arquivos
        // com caminho RELATIVO a raiz (o absoluto do core nao cabe na tela).
        jobs.handleCoverageFinished(true, 72.5, 145, 200, [
            { path: "/ws/src/main.rs", percent: 90.0,
              linesCovered: 9, linesTotal: 10 },
            { path: "/fora/outro.rs", percent: 50.0,
              linesCovered: 1, linesTotal: 2 }
        ], "");
        if (jobs.coverage.measuring) failures += 128;
        if (jobs.coverage.error !== "") failures += 256;
        if (jobs.coverage.summary.indexOf("72.5") < 0) failures += 512;
        if (jobs.coverage.filesModel.count !== 2) failures += 1024;
        if (jobs.coverage.filesModel.get(0).path !== "src/main.rs") failures += 2048;
        // Caminho fora da raiz NAO e mutilado: fica absoluto.
        if (jobs.coverage.filesModel.get(1).path !== "/fora/outro.rs") failures += 4096;

        // Falha: o erro do core aparece e NENHUM numero e inventado. Um "0%"
        // aqui pareceria medicao — e a pior mentira que este painel poderia
        // contar, porque e indistinguivel de um projeto sem testes.
        jobs.startCoverage("");
        jobs.handleCoverageFinished(false, 0, 0, 0, [],
                                    "compilou com --coverage?");
        if (jobs.coverage.measuring) failures += 8192;
        if (jobs.coverage.summary !== "") failures += 16384;
        if (jobs.coverage.error.indexOf("--coverage") < 0) failures += 32768;
        if (jobs.coverage.filesModel.count !== 0) failures += 65536;

        // Falha SEM texto de erro ainda tem que dizer alguma coisa: painel
        // que apaga tudo e nao explica e igual a painel que nunca rodou.
        jobs.startCoverage("");
        jobs.handleCoverageFinished(false, 0, 0, 0, [], "");
        if (jobs.coverage.error === "") failures += 131072;

        // clear() do workspace leva a cobertura junto: os numeros do projeto
        // anterior nao podem sobreviver a troca de projeto.
        jobs.handleCoverageFinished(true, 10.0, 1, 10, [
            { path: "/ws/a.c", percent: 10.0, linesCovered: 1, linesTotal: 10 }
        ], "");
        jobs.clear();
        if (jobs.coverage.summary !== "") failures += 262144;
        if (jobs.coverage.error !== "") failures += 524288;
        if (jobs.coverage.filesModel.count !== 0) failures += 1048576;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
