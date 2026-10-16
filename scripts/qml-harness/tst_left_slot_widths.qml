import QtQuick
import KineinVectis

// 16.5b (roadmaps/59 §7.1.5): a largura do slot da esquerda e' de cada
// janela — alargar o Banco nao alarga o Projeto — e o layout gravado a
// devolve; o ShellController e o codec REAIS.
Item {
    id: root

    property int failures: 0

    function check(condition, label) {
        if (!condition) { root.failures++; console.error("FALHOU: " + label); }
    }

    ShellController { id: shell }
    ShellController { id: reopened }

    Component.onCompleted: {
        shell.explorerPreferredWidth = 280;
        shell.leftWindow = "database";
        shell.explorerPreferredWidth = 420;   // o Banco alargou
        shell.leftWindow = "explorer";
        root.check(shell.explorerPreferredWidth === 280, "o Projeto volta com a dele: " + shell.explorerPreferredWidth);
        shell.leftWindow = "database";
        root.check(shell.explorerPreferredWidth === 420, "o Banco volta com a dele: " + shell.explorerPreferredWidth);
        shell.leftWindow = "git";
        root.check(shell.explorerPreferredWidth === 420, "sem largura propria, herda a atual");

        const layout = shell.layoutSnapshot ? shell.layoutSnapshot() : shell.codec.snapshot(shell);
        root.check(layout.sizes.left.explorer === 280 && layout.sizes.left.database === 420,
            "o layout grava as larguras: " + JSON.stringify(layout.sizes.left));
        reopened.applyingLayout = true;
        reopened.applyLayout(JSON.parse(JSON.stringify(layout)));
        reopened.applyingLayout = false;
        reopened.leftWindow = "explorer";
        root.check(reopened.explorerPreferredWidth === 280, "reaberto, o Projeto com a dele: " + reopened.explorerPreferredWidth);
        reopened.leftWindow = "database";
        root.check(reopened.explorerPreferredWidth === 420, "reaberto, o Banco com a dele: " + reopened.explorerPreferredWidth);
        Qt.exit(root.failures === 0 ? 0 : 1);
    }
}
