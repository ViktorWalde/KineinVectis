import QtQuick
import KineinVectis

// A escolha do dialogo vai para onde o valor mora (2026-10-03). O defeito:
// um projeto com `rigorProfile` no proprio settings ignorava a troca feita
// no dialogo, que sempre gravava no global — por baixo do valor do projeto.
Item {
    id: root

    property var writes: []

    SettingsController {
        id: settings

        onSetRequested: function(scope, values) {
            root.writes.push(scope + ":" + JSON.stringify(values));
        }
    }

    function check(condition, label) {
        if (!condition) console.error("FALHOU: " + label);
        return condition ? 0 : 1;
    }

    Component.onCompleted: {
        let failures = 0;
        settings.workspaceValues = ({});
        settings.setWhereItLives("rigorProfile", "balanced");
        failures += check(root.writes[0] === 'global:{"rigorProfile":"balanced"}',
                          "sem valor no projeto, vai para o global: " + root.writes[0]);

        settings.workspaceValues = ({ "rigorProfile": "strict" });
        settings.setWhereItLives("rigorProfile", "relaxed");
        failures += check(root.writes[1] === 'workspace:{"rigorProfile":"relaxed"}',
                          "o projeto define, vai para o projeto: " + root.writes[1]);

        // Outra chave do mesmo projeto, nao definida nele: global.
        settings.setWhereItLives("autoSave", false);
        failures += check(root.writes[2] === 'global:{"autoSave":false}', "chave a chave: " + root.writes[2]);

        if (failures !== 0) console.error("FALHAS " + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
