import QtQuick
import KineinVectis

// A arvore da janela do Banco (2026-10-03): conexao > esquema > tabela >
// coluna; um esquema so' pula o nivel; Mongo vira colecao > campos; o que
// ainda nao foi lido pede leitura; o que esta' aberto continua aberto
// depois de ler de novo.
Item {
    DataSourceTree {
        id: tree

        profiles: [{ name: "loja", engine: "sqlite" }, { name: "pg", engine: "postgres" },
                   { name: "sensores", engine: "mongo" }]
    }

    function names() {
        return tree.rows.map(r => "  ".repeat(r.depth) + r.name).join("/");
    }

    function check(condition, label) {
        if (!condition) console.error("FALHOU: " + label);
        return condition ? 0 : 1;
    }

    Component.onCompleted: {
        let failures = 0;
        failures += check(tree.rows.length === 3 && tree.rows[0].kind === "connection"
                          && tree.rows[0].detail === "SQLite", "tres conexoes fechadas");

        tree.toggle(tree.key(["c", "loja"]));
        failures += check(tree.rows[1].kind === "read", "aberta sem leitura pede leitura");
        tree.readingNames = { "loja": true };
        failures += check(tree.rows[1].kind === "status", "lendo");
        tree.readingNames = { "loja": false };
        tree.structures = { "loja": { schemas: [{ name: "main", tables: [
            { name: "clientes", kind: "table", columns: [{ name: "id", dataType: "INTEGER", nullable: true },
                                                         { name: "nome", dataType: "TEXT", nullable: false }] },
            { name: "vendas", kind: "view", columns: [] }] }], collections: [] } };
        failures += check(tree.rows[1].name === "clientes" && tree.rows[1].depth === 1
                          && tree.rows[2].kind === "view", "um esquema: tabelas direto na conexao");
        tree.toggle(tree.rows[1].key);
        failures += check(tree.rows[2].kind === "column" && tree.rows[3].detail === "TEXT · não nulo"
                          && tree.rows[2].table === "clientes" && tree.rows[2].connection === "loja",
                          "colunas com tipo e contexto: " + tree.rows[3].detail);

        // Varios esquemas: o nivel do esquema aparece.
        tree.toggle(tree.key(["c", "pg"]));
        tree.structures = Object.assign({}, tree.structures, { pg: { schemas: [
            { name: "public", tables: [{ name: "t", kind: "table", columns: [] }] },
            { name: "audit", tables: [] }], collections: [] } });
        const pgSchemas = tree.rows.filter(r => r.connection === "pg" && r.kind === "schema");
        failures += check(pgSchemas.length === 2 && pgSchemas[0].detail === "1 tabela", "esquemas do pg");
        // O que estava aberto (clientes) continua aberto depois da releitura.
        failures += check(tree.rows.some(r => r.kind === "column" && r.connection === "loja"),
                          "releitura mantem o aberto");

        // Mongo: colecao e campos, com tipos e presenca.
        tree.toggle(tree.key(["c", "sensores"]));
        tree.structures = Object.assign({}, tree.structures, { "sensores": { schemas: [], collections: [
            { name: "leituras", kind: "timeseries", documentCount: 3, declared: false, sampled: 3, timeField: "ts",
              fields: [{ path: "meta.placa", depth: 1, types: ["string"], presence: 0.5 }] }] } });
        const readings = tree.rows.find(r => r.name === "leituras");
        failures += check(readings.kind === "timeseries"
                          && readings.detail === "3 documentos · amostra de 3 · tempo: ts", "colecao: " + readings.detail);
        tree.toggle(readings.key);
        const field = tree.rows.find(r => r.kind === "field");
        failures += check(field.detail === "string · em 50%" && field.depth === 3, "campo: " + field.detail);

        // Falha de leitura: a mensagem e o caminho de tentar de novo.
        tree.structures = Object.assign({}, tree.structures, { pg: { schemas: [], collections: [], failed: "senha" } });
        failures += check(tree.rows.some(r => r.kind === "failed" && r.name === "senha"), "falha aparece");

        // Os mesmos separadores em posições diferentes eram uma chave só.
        tree.profiles = [{ name: "a|b", engine: "sqlite" }, { name: "a", engine: "sqlite" },
                         { name: "__proto__", engine: "sqlite" }, { name: "constructor", engine: "sqlite" }];
        tree.expanded = DataSourceMap.copy();
        tree.structures = DataSourceMap.copy(null, {
            ["a|b"]: { schemas: [{ name: "c", tables: [{ name: "d", columns: [], kind: "table" }] }], collections: [] },
            ["a"]: { schemas: [{ name: "b|c", tables: [{ name: "d", columns: [], kind: "table" }] }], collections: [] }
        });
        for (const name of ["a|b", "a", "__proto__", "constructor"]) tree.toggle(tree.key(["c", name]));
        const tables = tree.rows.filter(item => item.kind === "table");
        failures += check(tables.length === 2 && tables[0].key !== tables[1].key, "tuplas não colidem");
        tree.toggle(tables[0].key);
        failures += check(tree.rows.find(item => item.key === tables[0].key).expanded === true
            && tree.rows.find(item => item.key === tables[1].key).expanded === false, "expansão isolada");
        failures += check(tree.rows.filter(item => item.kind === "read").length === 2, "prototype não vira catálogo");
        tree.structures = DataSourceMap.copy(tree.structures, { ["__proto__"]: { schemas: [], collections: [] } });
        tree.readingNames = DataSourceMap.copy(null, { ["constructor"]: true });
        failures += check(DataSourceMap.get(tree.structures, "__proto__").schemas.length === 0
            && tree.rows.some(item => item.connection === "constructor" && item.kind === "status"), "nomes especiais conservados");
        failures += check(DataSourceMap.get({}, "constructor") === undefined, "leitura só de propriedade própria");

        if (failures !== 0) console.error("FALHAS " + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
