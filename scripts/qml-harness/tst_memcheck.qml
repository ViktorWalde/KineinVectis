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

        onRunMemcheckRequested: root.pedidos += 1
        onShowTabRequested: function(tab) { root.ultimaAba = tab; }
    }

    function origemDoProblema(indice) {
        return jobs.problemsModel.count > indice
             ? jobs.problemsModel.get(indice).source : "";
    }

    Component.onCompleted: {
        let failures = 0;

        // Sem workspace, analisar e no-op completo. Isto importa mais aqui que
        // nas outras fatias: a analise dinamica EXECUTA os testes do projeto.
        jobs.workspaceRoot = "";
        jobs.startMemcheck();
        if (root.pedidos !== 0) failures += 1;
        if (jobs.memchecking) failures += 2;

        // Com workspace: pede uma vez e traz a aba Problemas.
        jobs.workspaceRoot = "/ws";
        jobs.startMemcheck();
        if (root.pedidos !== 1) failures += 4;
        if (!jobs.memchecking) failures += 8;
        if (jobs.memcheckSummary === "") failures += 16;
        if (root.ultimaAba !== "problems") failures += 32;

        // Guarda de reentrancia: rodar os testes duas vezes em paralelo sob o
        // Valgrind disputaria os mesmos arquivos de saida do projeto.
        jobs.startMemcheck();
        if (root.pedidos !== 1) failures += 64;

        // Achado com origem PROPRIA. Se caisse em "quality", a analise
        // estatica o apagaria — e as duas rodam no mesmo fluxo de trabalho.
        jobs.handleMemcheckDiagnostic({
            severity: "error", message: "Invalid write of size 4",
            file: "src/vaza.c", line: 5
        });
        if (jobs.problemsModel.count !== 1) failures += 128;
        if (root.origemDoProblema(0) !== "memcheck") failures += 256;

        // As tres origens de analise convivem sem se atropelar.
        jobs.handleQualityDiagnostic({ severity: "warning", message: "lint",
                                       file: "src/a.c", line: 1 });
        jobs.handleAuditDiagnostic({ severity: "error", message: "RUSTSEC (x 1.0)",
                                     file: "Cargo.lock", line: 2 });
        if (jobs.problemsModel.count !== 3) failures += 512;
        jobs.removeProblemsBySource("quality");
        jobs.removeProblemsBySource("audit");
        if (jobs.problemsModel.count !== 1) failures += 1024;
        if (root.origemDoProblema(0) !== "memcheck") failures += 2048;

        // Sucesso COM achados diz quantos e em quantos testes.
        jobs.handleMemcheckFinished(true, 3, 2, "");
        if (jobs.memchecking) failures += 4096;
        if (jobs.memcheckSummary.indexOf("2") < 0) failures += 8192;
        if (jobs.memcheckSummary.indexOf("3") < 0) failures += 16384;

        // Sucesso SEM achados nao pode parecer falha: "nenhum problema" e um
        // resultado, e o mais desejavel deles.
        jobs.handleMemcheckFinished(true, 5, 0, "");
        if (jobs.memcheckSummary.indexOf("nenhum") < 0) failures += 32768;
        if (jobs.memcheckSummary.indexOf("5") < 0) failures += 65536;

        // Falha repassa o erro do core, que diz o GESTO que falta.
        jobs.startMemcheck();
        jobs.handleMemcheckFinished(false, 0, 0, "rode o build uma vez antes");
        if (jobs.memchecking) failures += 131072;
        if (jobs.memcheckSummary.indexOf("build") < 0) failures += 262144;

        // clear() do workspace leva a analise junto.
        jobs.clear();
        if (jobs.memcheckSummary !== "") failures += 524288;
        if (jobs.problemsModel.count !== 0) failures += 1048576;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
