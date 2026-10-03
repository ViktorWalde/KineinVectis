import QtQuick
import KineinVectis

// O CONSOLE SQL NO EDITOR (2026-10-03, decisao do autor: "editar/criar
// comandos SQL no proprio campo que ja' e' usado para desenvolver codigo").
// Cada conexao salva tem um arquivo em `.kinein/consoles/<conexao>.sql`
// (`.mongo` no MongoDB), que abre como aba comum do editor — realce, desfazer,
// salvar sozinho, tudo o que o editor ja' faz. Ctrl+Enter num desses
// arquivos executa a instrucao sob o cursor (ou a selecao) na conexao do
// arquivo; o resultado aparece na secao de dados da janela do Banco.
//
// Puro na logica (qual conexao, qual instrucao); os pedidos saem por sinal.
QtObject {
    id: root

    property var dataSourceController: null
    property string workspaceRoot: ""

    signal consoleRequested(string name)
    signal openFileRequested(string path)
    signal resultsRequested()

    // O nome de arquivo que o core da' a um perfil (datasource/console.rs):
    // letras, numeros, `.`, `-` e `_`; o resto vira `_`.
    function fileStem(name) {
        let stem = "";
        for (const c of name) {
            const keep = c.toLowerCase() !== c.toUpperCase() || (c >= "0" && c <= "9") || "._-".indexOf(c) >= 0;
            stem += keep ? c : "_";
        }
        stem = stem.replace(/^\.+|\.+$/g, "");
        return stem === "" ? "console" : stem;
    }

    readonly property string consolesDir: root.workspaceRoot === "" ? "" : root.workspaceRoot + "/.kinein/consoles/"

    function isConsole(path) {
        return root.consolesDir !== "" && path.indexOf(root.consolesDir) === 0;
    }

    // A conexao de um arquivo de console; "" quando nao e' console ou o
    // perfil ja' nao existe.
    function connectionFor(path) {
        if (!root.isConsole(path) || root.dataSourceController === null) return "";
        const file = path.substring(root.consolesDir.length);
        const stem = file.replace(/\.(sql|mongo)$/, "");
        const profile = root.dataSourceController.profiles.find(item => root.fileStem(item.name) === stem);
        return profile === undefined ? "" : profile.name;
    }

    // A instrucao a executar: a selecao, se houver; senao, no SQL, o trecho
    // entre o `;` anterior e o seguinte ao cursor; no Mongo, a linha. Linhas
    // de comentario (`--`, `//`) ficam de fora. Cursor logo DEPOIS do `;`
    // (o normal ao terminar de digitar) roda a instrucao que acabou ali —
    // antes o trecho vazio depois do `;` fazia o Ctrl+Enter nao fazer nada
    // (2026-10-03, achado na tela real).
    function statementAt(text, cursor, selectionStart, selectionEnd, mongo) {
        if (selectionEnd > selectionStart) return text.substring(selectionStart, selectionEnd).trim();
        const clean = piece => piece.split("\n").filter(line => !/^\s*(--|\/\/)/.test(line)).join("\n").trim();
        if (mongo) {
            const lineStart = text.lastIndexOf("\n", cursor - 1) + 1;
            const lineEnd = text.indexOf("\n", cursor);
            return clean(text.substring(lineStart, lineEnd < 0 ? text.length : lineEnd));
        }
        const start = text.lastIndexOf(";", cursor - 1) + 1;
        const end = text.indexOf(";", cursor);
        const here = clean(text.substring(start, end < 0 ? text.length : end));
        if (here !== "" || start === 0) return here;
        return clean(text.substring(text.lastIndexOf(";", start - 2) + 1, start - 1));
    }

    function open(name) {
        root.consoleRequested(name);
    }

    function handleResolved(path) {
        root.openFileRequested(path);
    }

    // Ctrl+Enter no editor. false = nao e' console (o atalho nao faz nada).
    function runFromEditor(path, text, cursor, selectionStart, selectionEnd) {
        const name = root.connectionFor(path);
        if (name === "") return false;
        const statement = root.statementAt(text, cursor, selectionStart, selectionEnd, path.endsWith(".mongo"));
        if (statement === "") return true;
        root.dataSourceController.runOn(name, statement, false);
        root.resultsRequested();
        return true;
    }

    // Clique duplo numa tabela da arvore: as primeiras linhas dela.
    function tableData(connection, engine, schema, table) {
        const quote = name => "\"" + name.replace(/"/g, "\"\"") + "\"";
        const sql = DataSourceKinds.isMongo(engine) ? table + " {}"
                  : "SELECT * FROM " + (DataSourceKinds.isSqlite(engine) || schema === "" ? "" : quote(schema) + ".")
                    + quote(table) + " LIMIT 200";
        root.dataSourceController.runOn(connection, sql, false);
        root.resultsRequested();
    }
}
