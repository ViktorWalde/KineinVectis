import QtQuick
import KineinVectis

Item {
    id: root
    property var sent: []
    DataSourceController {
        id: controller
        onQueryRequested: (name, password, sql, confirmed, maxRows, context, confirmation) => root.sent.push({
            name: name, password: password, sql: sql, confirmed: confirmed, maxRows: maxRows,
            context: context, confirmation: confirmation })
    }
    function check(condition, label) {
        if (!condition) console.error("FALHOU: " + label);
        return condition ? 0 : 1;
    }
    Component.onCompleted: {
        let failures = 0;
        const first = Object.assign(DataSourceKinds.emptyProfile(), { name: "origem", host: "db-a" });
        const second = Object.assign(DataSourceKinds.emptyProfile(), { name: "destino", host: "db-b" });
        controller.workspaceRoot = "/projeto";
        controller.handleList([first, second]);
        controller.select(first.name);
        controller.sessionPassword = "senha-da-origem";
        controller.runOn(second.name, "SELECT id FROM t", false, 123);
        const original = Object.assign({}, controller.lastQuery);
        controller.handleFailed("datasource.query", "senha", "SECRET_REQUIRED", original);
        failures += check(controller.selectedName === second.name && controller.panelVisible && controller.secretRequired
            && controller.sessionPassword === "", "senha pedida pelo console seleciona o destino certo e limpa a outra");
        failures += check(controller.secrets.pending.sql === original.sql
            && JSON.stringify(controller.secrets.pending).indexOf("senha-da-origem") < 0
            && controller.secrets.pending.password === undefined, "pedido pendente guarda somente intenção pública");
        controller.sessionPassword = "senha-do-destino";
        controller.retryWithSecret();
        const retried = root.sent[root.sent.length - 1];
        failures += check(retried.name === second.name && retried.sql === original.sql && retried.maxRows === 123
            && retried.password === "senha-do-destino" && !retried.confirmed
            && retried.context.clientContext !== original.clientContext, "Enter repete SQL e teto com um contexto novo");
        controller.handleQueried({ name: original.name, clientContext: original.clientContext, success: true,
            columns: ["id"], rows: [["antiga"]], rowCount: 1 });
        failures += check(controller.querying && controller.queryRows.length === 0, "resposta anterior à senha não encerra a retomada");
        controller.handleFailed("datasource.query", "senha", "SECRET_REQUIRED", controller.lastQuery);
        controller.editDraft("host", "outro-destino");
        const count = root.sent.length;
        controller.retryWithSecret();
        failures += check(root.sent.length === count && controller.secrets.pending === null && controller.sessionPassword === "",
            "editar destino revoga senha e retomada");
        controller.select(second.name);
        controller.runOn(second.name, "SELECT 2", false);
        controller.handleFailed("datasource.query", "senha", "SECRET_REQUIRED", controller.lastQuery);
        controller.close();
        controller.retryWithSecret();
        failures += check(controller.secrets.pending === null && root.sent.length === count + 1, "fechar cancela a retomada");
        controller.select(second.name);
        controller.runOn(second.name, "INSERT INTO t VALUES (1)", false, 71);
        controller.handleFailed("datasource.query", "confirme", "WRITE_CONFIRMATION_REQUIRED", controller.lastQuery);
        const measure = { name: second.name, clientContext: controller.impact.clientContext };
        controller.handleFailed("datasource.impact", "senha", "SECRET_REQUIRED", measure);
        failures += check(!controller.impact.open && controller.secretRequired
            && controller.secrets.pending.sql === "INSERT INTO t VALUES (1)"
            && controller.secrets.pending.confirmWrite === false, "senha pedida pela medição conserva o comando sem confirmar escrita");
        controller.sessionPassword = "senha-do-destino";
        controller.retryWithSecret();
        const afterImpact = root.sent[root.sent.length - 1];
        failures += check(afterImpact.sql === "INSERT INTO t VALUES (1)" && !afterImpact.confirmed
            && afterImpact.maxRows === 71 && afterImpact.password === "senha-do-destino",
            "retomar medição exige novamente o aceite da escrita e mantém o teto");
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
