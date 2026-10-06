import QtQuick
import KineinVectis

// Mesmo nome/SQL não autoriza resposta antiga nem confirma em outro projeto.
Item {
    id: root
    property var sent: []
    DataSourceController {
        id: controller
        onQueryRequested: (name, password, sql, confirmed, maxRows, context, confirmation) => root.sent.push({
            name: name, sql: sql, confirmed: confirmed, maxRows: maxRows, context: context, confirmation: confirmation })
    }
    function check(condition, label) {
        if (!condition) console.error("FALHOU: " + label);
        return condition ? 0 : 1;
    }
    function result(operation, rows) {
        return { name: operation.name, clientContext: operation.clientContext, success: true,
                 columns: ["id"], rows: rows, rowCount: rows.length, elapsedMs: 1 };
    }
    Component.onCompleted: {
        let failures = 0;
        const profile = Object.assign(DataSourceKinds.emptyProfile(), { name: "loja", user: "autor" });
        controller.workspaceRoot = "/projeto";
        controller.handleList([profile]);
        controller.select(profile.name);
        controller.runOn("loja", "SELECT id FROM t", false, 200);
        const first = Object.assign({}, controller.lastQuery);
        failures += check(root.sent[0].context.expectedContext.workspace === "/projeto"
            && root.sent[0].context.expectedContext.profile.name === "loja", "destino público acompanha pedido");
        controller.runOn("loja", "SELECT id FROM t", false, 200);
        const second = Object.assign({}, controller.lastQuery);
        failures += check(first.clientContext !== second.clientContext, "SQL igual recebe outro contexto");
        controller.handleQueried(result(first, [["antiga"]]));
        failures += check(controller.querying && controller.queryRows.length === 0, "sucesso antigo não troca grade nem encerra pedido novo");
        controller.handleFailed("datasource.query", "confirmação antiga", "WRITE_CONFIRMATION_REQUIRED", first);
        failures += check(!controller.impact.open && controller.querying, "recusa antiga não abre aviso");
        controller.handleQueried(result(second, [["atual"]]));
        failures += check(!controller.querying && controller.queryRows[0][0] === "atual", "resultado atual aparece");

        controller.runOn("loja", "DELETE FROM t", false, 200);
        const removing = Object.assign({}, controller.lastQuery);
        controller.handleFailed("datasource.query", "confirme", "WRITE_CONFIRMATION_REQUIRED", removing);
        const staleMeasurement = controller.impact.clientContext;
        controller.impact.handleMeasured({ name: "loja", sql: "DELETE FROM t", clientContext: staleMeasurement,
            severity: "destructive", confirmationTarget: "t", statements: [] });
        controller.impact.typed = "t";
        failures += check(controller.impact.canRun, "aviso atual libera apenas alvo completo");
        controller.editDraft("host", "outro-host");
        failures += check(!controller.impact.open && controller.lastQuery === null, "editar destino cancela aviso");
        const before = root.sent.length;
        controller.impact.confirm();
        failures += check(root.sent.length === before, "aviso cancelado não escreve");

        controller.select("loja");
        controller.runOn("loja", "DELETE FROM t", false, 200);
        controller.handleFailed("datasource.query", "confirme", "WRITE_CONFIRMATION_REQUIRED", controller.lastQuery);
        controller.impact.handleMeasured({ name: "loja", sql: "DELETE FROM t", clientContext: staleMeasurement,
            severity: "destructive", confirmationTarget: "t", statements: [] });
        failures += check(controller.impact.measuring, "medição antiga de SQL igual não libera outro aviso");
        controller.impact.handleMeasured({ name: "loja", sql: "DELETE FROM t", clientContext: controller.impact.clientContext,
            severity: "destructive", confirmationTarget: "t", statements: [] });
        controller.impact.typed = "t";
        controller.impact.confirm();
        failures += check(root.sent[root.sent.length - 1].confirmed && root.sent[root.sent.length - 1].maxRows === 200,
            "confirmação conserva o teto original");

        const previous = Object.assign({}, controller.lastQuery);
        controller.workspaceRoot = "/outro-projeto";
        controller.workspaceRoot = "/projeto";
        controller.handleList([profile]);
        controller.select("loja");
        controller.runOn("loja", "DELETE FROM t", false);
        controller.handleQueried(result(previous, [["antiga"]]));
        failures += check(controller.querying && controller.queryRows.length === 0 && !controller.impact.open,
            "reabrir o mesmo caminho não aceita resultado da sessão anterior");

        const production = Object.assign({}, profile, { production: true });
        controller.handleList([production]);
        controller.select("loja");
        controller.runOn("loja", "DELETE FROM t WHERE id = 1", false);
        controller.handleFailed("datasource.query", "produção", "WRITE_CONFIRMATION_REQUIRED", controller.lastQuery);
        controller.impact.handleMeasured({ name: "loja", sql: controller.lastQuery.sql, clientContext: controller.impact.clientContext,
            severity: "write", confirmationTarget: "t", requiresConnection: true, statements: [] });
        controller.impact.typed = "t";
        failures += check(!controller.impact.canRun, "produção também exige o nome da conexão");
        controller.impact.typedConnection = "loj";
        failures += check(!controller.impact.canRun, "nome parcial da conexão não libera produção");
        controller.impact.typedConnection = "loja";
        failures += check(controller.impact.canRun, "conexão e alvo completos liberam produção");
        controller.impact.confirm();
        const confirmed = root.sent[root.sent.length - 1];
        failures += check(confirmed.confirmation.connection === "loja" && confirmed.confirmation.target === "t"
            && confirmed.context.expectedContext.profile.production === true, "nomes e destino de produção chegam ao emissor");
        if (failures !== 0) console.error("FALHAS " + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
