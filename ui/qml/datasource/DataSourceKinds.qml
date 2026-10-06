pragma Singleton
import QtQuick

// QUAL MOTOR e QUE TIPO DE OBJETO — dono unico (2026-10-03). A janela do
// Banco, a arvore, o dialogo da conexao e o console perguntam daqui: a mesma
// derivacao espalhada em cinco arquivos e' como uma tela passa a chamar de
// "visao" o que outra chama de "tabela" sem ninguem perceber (catraca de
// duplicacao QML).
QtObject {
    function isMongo(engine) {
        return engine === "mongo";
    }

    function isSqlite(engine) {
        return engine === "sqlite";
    }

    function engineIcon(engine) {
        return isSqlite(engine) ? "file" : (isMongo(engine) ? "documents" : "database");
    }

    function engineName(engine) {
        return isSqlite(engine) ? "SQLite" : (isMongo(engine) ? "MongoDB" : "PostgreSQL");
    }

    function engineShort(engine) {
        return isSqlite(engine) ? "SQLite" : (isMongo(engine) ? "Mongo" : "PG");
    }

    // Os padroes de cada motor (2026-10-04): trocar o motor no formulario
    // trazia o PostgreSQL junto (`/var/run/postgresql`, 5432) para o MongoDB.
    function defaultsFor(engine) {
        if (isMongo(engine)) return { host: "localhost", port: 27017, database: "test" };
        if (isSqlite(engine)) return { host: "", port: 0, database: "" };
        return { host: "/var/run/postgresql", port: 5432, database: "postgres" };
    }

    // No rascunho `draft` (ja' com o motor novo), troca pelos padroes do
    // motor novo SO' os campos que ainda estao no padrao do anterior (ou
    // vazios): o que a pessoa digitou fica.
    function adoptDefaults(draft, fromEngine, toEngine) {
        const before = defaultsFor(fromEngine);
        const after = defaultsFor(toEngine);
        for (const key of ["host", "port", "database"]) {
            if (draft[key] === before[key] || draft[key] === "" || draft[key] === undefined) draft[key] = after[key];
        }
    }

    // Traducao do resultado: Mongo conta documentos; SQL conta linhas.
    function querySummary(outcome, engine) {
        if (outcome.affected !== undefined && outcome.affected !== null) {
            return (isMongo(engine) ? qsTr("%1 documento(s) afetado(s) em %2 ms")
                                   : qsTr("%1 linha(s) afetada(s) em %2 ms"))
                .arg(outcome.affected).arg(outcome.elapsedMs);
        }
        return qsTr("%1 linha(s)%2 em %3 ms").arg(outcome.rowCount)
            .arg(outcome.truncated ? qsTr(" — teto atingido") : "").arg(outcome.elapsedMs);
    }

    function isTimeseries(kind) {
        return kind === "timeseries";
    }

    // Como a tela desenha uma tabela (SQL) ou uma colecao (Mongo).
    function tableKind(kind) {
        return kind === "view" ? "view" : "table";
    }

    function collectionKind(kind) {
        return isTimeseries(kind) ? "timeseries" : (kind === "view" ? "view" : "collection");
    }

    function kindIcon(kind) {
        return ({ schema: "schema", table: "table", view: "eye", collection: "documents",
                  timeseries: "observability", read: "refresh", failed: "warning" })[kind] || "";
    }

    // Tem linhas para mostrar na secao de dados (clique duplo na arvore)?
    function hasData(kind) {
        return ["table", "view", "collection", "timeseries"].indexOf(kind) >= 0;
    }
}
