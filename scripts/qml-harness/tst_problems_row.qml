import QtQuick
import KineinVectis

// A linha do painel de Problemas (F5 na tela real, 2026-10-03). O defeito:
// o caminho ABSOLUTO do compilador ocupava a linha inteira, a largura da
// mensagem ficava negativa e ela sumia; a mensagem do clangd com notas
// (varias linhas) bagunçava a linha. Aqui: caminho relativo e limitado, a
// mensagem sempre com espaco, so' a primeira linha dela.
Item {
    id: root

    width: 980
    height: 200

    ListModel {
        id: problems

        ListElement {
            severity: "error"; code: "ovl_no_viable_function_in_call"; line: 7; column: 27; source: "clangd"
            file: "/tmp/claude-1000/um/caminho/bem/comprido/de/verdade/projetos/f5-app/src/main.cpp"
            message: "No matching function for call to 'soma'\n/tmp/x/calc.h:3: note: candidate function not viable"
        }
    }

    ProblemsPanel {
        id: panel

        anchors.fill: parent
        workspaceRoot: "/tmp/claude-1000/um/caminho/bem/comprido/de/verdade/projetos/f5-app"
        diagnosticsModel: problems
    }

    function findText(item, predicate) {
        for (let i = 0; i < item.children.length; i++) {
            const child = item.children[i];
            if (child.text !== undefined && child.font !== undefined && predicate(child)) return child;
            const found = findText(child, predicate);
            if (found !== null) return found;
        }
        return null;
    }

    function check(condition, label) {
        if (!condition) console.error("FALHOU: " + label);
        return condition ? 0 : 1;
    }

    Timer {
        interval: 50
        running: true
        onTriggered: {
            let failures = 0;
            const path = root.findText(panel.contentItem, t => t.text.indexOf("main.cpp:7") >= 0);
            failures += root.check(path !== null && path.text === "src/main.cpp:7",
                                   "caminho relativo ao projeto: " + (path ? path.text : "nenhum"));
            const message = root.findText(panel.contentItem, t => t.text.indexOf("No matching") === 0);
            failures += root.check(message !== null, "a mensagem aparece");
            if (message !== null) {
                failures += root.check(message.width > 200, "a mensagem tem espaco: " + message.width);
                failures += root.check(message.text.indexOf("\n") < 0, "so' a primeira linha da mensagem");
            }
            if (failures !== 0) console.error("FALHAS " + failures);
            Qt.exit(failures === 0 ? 0 : 1);
        }
    }
}
