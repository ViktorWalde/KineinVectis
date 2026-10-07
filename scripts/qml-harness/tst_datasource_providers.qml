pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Formulario/controller reais: os descritores decidem campos e disponibilidade.
Item {
    id: root
    width: 820
    height: 600
    property int failures: 0
    property var descriptors: [
        { id: "builtin.postgres", engine: "postgres", connectionKind: "server", profileFeatures: ["credentials", "verifiedTls"] },
        { id: "builtin.sqlite", engine: "sqlite", connectionKind: "file", profileFeatures: [] },
        { id: "builtin.mongo", engine: "mongo", connectionKind: "server", profileFeatures: ["credentials", "sampling"] },
        { id: "system.odbc", engine: "odbc", connectionKind: "dsn", profileFeatures: ["credentials"] }
    ]
    DataSourceController { id: controller; workspaceRoot: "/project" }
    DatabaseTreeActions { id: actions; controller: controller }
    DataSourcePanel {
        id: panel
        anchors.fill: parent
        draft: controller.draft
        profiles: controller.profiles
        providers: controller.providers
    }
    function find(item, key, value) {
        if (item[key] === value) return item;
        for (let index = 0; index < item.children.length; index++) {
            const found = root.find(item.children[index], key, value);
            if (found) return found;
        }
        return null;
    }
    function check(ok, label) {
        if (!ok) { root.failures++; console.error("FALHOU: " + label); }
    }
    Component.onCompleted: Qt.callLater(() => {
        const form = root.find(panel, "credentials", false);
        const save = root.find(panel, "text", "Salvar");
        root.check(form !== null && save !== null, "formulario e botao reais carregados");
        if (!form || !save) { Qt.exit(1); return; }
        controller.editDraft("name", "new");
        root.check(!form.enabled && !save.enabled, "sem metadata nao habilita formulario nem save");
        controller.handleList([], root.descriptors);
        root.check(form.enabled && form.server && form.credentials && form.verifiedTls
                   && !form.mongo && save.enabled, "PostgreSQL com campos declarados");
        controller.editDraft("engine", "sqlite");
        root.check(form.fileConnection && !form.server && !form.credentials && !form.verifiedTls, "SQLite sem rede nem segredo");
        controller.editDraft("engine", "mongo");
        root.check(form.server && form.credentials && form.mongo && !form.verifiedTls, "Mongo com amostra sem TLS PostgreSQL");
        controller.editDraft("engine", "odbc");
        root.check(form.odbc && !form.server && form.credentials && !form.mongo, "ODBC usa DSN");
        controller.editDraft("engine", "postgres");
        controller.handleList([], [Object.assign({}, root.descriptors[0], { profileFeatures: [] })]);
        root.check(!form.credentials && !form.verifiedTls, "campos obedecem descritor, nao comparacao com postgres");
        root.check(DataSourceKinds.providerOptions(controller.providers).length === 1, "seletor exibe apenas motores fornecidos");
        actions.showNew();
        root.check(actions.entries().filter(entry => entry.action.indexOf("new.") === 0).length === 1,
                   "menu usa os mesmos descritores do formulario");
        controller.handleList([]);
        root.check(controller.providers.length === 1, "atualizacao sem metadata conserva descritores");
        controller.editDraft("engine", "unknown");
        root.check(form.provider === null && !save.enabled && !form.server, "motor desconhecido indisponivel");
        root.check(Object.keys(DataSourceKinds.defaultsFor("unknown")).length === 0
                   && DataSourceKinds.engineName("unknown") !== "PostgreSQL", "sem fallback PostgreSQL");
        controller.workspaceRoot = "/other";
        root.check(controller.providers.length === 0 && !save.enabled, "troca de projeto limpa metadata");
        Qt.exit(root.failures === 0 ? 0 : 1);
    })
}
