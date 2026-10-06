import QtQuick
import KineinVectis

Item {
    id: root
    DataSourceController { id: controller }
    function check(condition, label) {
        if (!condition) console.error("FALHOU: " + label);
        return condition ? 0 : 1;
    }
    Component.onCompleted: {
        let failures = 0;
        const profile = Object.assign(DataSourceKinds.emptyProfile(), { name: "loja", host: "db-a" });
        controller.workspaceRoot = "/projeto";
        controller.handleList([profile]);
        controller.select(profile.name);
        controller.test();
        const oldTest = Object.assign({}, controller.catalog.pending["test:loja"]);
        controller.test();
        const newTest = Object.assign({}, controller.catalog.pending["test:loja"]);
        controller.handleTested("loja", false, "", "senha antiga", true, oldTest.clientContext);
        failures += check(controller.testing && !controller.secretRequired && controller.secrets.pending === null,
            "teste antigo não pede senha nem encerra teste novo");
        controller.handleTested("loja", true, "versão atual", "", false, newTest.clientContext);
        failures += check(!controller.testing && controller.serverVersion === "versão atual", "teste atual aparece");
        controller.introspectProfile("loja");
        const oldCatalog = Object.assign({}, controller.catalog.pending["introspect:loja"]);
        controller.introspectProfile("loja");
        const newCatalog = Object.assign({}, controller.catalog.pending["introspect:loja"]);
        controller.handleIntrospected("loja", true, [{ name: "antigo", tables: [] }], [], "", false, oldCatalog.clientContext);
        failures += check(controller.readingNames["loja"] && controller.structures["loja"] === undefined, "catálogo antigo não preenche árvore");
        controller.handleIntrospected("loja", true, [{ name: "atual", tables: [] }], [], "", false, newCatalog.clientContext);
        failures += check(!controller.readingNames["loja"] && controller.structures["loja"].schemas[0].name === "atual", "catálogo atual aparece");
        controller.handleList([Object.assign({}, profile, { host: "db-b" })]);
        failures += check(controller.structures["loja"] === undefined, "mesmo nome com outro destino limpa catálogo em cache");
        controller.select("loja");
        controller.test();
        const previous = Object.assign({}, controller.catalog.pending["test:loja"]);
        controller.workspaceRoot = "/outro";
        controller.workspaceRoot = "/projeto";
        controller.handleList([profile]);
        controller.select("loja");
        controller.test();
        controller.handleTested("loja", false, "", "senha de outra sessão", true, previous.clientContext);
        failures += check(controller.testing && !controller.secretRequired, "reabrir projeto não aceita teste de sessão anterior");
        controller.editDraft("database", "outro-banco");
        failures += check(!controller.testing && Object.keys(controller.catalog.pending).length === 0, "editar destino cancela pedidos");
        controller.test();
        failures += check(!controller.testing && controller.errorText.indexOf("Salve") >= 0, "teste de rascunho alterado pede salvar");
        const other = Object.assign({}, profile, { name: "outra" });
        controller.handleList([profile, other]);
        controller.select("loja");
        controller.test();
        const beforeSelection = Object.assign({}, controller.catalog.pending["test:loja"]);
        controller.select("outra");
        controller.handleTested("loja", false, "", "senha atrasada", true, beforeSelection.clientContext);
        failures += check(controller.selectedName === "outra" && !controller.secretRequired && controller.testMessage === "",
            "teste da seleção anterior não reabre senha nem muda a conexão atual");
        controller.introspectProfile("loja");
        const background = Object.assign({}, controller.catalog.pending["introspect:loja"]);
        controller.select("loja");
        controller.handleIntrospected("loja", false, [], [], "senha atrasada", true, background.clientContext);
        failures += check(!controller.secretRequired && controller.secrets.pending === null,
            "catálogo mantém resultado da árvore, mas seleção nova não recebe pedido de senha anterior");
        controller.introspectProfile("loja");
        controller.introspectProfile("outra");
        const anotherCatalog = Object.assign({}, controller.catalog.pending["introspect:outra"]);
        controller.handleFailed("datasource.introspect", "rede indisponível", "INTERNAL_ERROR", anotherCatalog);
        failures += check(controller.reading && controller.readingNames["loja"],
            "falha em outro catálogo não encerra o indicador do catálogo atual");
        controller.test();
        controller.editDraft("name", "novo-nome");
        failures += check(!controller.testing && controller.catalog.pending["test:loja"] === undefined,
            "renomear rascunho invalida teste do perfil selecionado");
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
