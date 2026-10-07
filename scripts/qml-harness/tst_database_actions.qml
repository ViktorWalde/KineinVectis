pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

Item {
    id: root
    width: 640
    height: 480
    property var requests: []
    property int failures: 0

    DataSourceController {
        id: bankController
        workspaceRoot: "/projeto"
    }
    DataSourceTree {
        id: tree
        workspaceRoot: bankController.workspaceRoot
        profiles: bankController.profiles
        structures: bankController.structures
        readingNames: bankController.readingNames
    }
    DatabaseTreeActions {
        id: treeActions
        controller: bankController
        treeModel: tree
        onConsoleRequested: name => root.requests.push(["console", name])
        onConsoleStatementRequested: (name, text) => root.requests.push(["console", name, text])
        onEditRequested: name => root.requests.push(["edit", name])
        onTableDataRequested: (connection, engine, schema, table, readSql) => root.requests.push(["data", connection, engine, schema, table, readSql])
        onNewRequested: engine => root.requests.push(["new", engine])
        onDiscoveryRequested: root.requests.push(["discover"])
        onCreationRequested: root.requests.push(["create"])
    }
    Connections {
        target: bankController
        function onIntrospectRequested(name, password, context) { root.requests.push(["refresh", name, context]); }
    }
    DatabaseTreeView {
        id: view
        width: 300
        height: 300
        controller: bankController
        treeModel: tree
        actions: treeActions
        onContextMenuRequested: (key, x, y) => treeActions.showRow(key)
        onTableDataRequested: (connection, engine, schema, table, readSql) => root.requests.push(["enter", connection, table])
    }

    function check(condition, label) {
        if (!condition) { root.failures++; console.error("FALHOU: " + label); }
    }
    function key(code, modifiers) { view.handleKey({ key: code, modifiers: modifiers || 0, accepted: false }); }
    function actionsList() { return treeActions.entries().filter(item => item.enabled).map(item => item.action).join(" "); }

    Component.onCompleted: {
        const profile = name => Object.assign(DataSourceKinds.emptyProfile(), { name: name, engine: "odbc", host: "", database: "fonte" });
        bankController.profiles = [profile("a|b"), profile("a"), profile("__proto__")];
        bankController.structures = DataSourceMap.copy(null, {
            ["a|b"]: { schemas: [{ name: "c", tables: [{ name: "d", kind: "table", columns: [], readSql: "SELECT driver" }] }], collections: [] },
            ["a"]: { schemas: [{ name: "b|c", tables: [{ name: "d", kind: "view", columns: [] }] }], collections: [] }
        });
        tree.toggle(tree.key(["c", "a|b"]));
        tree.toggle(tree.key(["c", "a"]));
        const first = tree.rows.find(row => row.kind === "table");
        const other = tree.rows.find(row => row.kind === "view");
        tree.select(first.key);
        treeActions.showRow(first.key);
        root.check(root.actionsList() === "database.data database.console database.refresh database.copy", "ações da tabela, sem escrita fabricada");
        treeActions.activateMenu("database.data");
        root.check(JSON.stringify(root.requests.pop()) === JSON.stringify(["data", "a|b", "odbc", "c", "d", "SELECT driver"]), "contexto e leitura ODBC intactos");
        root.check(!treeActions.menuOpen, "ação fecha menu");
        treeActions.showRow(first.key);
        tree.select(other.key);
        treeActions.activateMenu("database.console");
        root.check(JSON.stringify(root.requests.pop()) === JSON.stringify(["console", "a|b", "SELECT driver"]), "console usa leitura da chave do menu, não da seleção seguinte");

        tree.select(first.key);
        treeActions.dispatch("database.refresh", tree.selectedRow);
        const request = root.requests.pop();
        root.check(request[0] === "refresh" && request[1] === "a|b" && !!request[2].clientContext, "releitura pelo catálogo correlacionado");
        root.check(!treeActions.canRefresh(tree.selectedRow), "releitura ocupada bloqueada");
        root.check(tree.selectedKey === first.key && tree.selectedRow.kind === "connection", "leitura conserva objeto com conexão provisória");
        treeActions.dispatch("database.refresh", tree.selectedRow);
        root.check(root.requests.length === 0, "não duplica pedido ocupado");
        bankController.handleIntrospected("a|b", true, [{ name: "c", tables: [
            { name: "antes", kind: "table", columns: [] }, { name: "d", kind: "table", columns: [], readSql: "SELECT driver" }
        ] }], [], "", false, request[2].clientContext);
        bankController.structures = DataSourceMap.copy(bankController.structures);
        root.check(tree.selectedRow.key === first.key && tree.selectedIndex > 0, "releitura conserva chave apesar do índice");
        // O ListView pode recalcular currentIndex quando o modelo muda.
        // Navegação deve usar o índice lógico do objeto, mesmo nessa janela.
        view.currentIndex = 0;
        root.key(Qt.Key_Up);
        root.check(tree.selectedRow.name === "antes", "seta usa índice da seleção, não índice transitório do ListView");
        tree.select(first.key);
        treeActions.showRow(first.key);
        bankController.structures = DataSourceMap.copy(bankController.structures, {
            ["a|b"]: { schemas: [{ name: "c", tables: [] }], collections: [] }
        });
        root.check(!treeActions.menuOpen && tree.selectedRow.kind === "connection" && tree.selectedRow.connection === "a|b", "objeto removido fecha menu e seleciona conexão");
        treeActions.showRow(tree.selectedKey);
        root.check(root.actionsList() === "database.console database.refresh database.edit database.disconnect database.copy", "ações da conexão incluem desconexão real");
        bankController.profiles = bankController.profiles.map(item => Object.assign({}, item, item.name === "a|b" ? { database: "outro" } : {}));
        root.check(!treeActions.menuOpen, "perfil diferente fecha menu");
        treeActions.activateMenu("database.edit");
        root.check(root.requests.length === 0, "menu inválido não age");

        treeActions.showNew();
        root.check(root.actionsList() === "new.postgres new.sqlite new.mongo new.odbc database.discover database.create", "motores existentes e Novo banco no menu");
        treeActions.activateMenu("new.mongo");
        root.check(JSON.stringify(root.requests.pop()) === JSON.stringify(["new", "mongo"]), "motor escolhido");
        treeActions.showNew();
        treeActions.activateMenu("database.create");
        root.check(JSON.stringify(root.requests.pop()) === JSON.stringify(["create"]), "Novo banco usa intenção de criação");
        treeActions.activateMenu("database.create");
        root.check(root.requests.length === 0, "menu fechado não duplica criação");
        treeActions.showNew();
        bankController.workspaceRoot = "/outro";
        root.check(!treeActions.menuOpen && tree.selectedKey === "" && Object.keys(tree.expanded).length === 0, "workspace limpa seleção e menu");
        treeActions.activateMenu("database.create");
        root.check(root.requests.length === 0, "menu de outro workspace não abre criação");

        bankController.profiles = [profile("a"), profile("__proto__")];
        root.key(Qt.Key_End);
        root.check(tree.selectedRow.connection === "__proto__", "End com nome especial");
        root.key(Qt.Key_Home);
        root.key(Qt.Key_Right);
        root.check(tree.selectedRow.expanded, "direita expande conexão");
        root.key(Qt.Key_Right);
        root.check(tree.selectedRow.depth === 1, "direita entra no filho");
        root.key(Qt.Key_Left);
        root.check(tree.selectedRow.kind === "connection", "esquerda volta ao pai");
        root.check(view.isMenuKey({ key: Qt.Key_F10, modifiers: Qt.ShiftModifier })
            && !view.isMenuKey({ key: Qt.Key_F10, modifiers: Qt.ControlModifier | Qt.ShiftModifier }), "atalho de contexto exato");
        root.key(Qt.Key_F10, Qt.ShiftModifier);
        root.check(treeActions.menuOpen && treeActions.menuKey === tree.selectedKey, "Shift+F10 usa seleção");
        treeActions.menuOpen = false;
        treeActions.dispatch("database.collapse", null);
        root.check(tree.rows.length === 2 && tree.selectedRow.kind === "connection", "recolher tudo");
        root.key(Qt.Key_Down);
        root.key(Qt.Key_Up);
        root.check(tree.selectedRow.connection === "a", "setas preservam seleção");
        const event = { key: Qt.Key_Tab, modifiers: 0, accepted: true };
        view.handleKey(event);
        root.check(!event.accepted, "Tab segue para o ciclo de foco");
        if (root.failures) console.error("FALHAS " + root.failures);
        Qt.exit(root.failures === 0 ? 0 : 1);
    }
}
