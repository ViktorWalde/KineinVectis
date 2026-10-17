import QtQuick
import KineinVectis

// C1 (0.4.0c, roadmaps/52 §14.3): a caixa "Checar com o compilador ao salvar
// (C/C++)". O SettingsController e a SettingsEditorPage REAIS: o efetivo
// ausente vale ligado (o padrao do core); desligado chega desligado; a caixa
// pede a troca da chave certa; o selo de projeto aparece quando o projeto a
// define.
Item {
    id: root
    width: 700
    height: 600

    property var changes: []

    SettingsController {
        id: controller
    }

    QtObject {
        id: dialog

        property bool formatOnSave: false
        property bool autoSave: true
        property bool compilerCheckOnSave: controller.compilerCheckOnSave
        property bool autoClosePairs: true
        property int editorFontSize: 14
        property var shownProjectKeys: []
        signal settingChanged(string key, var value)
        onSettingChanged: (key, value) => root.changes.push(key + "=" + value)
    }

    SettingsEditorPage {
        id: page

        width: parent.width
        settings: dialog
    }

    function check(condition, label) {
        if (!condition) console.error("FALHOU: " + label);
        return condition ? 0 : 1;
    }

    function findRow(item, label) {
        if (item.label === label) return item;
        for (let i = 0; i < item.children.length; i++) {
            const found = findRow(item.children[i], label);
            if (found !== null) return found;
        }
        return null;
    }

    Component.onCompleted: {
        let failures = 0;
        controller.handleResolved({}, {}, {});
        failures += check(controller.compilerCheckOnSave === true, "ausente: ligado");
        controller.handleResolved({ compilerCheckOnSave: false }, {}, {});
        failures += check(controller.compilerCheckOnSave === false, "desligado chega desligado");

        const row = root.findRow(page, "Checar com o compilador ao salvar (C/C++)");
        failures += check(row !== null, "a caixa existe na pagina do editor");
        if (row !== null) {
            failures += check(row.checked === false, "a caixa mostra o efetivo");
            row.toggled();
            failures += check(root.changes.length === 1 && root.changes[0] === "compilerCheckOnSave=true",
                              "a troca pede a chave certa: " + JSON.stringify(root.changes));
            failures += check(row.fromProject === false, "sem a chave no projeto, sem selo");
            dialog.shownProjectKeys = ["compilerCheckOnSave"];
            failures += check(row.fromProject === true, "o projeto define: o selo");
        }
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
