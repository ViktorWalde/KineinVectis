import QtQuick
import KineinVectis

Item {
    id: root
    DataSourceController { id: controller }
    readonly property var secrets: controller.secrets
    property var sent: ({})
    QtObject {
        id: bridge
        function dataSourceList() {}
        function dataSourceTest(name, password) { root.sent = { name: name, password: password }; }
        function dataSourceIntrospect(name, password) { root.sent = { name: name, password: password }; }
        function dataSourceQuery(name, password, sql, maxRows, confirmed) { root.sent = { name: name, password: password }; }
        function dataSourceImpact(name, password, sql) { root.sent = { name: name, password: password }; }
    }
    DataSourceRequestRouter {
        coreClient: bridge
        dataSourceController: controller
    }
    function check(condition, label) {
        if (!condition) console.error("FALHOU: " + label);
        return condition ? 0 : 1;
    }
    Component.onCompleted: {
        let failures = 0;
        const first = Object.assign(DataSourceKinds.emptyProfile(), { name: "primeiro", host: "db-a", user: "a" });
        const second = Object.assign(DataSourceKinds.emptyProfile(), { name: "segundo", host: "db-b", user: "b" });
        controller.workspaceRoot = "/primeiro-projeto";
        controller.handleList([first, second]);
        controller.select(first.name);
        secrets.value = "sentinela-local";
        failures += check(secrets.forName(first.name) === "sentinela-local", "senha disponivel no perfil de origem");
        failures += check(secrets.forName(second.name) === "", "senha nunca cruza conexoes");
        controller.runOn(first.name, "SELECT 1", false);
        failures += check(root.sent.password === "sentinela-local", "consulta de origem recebe senha");
        controller.runOn(second.name, "SELECT 1", false);
        failures += check(root.sent.password === "", "consulta de outra conexao nao recebe senha");
        controller.testProfile(second.name);
        failures += check(root.sent.password === "", "teste de outra conexao nao recebe senha");
        controller.introspectProfile(second.name);
        failures += check(root.sent.password === "", "catalogo de outra conexao nao recebe senha");
        controller.runOn(second.name, "DELETE FROM t", false);
        controller.impact.begin(second.name, "DELETE FROM t");
        failures += check(root.sent.password === "", "impacto de outra conexao nao recebe senha");
        controller.runOn(first.name, "DELETE FROM t", false);
        controller.impact.begin(first.name, "DELETE FROM t");
        failures += check(root.sent.password === "sentinela-local", "impacto de origem recebe senha");
        controller.editDraft("host", "destino-diferente");
        failures += check(secrets.value === "" && secrets.forName(first.name) === "", "editar destino limpa segredo");
        controller.select(first.name);
        secrets.value = "sentinela-local";
        controller.handleList([Object.assign({}, first, { host: "outro-host" }), second]);
        failures += check(secrets.value === "", "mesmo nome com outro destino revoga segredo");
        controller.handleList([first, second]);
        controller.select(first.name);
        secrets.value = "sentinela-local";
        controller.workspaceRoot = "/segundo-projeto";
        failures += check(secrets.value === "", "trocar projeto limpa segredo");
        controller.handleList([first, second]);
        controller.select(first.name);
        secrets.value = "sentinela-local";
        controller.handleList([second]);
        failures += check(secrets.value === "", "remover perfil limpa segredo");
        controller.handleList([first, second]);
        controller.select(first.name);
        controller.sessionPassword = "sentinela-local";
        controller.close();
        failures += check(controller.sessionPassword === "", "fechar limpa alias publico");
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
