import QtQuick
import "../../ui/qml/datasource"
// O singleton so' e' singleton pelo modulo; o diretorio o veria como tipo.
import KineinVectis as Kv

// A consulta (0.121.0): o DataSourceController REAL com o roteador falso.
//
// O que se prova: executar so' com perfil nomeado e texto; o pedido leva a
// senha da sessao e `confirmWrite` falso; a recusa WRITE_CONFIRMATION_REQUIRED
// (pelo CODIGO) abre a confirmacao e o reenvio vai com `true`; o resultado
// vira colunas/linhas e status; a escrita mostra `affected`; a falha vira
// texto e `secretRequired` abre o campo de senha; trocar de perfil limpa; o
// perfil clonado leva `tls`/`caFile` e nunca senha.
Item {
    id: root

    property var requests: []
    property string queryText: ""

    DataSourceController {
        id: c

        onQueryRequested: function(name, password, sql, confirmWrite, maxRows) {
            root.requests.push({ name: name, password: password, sql: sql, confirmWrite: confirmWrite, maxRows: maxRows });
        }
    }

    function outcome(value) {
        return Object.assign({ name: c.lastQuery.name, clientContext: c.lastQuery.clientContext }, value);
    }

    Component.onCompleted: {
        let failures = 0;
        c.workspaceRoot = "/w";

        // Sem perfil nomeado ou sem texto: nada sai.
        root.queryText = "SELECT 1";
        c.runOn(c.draft.name, root.queryText, false);
        c.editDraft("name", "local");
        root.queryText = "   ";
        c.runOn(c.draft.name, root.queryText, false);
        if (root.requests.length !== 0 || c.querying) failures += 1;

        // O pedido leva a senha da sessao e confirmWrite falso.
        c.handleList([c.cloneProfile(c.draft)]);
        c.select("local");
        c.sessionPassword = "s3nha";
        root.queryText = "SELECT id FROM t";
        c.runOn(c.draft.name, root.queryText);
        if (root.requests.length !== 1 || !c.querying || root.requests[0].password !== "s3nha"
                || root.requests[0].confirmWrite !== false || root.requests[0].name !== "local") failures += 2;

        // O resultado vira grade e status.
        c.handleQueried(root.outcome({ success: true, columns: ["id", "nome"], rows: [["1", "a"], ["2", null]], rowCount: 2, truncated: true, elapsedMs: 3 }));
        if (c.querying || c.queryColumns.length !== 2 || c.queryRows.length !== 2 || c.queryRows[1][1] !== null) failures += 4;
        if (c.queryStatus.indexOf("2 linha(s)") !== 0 || c.queryStatus.indexOf("teto") < 0) failures += 8;

        // CARREGAR MAIS (passo 14): sem `access` nao ha' leitura provada, nada
        // a carregar; a leitura cortada repete o MESMO texto com o dobro do teto.
        if (Kv.DataSourceKinds.nextRowCeiling(c.lastQuery) !== 0) failures += 16384;
        c.runOn(c.draft.name, "SELECT id FROM t", false, 500);
        c.handleQueried(root.outcome({ success: true, columns: ["id"], rows: [["1"]], rowCount: 500, truncated: true, access: "read", elapsedMs: 3 }));
        c.queries.loadMore();
        const more = root.requests[root.requests.length - 1];
        if (root.requests.length !== 3 || more.sql !== "SELECT id FROM t" || more.maxRows !== 1000
                || more.confirmWrite !== false || !c.querying) failures += 32768;
        // Ja' pedindo: um segundo clique nao duplica a leitura.
        c.queries.loadMore();
        if (root.requests.length !== 3) failures += 65536;
        // O teto do contrato e' o fim; inteira, escrita ou previa nao repetem.
        c.handleQueried(root.outcome({ success: true, columns: ["id"], rows: [["1"]], rowCount: 1000, truncated: true, access: "read", elapsedMs: 3 }));
        if (Kv.DataSourceKinds.nextRowCeiling(c.lastQuery) !== 2000
                || Kv.DataSourceKinds.nextRowCeiling(Object.assign({}, c.lastQuery, { rowCount: 8000 })) !== 10000
                || Kv.DataSourceKinds.nextRowCeiling(Object.assign({}, c.lastQuery, { rowCount: 10000 })) !== 0
                || Kv.DataSourceKinds.nextRowCeiling(Object.assign({}, c.lastQuery, { truncated: false })) !== 0
                || Kv.DataSourceKinds.nextRowCeiling(Object.assign({}, c.lastQuery, { access: "write" })) !== 0
                || Kv.DataSourceKinds.nextRowCeiling(Object.assign({}, c.lastQuery, { preview: true })) !== 0
                || Kv.DataSourceKinds.nextRowCeiling(null) !== 0) failures += 131072;
        root.requests = root.requests.slice(0, 1);

        // A escrita: recusa pelo CODIGO abre a confirmacao; o reenvio vai com true.
        root.queryText = "DELETE FROM t";
        c.runOn(c.draft.name, root.queryText, false);
        c.handleFailed("datasource.query", "esta instrucao ESCREVE", "WRITE_CONFIRMATION_REQUIRED", c.lastQuery);
        if (c.querying || !c.writeConfirmationRequired || c.errorText !== "" || c.queryStatus.indexOf("ESCREVE") < 0) failures += 16;
        c.runOn(c.draft.name, root.queryText, true);
        if (root.requests.length !== 3 || root.requests[2].confirmWrite !== true || c.writeConfirmationRequired) failures += 32;
        c.handleQueried(root.outcome({ success: true, columns: [], rows: [], rowCount: 0, affected: 4, truncated: false, elapsedMs: 1 }));
        if (c.queryStatus.indexOf("4 linha(s) afetada(s)") !== 0 || c.queryColumns.length !== 0) failures += 64;

        // A falha do motor vira texto; a senha pedida abre o campo.
        c.runOn(c.draft.name, "SELECT * FROM x", false);
        c.handleQueried(root.outcome({ success: false, message: "relation \"x\" does not exist", secretRequired: false }));
        if (c.queryStatus.indexOf("relation") !== 0 || c.secretRequired) failures += 128;
        c.runOn(c.draft.name, "SELECT 1", false);
        c.handleQueried(root.outcome({ success: false, message: "senha", secretRequired: true }));
        if (!c.secretRequired) failures += 256;
        c.runOn(c.draft.name, "SELECT 1", false);
        c.handleFailed("datasource.query", "a variavel nao esta definida", "SECRET_REQUIRED", c.lastQuery);
        if (!c.secretRequired || c.errorText !== "") failures += 512;
        // Outra recusa continua sendo erro de produto.
        c.handleFailed("datasource.save", "informe o host", "INVALID_PARAMS");
        if (c.errorText !== "informe o host") failures += 1024;

        // Trocar de perfil limpa a consulta; o clone leva tls/caFile e nunca senha.
        c.handleList([{ name: "seguro", host: "db", port: 5432, database: "app", user: "u", tls: "require", caFile: "/etc/ssl/db.pem" }]);
        c.select("seguro");
        if (c.queryColumns.length !== 0 || c.queryStatus !== "" || c.writeConfirmationRequired) failures += 2048;
        const clone = c.cloneProfile(c.draft);
        if (clone.tls !== "require" || clone.caFile !== "/etc/ssl/db.pem" || clone.password !== undefined) failures += 4096;
        if (c.emptyDraft().tls !== "disable") failures += 8192;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
