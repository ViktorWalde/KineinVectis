import QtQuick
import KineinVectis

// Trocar motor preserva o que o autor digitou e limpa o segredo anterior.
// A amostra Mongo sobrevive a editar outro campo e ao save.
Item {
    id: root
    property var saved: null
    DataSourceController {
        id: ctl
        onSaveRequested: profile => root.saved = profile
    }
    function check(ok, message) {
        if (!ok) console.error("FALHOU: " + message);
        return ok ? 0 : 1;
    }
    Component.onCompleted: {
        let failures = 0;
        ctl.editDraft("engine", "mongo");
        failures += check(ctl.draft.host === "localhost" && ctl.draft.port === 27017
                          && ctl.draft.database === "test", "padroes Mongo");
        ctl.editDraft("sampleSize", 37);
        ctl.editDraft("name", "sensores");
        ctl.save();
        failures += check(root.saved.sampleSize === 37, "amostra sobrevive ao save");
        ctl.sessionPassword = "somente-mongo";
        ctl.editDraft("engine", "postgres");
        failures += check(ctl.sessionPassword === "", "segredo nao vai para outro motor");
        failures += check(ctl.draft.host === "/var/run/postgresql" && ctl.draft.port === 5432
                          && ctl.draft.database === "postgres", "padroes PostgreSQL");
        ctl.editDraft("host", "servidor-do-autor");
        ctl.editDraft("port", 7777);
        ctl.editDraft("database", "app");
        ctl.editDraft("engine", "mongo");
        failures += check(ctl.draft.host === "servidor-do-autor" && ctl.draft.port === 7777
                          && ctl.draft.database === "app", "campos personalizados preservados");
        ctl.startNew();
        ctl.editDraft("engine", "sqlite");
        failures += check(ctl.draft.host === "" && ctl.draft.port === 0 && ctl.draft.database === "",
                          "SQLite nao herda rede nem banco Postgres");
        ctl.editDraft("engine", "mongo");
        failures += check(ctl.draft.port === 27017, "porta vazia do SQLite adota Mongo");
        ctl.handleList([Object.assign({}, root.saved)]);
        ctl.runOn("sensores", "s.updateMany({}, {})", false);
        ctl.handleQueried({ name: "sensores", clientContext: ctl.lastQuery.clientContext, success: false, confirmationSql: "outra consulta" });
        failures += check(!ctl.impact.open && ctl.querying, "recusa antiga nao abre aviso novo");
        ctl.handleQueried({ name: "sensores", clientContext: ctl.lastQuery.clientContext, success: false, confirmationSql: "s.updateMany({}, {})" });
        failures += check(ctl.impact.open && ctl.impact.measuring
                          && ctl.impact.sql === "s.updateMany({}, {})", "preflight abre impacto");
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
