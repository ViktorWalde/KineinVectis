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
