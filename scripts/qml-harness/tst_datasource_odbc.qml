import QtQuick
import KineinVectis

// Consentimento do controller real: cancelar/stale nunca reenvia a consulta.
Item {
    id: root
    property var approvals: []
    property var queries: []
    DataSourceController {
        id: controller
        onQueryRequested: (name, password, sql, confirmed, maxRows) =>
            root.queries.push({ name: name, sql: sql, confirmed: confirmed, maxRows: maxRows })
    }
    Connections {
        target: controller.odbc
        function onAuthorizeRequested(name, identity, workspace) {
            root.approvals.push({ name: name, identity: identity, workspace: workspace });
        }
    }
    function check(condition, label) {
        if (!condition) console.error("FALHOU: " + label);
        return condition ? 0 : 1;
    }
    Component.onCompleted: {
        let failures = 0;
        controller.workspaceRoot = "/projeto";
        controller.profiles = [{ name: "outro", engine: "odbc", database: "dsn" }];
        controller.editDraft("host", "servidor-antigo");
        controller.editDraft("engine", "odbc");
        failures += check(controller.draft.host === "" && controller.draft.port === 0
                          && controller.draft.database === "", "ODBC usa DSN e limpa rede anterior");
        controller.odbc.handleSources([{ dsn: "dsn", driver: "driver", identity: "fonte" }]);
        failures += check(controller.odbc.sources.length === 1 && root.approvals.length === 0, "listar nao autoriza");
        controller.runOn("outro", "SELECT * FROM t", false, 200);
        const details = { name: "outro", dsn: "dsn", driver: "driver", identity: "desafio", workspace: "/projeto",
                          query: { name: "outro", sql: "SELECT * FROM t", confirmWrite: false, maxRows: 200 } };
        controller.odbc.handleRequired("datasource.query", details);
        failures += check(controller.odbc.open && root.approvals.length === 0, "recusa abre aviso sem gesto automatico");
        controller.odbc.cancel();
        controller.odbc.handleAuthorized("outro", "desafio", "/projeto");
        failures += check(root.queries.length === 1 && !controller.odbc.open, "cancelar nao conecta nem retoma");
        controller.odbc.handleRequired("datasource.query", details);
        controller.odbc.confirm();
        controller.odbc.confirm();
        failures += check(root.approvals.length === 1 && controller.odbc.authorizing, "um gesto, um pedido");
        controller.odbc.handleAuthorized("outro", "desafio", "/antigo");
        failures += check(root.queries.length === 1, "projeto antigo ignorado");
        controller.odbc.handleAuthorized("outro", "desafio", "/projeto");
        failures += check(root.queries.length === 2 && root.queries[1].maxRows === 200
                          && !root.queries[1].confirmed && !controller.odbc.open, "reenvia somente a leitura original com teto");
        controller.odbc.handleRequired("datasource.query", details);
        controller.odbc.confirm();
        controller.runOn("outro", "DELETE FROM t", false);
        controller.odbc.handleAuthorized("outro", "desafio", "/projeto");
        failures += check(root.queries.length === 3 && !root.queries[2].confirmed, "consulta mudada nao ganha confirmacao antiga");
        controller.runOn("outro", "SELECT * FROM t", false);
        controller.odbc.handleRequired("datasource.query", details);
        controller.profiles = [{ name: "outro", engine: "odbc", database: "outro-dsn" }];
        controller.odbc.confirm();
        failures += check(root.approvals.length === 2, "perfil alterado exige outro aviso");
        controller.workspaceRoot = "/outro-projeto";
        failures += check(!controller.odbc.open && controller.lastQuery === null, "trocar projeto limpa consulta e consentimento");
        controller.odbc.handleRequired("datasource.query", details);
        failures += check(!controller.odbc.open, "recusa do projeto anterior nao abre aviso novo");
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
