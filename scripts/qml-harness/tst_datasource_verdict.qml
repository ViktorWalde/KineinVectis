pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O veredito do Testar no painel REAL (16.1, F1; roadmaps/59 §7.1.1): o
// sucesso aparece com a versao do servidor; editar o formulario apaga o
// sucesso; a falha e o pedido de senha continuam na tela.
Item {
    id: root
    width: 1000
    height: 700
    property int failures: 0
    property var verdict: null
    property var profile: Object.assign(DataSourceKinds.emptyProfile(), { name: "pg", port: 5432 })

    DataSourceController {
        id: bankController
        workspaceRoot: "/projeto"
    }
    DataSourcePanelHost {
        id: panelHost
        anchors.fill: parent
        controller: bankController
        visible: bankController.panelVisible
        maxAvailableWidth: root.width
        maxAvailableHeight: root.height
    }
    function check(condition, label) {
        if (!condition) { root.failures++; console.error("FALHOU: " + label); }
    }
    function findNamed(item, name) {
        if (item.objectName === name) return item;
        for (let index = 0; index < item.children.length; index++) {
            const found = root.findNamed(item.children[index], name);
            if (found) return found;
        }
        return null;
    }
    // Pede o teste e responde como o core responderia.
    function answer(ok, version, message, needsSecret) {
        bankController.test();
        const operation = bankController.catalog.pending["test:pg"];
        root.check(!!operation && root.verdict.visible && root.verdict.testing, "Testar mostra o Conectando");
        if (!operation) return;
        bankController.handleTested("pg", ok, version, message, needsSecret, operation.clientContext);
    }
    Component.onCompleted: {
        bankController.handleList([root.profile]);
        bankController.select("pg");
        bankController.open();
        root.verdict = root.findNamed(panelHost, "dataSourceVerdict");
        root.check(!!root.verdict, "o veredito existe no painel");
        if (!root.verdict) { Qt.exit(1); return; }
        root.check(!root.verdict.visible, "sem teste, sem veredito");

        root.answer(true, "PostgreSQL 16.4", "", false);
        root.check(root.verdict.visible && root.verdict.ok && root.verdict.serverVersion === "PostgreSQL 16.4",
            "o sucesso aparece com a versão do servidor");

        bankController.editDraft("port", 5433);
        root.check(!root.verdict.visible && !bankController.testOk && bankController.serverVersion === "",
            "editar o formulário apaga o sucesso");
        bankController.editDraft("port", 5432);
        root.check(!root.verdict.visible, "voltar o campo não ressuscita o sucesso antigo");

        root.answer(false, "", "conexão recusada", false);
        root.check(root.verdict.visible && !root.verdict.ok && root.verdict.message === "conexão recusada",
            "a falha aparece com a mensagem");
        bankController.editDraft("host", "outro");
        root.check(root.verdict.visible && root.verdict.message === "conexão recusada", "editar não apaga a falha");

        bankController.select("pg");
        root.answer(false, "", "senha exigida", true);
        bankController.editDraft("database", "outro");
        root.check(root.verdict.visible && root.verdict.secretRequired, "editar não esconde o campo de senha");
        Qt.exit(root.failures === 0 ? 0 : 1);
    }
}
