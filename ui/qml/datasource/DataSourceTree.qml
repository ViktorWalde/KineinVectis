import QtQuick
import KineinVectis

// A ARVORE da janela do Banco (2026-10-03, decisao do autor: o explorador do
// banco acoplado ao layout, como a janela Database da JetBrains, e nao um
// painel por cima do codigo). Puro: recebe as conexoes, a estrutura lida de
// cada uma e o que esta' aberto, e devolve as LINHAS ja' achatadas, com a
// profundidade de cada uma — a janela so' desenha.
//
//   conexao                       (abre: le a estrutura se ainda nao leu)
//     esquema                     so' quando ha' mais de um
//       tabela / visao
//         coluna  tipo · nulo
//   conexao Mongo
//     colecao / serie temporal
//       campo  tipos · presenca
//
// Tuplas JSON identificam os objetos sem confundir nomes que contêm |.
// As chaves continuam estáveis entre leituras, preservando a expansão.
QtObject {
    id: root

    property var profiles: []
    property var structures: DataSourceMap.copy()
    property var readingNames: DataSourceMap.copy()
    property var expanded: DataSourceMap.copy()

    readonly property var rows: root.buildRows()

    function key(parts) { return JSON.stringify(parts); }

    function isExpanded(key) {
        return DataSourceMap.get(root.expanded, key) === true;
    }

    function toggle(key) {
        const next = DataSourceMap.copy(root.expanded);
        next[key] = !root.isExpanded(key);
        root.expanded = next;
    }

    function plural(count, one, many) {
        return count === 1 ? one : many.arg(count);
    }

    function row(key, depth, kind, name, detail, expandable, extra) {
        const base = { key: key, depth: depth, kind: kind, name: name, detail: detail,
                       expandable: expandable, expanded: expandable && root.isExpanded(key) };
        return Object.assign(base, extra || {});
    }

    function columnRows(out, prefix, depth, table, context) {
        for (const column of table.columns) {
            out.push(root.row(root.key(prefix.concat(["column", column.name])), depth, "column", column.name,
                              column.dataType + (column.nullable ? "" : qsTr(" · não nulo")), false, context));
        }
    }

    function tableRows(out, prefix, depth, tables, context) {
        for (const table of tables) {
            const parts = prefix.concat([table.name]);
            const key = root.key(parts);
            const extra = Object.assign({ table: table.name, readSql: table.readSql || "" }, context);
            out.push(root.row(key, depth, DataSourceKinds.tableKind(table.kind), table.name,
                              root.plural(table.columns.length, qsTr("1 coluna"), qsTr("%1 colunas")), true, extra));
            if (root.isExpanded(key)) root.columnRows(out, parts, depth + 1, table, extra);
        }
    }

    function collectionDetail(collection) {
        let text = collection.documentCount !== undefined && collection.documentCount !== null
                   ? root.plural(collection.documentCount, qsTr("1 documento"), qsTr("%1 documentos"))
                   : qsTr("visão");
        if (collection.declared === true) text += qsTr(" · esquema declarado");
        else if (collection.sampled > 0) text += qsTr(" · amostra de %1").arg(collection.sampled);
        if (DataSourceKinds.isTimeseries(collection.kind) && collection.timeField) {
            text += qsTr(" · tempo: %1").arg(collection.timeField);
        }
        return text;
    }

    function collectionRows(out, prefix, collections, context) {
        for (const collection of collections) {
            const parts = prefix.concat([collection.name]);
            const key = root.key(parts);
            const extra = Object.assign({ table: collection.name }, context);
            out.push(root.row(key, 1, DataSourceKinds.collectionKind(collection.kind), collection.name, root.collectionDetail(collection), true, extra));
            if (!root.isExpanded(key) || collection.fields === undefined) continue;
            for (const field of collection.fields) {
                const presence = field.presence !== undefined && field.presence !== null && field.presence < 1
                                 ? qsTr(" · em %1%").arg(Math.round(field.presence * 100)) : "";
                out.push(root.row(root.key(parts.concat(["field", field.path])), 2 + field.depth, "field", field.path,
                                  field.types.join(" | ") + presence, false, extra));
            }
        }
    }

    function structureRows(out, profile) {
        const name = profile.name;
        const context = { connection: name, engine: profile.engine };
        const structure = DataSourceMap.get(root.structures, name);
        if (DataSourceMap.get(root.readingNames, name) === true) {
            out.push(root.row(root.key(["r", name]), 1, "status", qsTr("Lendo a estrutura…"), "", false, context));
            return;
        }
        if (structure === undefined) {
            out.push(root.row(root.key(["r", name]), 1, "read", qsTr("Ler a estrutura"), "", false, context));
            return;
        }
        if (structure.failed !== undefined) {
            out.push(root.row(root.key(["r", name]), 1, "failed", structure.failed, qsTr("tentar de novo"), false, context));
            return;
        }
        if (DataSourceKinds.isMongo(profile.engine)) {
            root.collectionRows(out, ["m", name], structure.collections, context);
            return;
        }
        // Um esquema so' (o "main" do SQLite): as tabelas direto na conexao.
        if (structure.schemas.length === 1) {
            const schema = structure.schemas[0];
            root.tableRows(out, ["t", name, schema.name], 1, schema.tables,
                           Object.assign({ schema: schema.name }, context));
            return;
        }
        for (const schema of structure.schemas) {
            const key = root.key(["s", name, schema.name]);
            const extra = Object.assign({ schema: schema.name }, context);
            out.push(root.row(key, 1, "schema", schema.name,
                              root.plural(schema.tables.length, qsTr("1 tabela"), qsTr("%1 tabelas")), true, extra));
            if (root.isExpanded(key)) root.tableRows(out, ["t", name, schema.name], 2, schema.tables, extra);
        }
    }

    function buildRows() {
        const out = [];
        for (const profile of root.profiles) {
            const key = root.key(["c", profile.name]);
            const policy = DataSourceKinds.policyLabel(profile);
            out.push(root.row(key, 0, "connection", profile.name, DataSourceKinds.engineName(profile.engine) + (policy ? " · " + policy : ""), true,
                              { connection: profile.name, engine: profile.engine, production: profile.production === true }));
            if (root.isExpanded(key)) root.structureRows(out, profile);
        }
        return out;
    }
}
