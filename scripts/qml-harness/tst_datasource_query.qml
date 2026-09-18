import QtQuick
import "../../ui/qml/datasource"

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

    property var pedidos: []

    DataSourceController {
        id: c

        onQueryRequested: function(name, password, sql, confirmWrite) {
            root.pedidos.push({ name: name, password: password, sql: sql, confirmWrite: confirmWrite });
        }
    }

    Component.onCompleted: {
        let failures = 0;
        c.workspaceRoot = "/w";

        // Sem perfil nomeado ou sem texto: nada sai.
        c.sql = "SELECT 1";
        c.runQuery(false);
        c.editDraft("name", "local");
        c.sql = "   ";
        c.runQuery(false);
        if (root.pedidos.length !== 0 || c.querying) failures += 1;

        // O pedido leva a senha da sessao e confirmWrite falso.
        c.sessionPassword = "s3nha";
        c.sql = "SELECT id FROM t";
        c.runQuery();
        if (root.pedidos.length !== 1 || !c.querying || root.pedidos[0].password !== "s3nha"
                || root.pedidos[0].confirmWrite !== false || root.pedidos[0].name !== "local") failures += 2;

        // O resultado vira grade e status.
        c.handleQueried({ success: true, columns: ["id", "nome"], rows: [["1", "a"], ["2", null]], rowCount: 2, truncated: true, elapsedMs: 3 });
        if (c.querying || c.queryColumns.length !== 2 || c.queryRows.length !== 2 || c.queryRows[1][1] !== null) failures += 4;
        if (c.queryStatus.indexOf("2 linha(s)") !== 0 || c.queryStatus.indexOf("teto") < 0) failures += 8;

        // A escrita: recusa pelo CODIGO abre a confirmacao; o reenvio vai com true.
        c.sql = "DELETE FROM t";
        c.runQuery(false);
        c.handleFailed("datasource.query", "esta instrucao ESCREVE", "WRITE_CONFIRMATION_REQUIRED");
        if (c.querying || !c.writeConfirmationRequired || c.errorText !== "" || c.queryStatus.indexOf("ESCREVE") < 0) failures += 16;
        c.runQuery(true);
        if (root.pedidos.length !== 3 || root.pedidos[2].confirmWrite !== true || c.writeConfirmationRequired) failures += 32;
        c.handleQueried({ success: true, columns: [], rows: [], rowCount: 0, affected: 4, truncated: false, elapsedMs: 1 });
        if (c.queryStatus.indexOf("4 linha(s) afetada(s)") !== 0 || c.queryColumns.length !== 0) failures += 64;

        // A falha do motor vira texto; a senha pedida abre o campo.
        c.handleQueried({ success: false, message: "relation \"x\" does not exist", secretRequired: false });
        if (c.queryStatus.indexOf("relation") !== 0 || c.secretRequired) failures += 128;
        c.handleQueried({ success: false, message: "senha", secretRequired: true });
        if (!c.secretRequired) failures += 256;
        c.handleFailed("datasource.query", "a variavel nao esta definida", "SECRET_REQUIRED");
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
