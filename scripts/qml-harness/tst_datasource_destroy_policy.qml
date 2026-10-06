import QtQuick
import KineinVectis

Item {
    id: root
    DataSourceController { id: controller }
    DataSourceDestroyBox {
        id: box
        profileName: "loja"
        database: "dados"
        production: true
    }
    function check(condition, label) {
        if (!condition) console.error("FALHOU: " + label);
        return condition ? 0 : 1;
    }
    Component.onCompleted: {
        let failures = 0;
        failures += check(box.canRemove, "produção permite remover somente configuração");
        box.withData = true;
        failures += check(!box.canRemove, "dados de produção exigem os dois nomes");
        box.typedConnection = "loj";
        box.typedDatabase = "dados";
        failures += check(!box.canRemove, "nome parcial não libera remoção");
        box.typedConnection = "loja";
        failures += check(box.canRemove, "nomes completos liberam remoção");
        box.readOnly = true;
        failures += check(!box.withData && !box.removesData && box.canRemove, "somente leitura permite remover perfil, preservando dados");
        box.withData = true;
        failures += check(!box.removesData, "estado anterior não vence somente leitura");

        const profile = Object.assign(DataSourceKinds.emptyProfile(), { name: "loja", database: "dados", production: true });
        controller.workspaceRoot = "/projeto";
        controller.handleList([profile]);
        controller.select("loja");
        const discovery = controller.discovery;
        discovery.destroyProfile("loja", true, { connection: "loja", target: "dados" });
        const old = Object.assign({}, discovery.pendingDestroy);
        controller.workspaceRoot = "/outro";
        controller.workspaceRoot = "/projeto";
        controller.handleList([profile]);
        controller.select("loja");
        discovery.destroyProfile("loja", true, { connection: "loja", target: "dados" });
        discovery.handleDestroyResolved([], true, "", "", "", old.clientContext);
        discovery.handleDestroyed(true, "antigo", [], old.clientContext);
        failures += check(discovery.destroying && controller.profiles.length === 1, "remoção antiga não limpa catálogo do projeto reaberto");
        const current = discovery.pendingDestroy;
        failures += check(current.expectedContext.profile.production === true && current.confirmation.target === "dados",
            "remoção leva destino e confirmação públicos");
        discovery.handleDestroyResolved([], true, "", "", "", current.clientContext);
        failures += check(!discovery.destroying && controller.profiles.length === 0, "remoção atual atualiza catálogo");
        controller.handleList([profile]);
        controller.select("loja");
        discovery.destroyProfile("loja", true, { connection: "loja", target: "dados" });
        const replaced = Object.assign({}, discovery.pendingDestroy);
        controller.handleList([Object.assign({}, profile, { host: "outro-servidor" })]);
        failures += check(!discovery.destroying && discovery.pendingDestroy === null,
            "perfil substituído encerra indicador da remoção anterior");
        discovery.handleDestroyed(true, "antigo", [], replaced.clientContext);
        failures += check(controller.profiles.length === 1 && controller.profiles[0].host === "outro-servidor",
            "fim da remoção antiga não apaga perfil substituído na tela");
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
