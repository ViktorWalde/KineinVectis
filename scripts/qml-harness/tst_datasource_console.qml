import QtQuick
import KineinVectis

// O console SQL no editor (2026-10-03): qual conexao um arquivo de console
// representa, qual instrucao o Ctrl+Enter executa, e o pedido que sai.
Item {
    id: root

    property var ran: []
    property int results: 0

    QtObject {
        id: fakeController

        property var profiles: [{ name: "loja", engine: "sqlite" }, { name: "meu pg", engine: "postgres" }]

        function runOn(name, text, confirmWrite) {
            root.ran.push(name + "|" + text);
        }
    }

    DataSourceConsoleController {
        id: consoles

        dataSourceController: fakeController
        workspaceRoot: "/p"
        onResultsRequested: root.results += 1
    }

    function check(condition, label) {
        if (!condition) console.error("FALHOU: " + label);
        return condition ? 0 : 1;
    }

    Component.onCompleted: {
        let failures = 0;
        // O nome de arquivo e' o mesmo que o core da' (datasource/console.rs).
        failures += check(consoles.fileStem("meu pg") === "meu_pg" && consoles.fileStem("../x") === "_x"
                          && consoles.fileStem("...") === "console", "fileStem igual ao do core");
        failures += check(consoles.connectionFor("/p/.kinein/consoles/loja.sql") === "loja", "console da loja");
        failures += check(consoles.connectionFor("/p/.kinein/consoles/meu_pg.sql") === "meu pg", "nome com espaco");
        failures += check(consoles.connectionFor("/p/src/main.sql") === "", "fora de .kinein/consoles nao e' console");
        failures += check(consoles.connectionFor("/p/.kinein/consoles/sumiu.sql") === "", "perfil apagado");

        const text = "-- Console da conexao loja.\nselect 1;\n\nselect *\nfrom clientes\nwhere id = 2;\nselect 3;";
        // Cursor no meio da segunda instrucao (em "clientes").
        const cursor = text.indexOf("clientes") + 3;
        failures += check(consoles.statementAt(text, cursor, cursor, cursor, false)
                          === "select *\nfrom clientes\nwhere id = 2", "a instrucao sob o cursor");
        failures += check(consoles.statementAt(text, 3, 3, 3, false) === "select 1",
                          "comentario fora: " + JSON.stringify(consoles.statementAt(text, 3, 3, 3, false)));
        // Cursor logo depois do `;` (fim da digitacao): a instrucao que acabou ali.
        failures += check(consoles.statementAt(text, text.length, text.length, text.length, false) === "select 3",
                          "depois do ultimo ;: " + JSON.stringify(consoles.statementAt(text, text.length, text.length,
                                                                                         text.length, false)));
        failures += check(consoles.statementAt("select 1;\n", 10, 10, 10, false) === "select 1",
                          "linha vazia depois do unico ;");
        failures += check(consoles.statementAt("-- so comentario\n", 5, 5, 5, false) === "", "nada a executar");
        const sel = text.indexOf("select 3");
        failures += check(consoles.statementAt(text, 0, sel, sel + 8, false) === "select 3", "a selecao vence");
        failures += check(consoles.statementAt("// mongo\nleituras {}\npedidos {\"a\": 1}", 12, 12, 12, true)
                          === "leituras {}", "Mongo: a linha");

        failures += check(consoles.runFromEditor("/p/.kinein/consoles/loja.sql", text, cursor, cursor, cursor)
                          && root.ran[0] === "loja|select *\nfrom clientes\nwhere id = 2" && root.results === 1,
                          "Ctrl+Enter executa na conexao do arquivo: " + root.ran[0]);
        failures += check(!consoles.runFromEditor("/p/src/a.sql", text, 0, 0, 0) && root.ran.length === 1,
                          "fora de console nao executa");
        consoles.tableData("loja", "sqlite", "main", "clientes");
        consoles.tableData("meu pg", "postgres", "public", "pedidos");
        failures += check(root.ran[1] === "loja|SELECT * FROM \"clientes\" LIMIT 200"
                          && root.ran[2] === "meu pg|SELECT * FROM \"public\".\"pedidos\" LIMIT 200",
                          "dados da tabela: " + root.ran[1] + " / " + root.ran[2]);

        if (failures !== 0) console.error("FALHAS " + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
