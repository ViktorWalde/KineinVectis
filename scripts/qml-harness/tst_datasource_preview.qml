import QtQuick
import KineinVectis

Item {
    id: root
    width: 1000
    height: 800
    property var decisions: []
    property var cancellations: []
    DataSourceController { id: controller }
    Connections {
        target: controller.previews
        function onDecisionRequested(operation) { root.decisions.push(operation); }
        function onCancelExecutionRequested(jobId) { root.cancellations.push(jobId); }
    }
    DataSourcePreviewDialog {
        id: dialog
        controller: controller.previews
        visible: controller.previews.open
        onDismissRequested: controller.previews.decide("rollback")
    }
    function check(condition, label) {
        if (!condition) console.error("FALHOU: " + label);
        return condition ? 0 : 1;
    }
    function prepared(query, id) {
        return { name: query.name, clientContext: query.clientContext, jobId: id, previewId: "preview-" + id,
            expiresInSeconds: 60, sql: query.sql, executedSql: query.sql + " RETURNING *",
            columns: ["id"], rows: [["1"]], affected: 1, truncated: false };
    }
    Component.onCompleted: {
        let failures = 0;
        const profile = Object.assign(DataSourceKinds.emptyProfile(), { name: "local", host: "db", production: true });
        controller.workspaceRoot = "/projeto";
        controller.handleList([profile]);
        controller.select(profile.name);
        controller.runOn(profile.name, "INSERT INTO t VALUES (1)", false, 23, null, true);
        const first = Object.assign({}, controller.lastQuery);
        failures += check(first.preview === true && first.confirmWrite === false, "gesto de prévia não confirma a escrita");
        controller.handleFailed("datasource.query", "confirme", "WRITE_CONFIRMATION_REQUIRED", first);
        controller.impact.handleMeasured({ name: first.name, sql: first.sql, clientContext: controller.impact.clientContext,
            severity: "write", statements: [], previewEligible: true });
        failures += check(controller.impact.previewSelected && controller.impact.canRun, "elegibilidade vem do core e conserva o gesto");
        controller.impact.confirm();
        const original = Object.assign({}, controller.lastQuery);
        failures += check(original.preview === true && original.confirmWrite === true && original.maxRows === 23,
            "confirmação conserva prévia e teto");
        controller.queries.accepted({ name: original.name, clientContext: original.clientContext, jobId: "job-1", preview: true });
        controller.previews.prepared(prepared(original, "job-1"));
        failures += check(controller.previews.open && controller.previews.canCommit && controller.querying, "prévia mantém o job pendente");
        dialog.dismissFromKeyboard();
        failures += check(root.decisions[0].decision === "rollback" && controller.previews.deciding, "Escape pede rollback sem presumir sucesso");
        controller.previews.decide("commit");
        failures += check(root.decisions.length === 1, "decisão pendente não permite um segundo envio");
        controller.handleQueried({ name: original.name, clientContext: original.clientContext, success: true,
            previewOutcome: "rolledBack", message: "desfeita", access: "write", columns: [], rows: [], affected: 1 });
        failures += check(!controller.previews.open && !controller.lastQuery.wrote && controller.queryStatus === "desfeita",
            "rollback não se apresenta como escrita persistida");
        controller.runOn(profile.name, "UPDATE t SET v=2 WHERE id=1", true, 20, null, true);
        const second = Object.assign({}, controller.lastQuery);
        controller.previews.prepared(prepared(second, "job-2"));
        controller.runOn(profile.name, "SELECT 1", false);
        failures += check(root.decisions[1].decision === "rollback" && !controller.previews.open, "nova consulta descarta a transação anterior");
        controller.previews.prepared(prepared(second, "job-2"));
        failures += check(root.cancellations.indexOf("job-2") >= 0 && !controller.previews.open, "evento atrasado é cancelado e não reabre o diálogo");
        controller.runOn(profile.name, "INSERT INTO t VALUES (2)", true, 20, null, true);
        const third = Object.assign({}, controller.lastQuery);
        controller.workspaceRoot = "/outro";
        controller.queries.accepted({ name: third.name, clientContext: third.clientContext, jobId: "job-3", preview: true });
        failures += check(root.cancellations.indexOf("job-3") >= 0, "aceite atrasado após trocar projeto cancela a execução");
        controller.handleList([profile]);
        controller.select(profile.name);
        controller.runOn(profile.name, "INSERT INTO t VALUES (4)", true, 20, null, true);
        const fourth = Object.assign({}, controller.lastQuery);
        controller.previews.prepared(prepared(fourth, "job-4"));
        controller.previews.remaining = 0;
        const beforeExpired = root.decisions.length;
        controller.previews.decide("commit");
        failures += check(!controller.previews.canCommit && root.decisions.length === beforeExpired,
            "prazo local vencido não envia COMMIT");
        controller.previews.decide("rollback");
        failures += check(root.decisions.length === beforeExpired + 1 && root.decisions[beforeExpired].decision === "rollback",
            "prazo vencido ainda permite pedir descarte");
        controller.handleQueried({ name: fourth.name, clientContext: fourth.clientContext, success: true,
            previewOutcome: "expired", message: "expirada", access: "write", columns: [], rows: [] });
        controller.runOn(profile.name, "INSERT INTO t VALUES (5)", true, 20, null, true);
        const fifth = Object.assign({}, controller.lastQuery);
        controller.previews.prepared(prepared(fifth, "job-5"));
        controller.previews.decide("commit");
        controller.handleQueried({ name: fifth.name, clientContext: fifth.clientContext, success: false,
            previewOutcome: "unknown", message: "resultado desconhecido", access: "write", columns: [], rows: [] });
        failures += check(!controller.previews.open && !controller.lastQuery.wrote && controller.queryStatus === "resultado desconhecido",
            "resposta perdida não se apresenta como escrita confirmada");
        controller.runOn(profile.name, "INSERT INTO t VALUES (6)", true, 20, null, true);
        controller.previews.prepared(prepared(controller.lastQuery, "job-6"));
        controller.previews.decide("commit");
        const beforeChange = root.decisions.length;
        controller.workspaceRoot = "/terceiro";
        failures += check(root.decisions.length === beforeChange && !controller.previews.open,
            "troca de projeto não tenta revogar COMMIT já aceito");
        controller.handleList([profile]);
        controller.select(profile.name);
        controller.runOn(profile.name, "INSERT INTO t VALUES (7)", true, 29, null, true);
        const needsSecret = Object.assign({}, controller.lastQuery);
        controller.handleFailed("datasource.query", "senha", "SECRET_REQUIRED", needsSecret);
        failures += check(controller.panelVisible && controller.secrets.pending.preview === true,
            "senha conserva intenção de prévia");
        controller.sessionPassword = "senha-da-prova";
        controller.retryWithSecret();
        failures += check(!controller.panelVisible && controller.lastQuery.preview === true
            && controller.lastQuery.sql === needsSecret.sql && controller.lastQuery.maxRows === 29
            && controller.passwordFor(profile.name) === "senha-da-prova",
            "Enter retoma prévia sem empilhar formulário ou apagar credencial necessária");
        Qt.callLater(() => Qt.exit(failures === 0 ? 0 : 1));
    }
}
