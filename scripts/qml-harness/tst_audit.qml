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

        onRunAuditRequested: root.pedidos += 1
        onShowTabRequested: function(tab) { root.ultimaAba = tab; }
    }

    Component.onCompleted: {
        let failures = 0;

        // Sem workspace, auditar e no-op completo.
        jobs.workspaceRoot = "";
        jobs.startAudit();
        if (root.pedidos !== 0) failures += 1;
        if (jobs.auditing) failures += 2;

        // Com workspace: pede uma vez e traz a aba Problemas para a frente,
        // porque e la que os achados aparecem.
        jobs.workspaceRoot = "/ws";
        jobs.startAudit();
        if (root.pedidos !== 1) failures += 4;
        if (!jobs.auditing) failures += 8;
        if (root.ultimaAba !== "problems") failures += 16;

        // Guarda de reentrancia: clicar de novo enquanto audita nao dispara
        // um segundo job.
        jobs.startAudit();
        if (root.pedidos !== 1) failures += 32;

        // Achado vira problema com origem PROPRIA "audit". Se caisse em
        // "quality", rodar a analise estatica depois apagaria a auditoria.
        jobs.handleAuditDiagnostic({
            severity: "error", message: "RUSTSEC-2026-0001: estouro (x 1.0.0)",
            file: "Cargo.lock", line: 42
        });
        // Leitura DEFENSIVA de proposito: se o achado nao entrou, um
        // `get(0).source` estoura TypeError e o harness trava ate o timeout
        // em vez de dizer o que falhou. Check que trava e' check que esconde.
        function origemDoProblema(indice) {
            return jobs.problemsModel.count > indice
                 ? jobs.problemsModel.get(indice).source : "";
        }
        if (jobs.problemsModel.count !== 1) failures += 64;
        if (origemDoProblema(0) !== "audit") failures += 128;

        // A prova de que as duas origens NAO se atropelam.
        jobs.handleQualityDiagnostic({
            severity: "warning", message: "lint qualquer", file: "src/a.rs", line: 1
        });
        if (jobs.problemsModel.count !== 2) failures += 256;
        jobs.removeProblemsBySource("quality");
        if (jobs.problemsModel.count !== 1) failures += 512;
        if (origemDoProblema(0) !== "audit") failures += 1024;

        // Sucesso resume o resultado E diz se a base foi atualizada. Uma
        // auditoria com base velha apresentada como fresca tranquiliza sem
        // motivo — que e o pior servico que ela poderia prestar.
        jobs.handleAuditFinished(true, 0, {
            advisoryCount: 1225, lastUpdated: "2026-08-21T08:28:40+02:00", offline: true
        }, "");
        if (jobs.auditing) failures += 2048;
        if (jobs.auditSummary.indexOf("nenhuma") < 0) failures += 4096;
        if (jobs.auditSummary.indexOf("2026-08-21") < 0) failures += 8192;
        if (jobs.auditSummary.indexOf("rede") < 0) failures += 16384;

        // Com a base atualizada pela rede, o aviso de "sem atualizar" some.
        jobs.handleAuditFinished(true, 2, {
            advisoryCount: 1225, lastUpdated: "2026-08-21T08:28:40+02:00", offline: false
        }, "");
        if (jobs.auditSummary.indexOf("2") < 0) failures += 32768;
        if (jobs.auditSummary.indexOf("sem atualizar") >= 0) failures += 65536;

        // Falha repassa o erro do core, que ja diz COMO habilitar a rede.
        // Resumir para "falhou" apagaria a unica instrucao acionavel.
        jobs.startAudit();
        jobs.handleAuditFinished(false, 0, {}, "autorize allowNetwork=true");
        if (jobs.auditing) failures += 131072;
        if (jobs.auditSummary.indexOf("allowNetwork") < 0) failures += 262144;

        // clear() do workspace leva a auditoria junto.
        jobs.clear();
        if (jobs.auditSummary !== "") failures += 524288;
        if (jobs.problemsModel.count !== 0) failures += 1048576;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
