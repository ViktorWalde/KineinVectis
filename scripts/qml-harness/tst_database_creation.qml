pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Menu e dialogo reais: abrir/cancelar nao cria dados nem perde contexto.
Item {
    id: root
    width: 1000
    height: 700
    property int stage: 0
    property int failures: 0
    property int writes: 0
    property var popup: null
    property var form: null
    property var creation: null
    property var profile: Object.assign(DataSourceKinds.emptyProfile(), { name: "pg" })

    DataSourceController {
        id: bankController
        workspaceRoot: "/projeto"
        onQueryRequested: root.writes++
    }
    Connections {
        target: bankController.discovery
        function onCreateSqliteRequested(name, path) { root.writes++; }
        function onCreateServerRequested(engine, name, port) { root.writes++; }
    }
    DatabaseWindow {
        id: bank
        width: 260
        height: 580
        controller: bankController
        menuLayer: root
        onCreationRequested: bankController.openCreation()
    }
    DataSourcePanelHost {
        id: panelHost
        anchors.fill: parent
        controller: bankController
        visible: bankController.panelVisible
        maxAvailableWidth: root.width
        maxAvailableHeight: root.height
        onDismissRequested: bankController.close()
    }
    function check(condition, label) {
        if (!condition) { root.failures++; console.error("FALHOU: " + label); }
    }
    function findItem(item, propertyName) {
        if (item[propertyName] !== undefined) return item;
        for (let index = 0; index < item.children.length; index++) {
            const found = root.findItem(item.children[index], propertyName);
            if (found) return found;
        }
        return null;
    }
    Timer {
        interval: 50
        running: true
        repeat: true
        onTriggered: {
            if (root.stage === 0) {
                bankController.handleList([root.profile]);
                bankController.select("pg");
                bankController.runOn("pg", "SELECT 1", false);
                bankController.handleQueried({ name: "pg", clientContext: bankController.lastQuery.clientContext,
                    success: true, columns: ["id"], rows: [["preservado"]], elapsedMs: 1 });
                root.writes = 0;
                bankController.sessionPassword = "senha-de-prova";
                bankController.errorText = "erro anterior";
                root.form = root.findItem(panelHost, "face");
                root.creation = root.findItem(panelHost, "serverProfileNamed");
                for (let index = 0; index < root.children.length; index++) {
                    if (typeof root.children[index].restorePreviousFocus === "function") root.popup = root.children[index];
                }
                root.check(!!root.form && !!root.creation && !!root.popup, "composição de menu, diálogo e criação");
                if (!root.form || !root.creation || !root.popup) { Qt.exit(1); return; }
                bank.focusTree();
                bank.showNewMenu(bank, 0, 0);
            } else if (root.stage === 1) {
                const index = root.popup.items.findIndex(item => item.action === "database.create");
                root.check(index >= 0, "Novo banco acessível no menu");
                if (index < 0) { Qt.exit(1); return; }
                root.popup.activate(index);
                root.check(!root.popup.visible && panelHost.visible && root.form.face === "create", "menu abre Criar banco");
                root.check(!root.popup.activeFocus && panelHost.activeFocus, "foco permanece no diálogo após fechar menu");
                root.check(bankController.selectedName === "pg" && bankController.draft.database === root.profile.database,
                    "perfil e rascunho preservados");
                root.check(bankController.sessionPassword === "" && bankController.errorText === "" && root.writes === 0,
                    "segredo/erro descartados sem escrita");
                root.check(bankController.queryRows[0][0] === "preservado", "abrir criação conserva resultado do console");
                root.check(root.creation.serverProfileNamed && root.creation.kind === "sqlite", "SQLite inicial e PostgreSQL salvo disponível");
                panelHost.dismissFromKeyboard();
                root.check(!panelHost.visible && root.form.face === "connection", "cancelar devolve face de conexão");
                bankController.open();
                root.check(root.form.face === "connection", "reabrir formulário não fica preso em criação");
                bankController.close();
                bankController.editDraft("database", "alterado");
                bankController.openCreation();
                root.check(!root.creation.serverProfileNamed, "rascunho não salvo não cria no servidor");
                bankController.close();
                bankController.handleList([Object.assign({}, root.profile, { readOnly: true })]);
                bankController.select("pg");
                bankController.openCreation();
                root.check(!root.creation.serverProfileNamed, "somente leitura não cria no servidor");
                bankController.workspaceRoot = "";
                root.check(root.form.face === "connection", "workspace descarta face antiga");
                bankController.close();
                bankController.openCreation();
                root.check(!panelHost.visible && root.writes === 0, "sem projeto não abre criação nem executa");
                Qt.exit(root.failures === 0 ? 0 : 1);
            }
            root.stage++;
        }
    }
}
