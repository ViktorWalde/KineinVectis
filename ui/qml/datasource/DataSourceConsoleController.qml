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

    // A instrucao a executar: a selecao, se houver; senao, no Mongo, a linha;
    // no SQL, o trecho sob o cursor, e um trecho TERMINA num `;` ou numa
    // LINHA EM BRANCO, como no console da JetBrains. Linhas de comentario
    // (`--`, `//`) ficam de fora. Cursor num trecho vazio (logo depois do
    // `;`, ou numa linha em branco) vale a instrucao que acabou antes dele.
    //
    // Por que a linha em branco (2026-10-03, achado na tela real): um
    // `select ...` sem `;` seguido de linhas vazias e de um `DELETE FROM x;`
    // virava UMA instrucao "select ... DELETE ..." — foi para o caminho de
    // leitura, o motor recusou, e o painel do impacto nunca abriu.
    function statementAt(text, cursor, selectionStart, selectionEnd, mongo) {
        if (selectionEnd > selectionStart) return text.substring(selectionStart, selectionEnd).trim();
        const clean = piece => piece.split("\n").filter(line => !/^\s*(--|\/\/)/.test(line)).join("\n").trim();
        if (mongo) {
            const lineStart = text.lastIndexOf("\n", cursor - 1) + 1;
            const lineEnd = text.indexOf("\n", cursor);
            return clean(text.substring(lineStart, lineEnd < 0 ? text.length : lineEnd));
        }
        const pieces = [];
        const boundary = /;|\n[ \t]*(?=\n)/g;
        let start = 0;
        for (let match = boundary.exec(text); match !== null; match = boundary.exec(text)) {
            pieces.push({ start: start, text: clean(text.substring(start, match.index)) });
            start = match.index + match[0].length;
        }
        pieces.push({ start: start, text: clean(text.substring(start)) });
        let at = 0;
        while (at + 1 < pieces.length && pieces[at + 1].start <= cursor) at++;
        while (at > 0 && pieces[at].text === "") at--;
        return pieces[at].text;
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
    function tableData(connection, engine, schema, table, readSql) {
        if (DataSourceKinds.isOdbc(engine)) {
            if (!readSql) {
                root.dataSourceController.queryStatus = qsTr("O driver não forneceu uma leitura para esta tabela. Use o console.");
            } else root.dataSourceController.runOn(connection, readSql, false, 200);
            root.resultsRequested();
            return;
        }
        const quote = name => "\"" + name.replace(/"/g, "\"\"") + "\"";
        const sql = DataSourceKinds.isMongo(engine) ? table + " {}"
                  : "SELECT * FROM " + (DataSourceKinds.isSqlite(engine) || schema === "" ? "" : quote(schema) + ".")
                    + quote(table) + " LIMIT 200";
        root.dataSourceController.runOn(connection, sql, false);
        root.resultsRequested();
    }
}
