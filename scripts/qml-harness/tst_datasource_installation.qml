pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// 9a.3 (arquitetura/39 §6.2.2): o campo "Adaptador" no formulario e no
// controller reais. As opcoes vem do descritor do core (`installations`,
// 0.167.0); o rascunho leva `{ kind: "ide" }` ou nada.
Item {
    id: root
    width: 820
    height: 600
    property int failures: 0
    property var saved: null
    property var descriptors: [
        { id: "builtin.postgres", engine: "postgres", connectionKind: "server",
          profileFeatures: ["credentials", "verifiedTls"], installations: ["builtin"] },
        { id: "builtin.sqlite", engine: "sqlite", connectionKind: "file",
          profileFeatures: [], installations: ["builtin", "ide"] }
    ]
    DataSourceController {
        id: controller
        workspaceRoot: "/project"
        onSaveRequested: profile => root.saved = profile
    }
    // A mesma fiacao do DataSourcePanelHost: o gesto do campo passa pelo
    // `fieldEdited` do painel ate' o `editDraft` do controller.
    DataSourcePanel {
        id: panel
        anchors.fill: parent
        draft: controller.draft
        profiles: controller.profiles
        providers: controller.providers
        onFieldEdited: (field, value) => controller.editDraft(field, value)
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
    function installed(profile) {
        return profile.installation !== undefined && profile.installation !== null
            ? profile.installation.kind : "builtin";
    }
    Component.onCompleted: Qt.callLater(() => {
        const field = root.find(panel, "objectName", "dataSourceAdapterField");
        root.check(field !== null, "campo Adaptador carregado");
        if (!field) { Qt.exit(1); return; }
        controller.handleList([], root.descriptors);

        // So' aparece quando o motor oferece mais de um adaptador.
        root.check(!field.visible, "PostgreSQL, so' o interno: sem campo");
        controller.editDraft("engine", "sqlite");
        controller.editDraft("name", "estacao");
        controller.editDraft("database", "estacao.db");
        root.check(field.visible && field.current === "builtin", "SQLite mostra o campo no interno");
        root.check(field.choices.length === 2 && field.choices[1].value === "ide", "opcoes na ordem do descritor");

        // O gesto vai ao rascunho e ao save.
        field.selected("ide");
        root.check(root.installed(controller.draft) === "ide" && field.current === "ide", "O da IDE no rascunho");
        controller.save();
        root.check(root.saved !== null && root.installed(root.saved) === "ide"
                   && root.saved.installation.kind === "ide" && Object.keys(root.saved.installation).length === 1,
                   "save leva { kind: \"ide\" } e nada mais");

        // Voltar ao interno tira o campo do perfil (ausente = interno).
        field.selected("builtin");
        controller.save();
        root.check(root.saved.installation === undefined, "interno nao manda installation");

        // Escolha que o motor nao oferece e' ignorada.
        controller.editDraft("installation", "path");
        root.check(root.installed(controller.draft) === "builtin", "escolha desconhecida ignorada");

        // Trocar de motor derruba a escolha que o novo nao oferece (senao o
        // save daria INVALID_PARAMS); voltar ao SQLite comeca no interno.
        field.selected("ide");
        controller.editDraft("engine", "postgres");
        root.check(root.installed(controller.draft) === "builtin" && !field.visible, "PostgreSQL derruba o da IDE");
        controller.editDraft("engine", "sqlite");
        root.check(root.installed(controller.draft) === "builtin", "de volta ao SQLite, interno");

        // Perfil salvo com o da IDE abre com a escolha marcada.
        // A forma que o core lista: host, porta e usuario vem sempre.
        const profiles = [{ name: "estacao", engine: "sqlite", host: "", port: 0, database: "estacao.db",
                            user: "", installation: { kind: "ide" } }];
        controller.handleList(profiles, root.descriptors);
        controller.select("estacao");
        root.check(field.visible && field.current === "ide", "perfil salvo abre no da IDE");

        // Descritor de um core anterior a 0.167.0, sem `installations`: so' o
        // interno, sem campo; tipo futuro desconhecido nao vira opcao. O
        // rascunho continua o SQLite selecionado.
        controller.handleList(profiles, [Object.assign({}, root.descriptors[1], { installations: undefined })]);
        root.check(controller.draft.engine === "sqlite" && !field.visible, "descritor sem installations esconde o campo");
        controller.handleList(profiles, [Object.assign({}, root.descriptors[1], { installations: ["builtin", "path"] })]);
        root.check(controller.draft.engine === "sqlite" && !field.visible, "tipo desconhecido nao vira opcao");

        Qt.exit(root.failures === 0 ? 0 : 1);
    })
}
