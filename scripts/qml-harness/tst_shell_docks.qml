import QtQuick
import KineinVectis

// As janelas acopladas abrem DO LADO DO ICONE (2026-10-03, pedido do autor;
// ele avisou que "essa parte pode ser complicada de fazer sem bugs"). O que se
// prova:
//   - padrao: tudo na esquerda, um slot, o icone do aberto fecha;
//   - icone na direita: a janela abre no slot da direita, sem fechar a da
//     esquerda (Projeto a esquerda e Banco a direita ao mesmo tempo);
//   - a mesma janela nunca fica nos dois lados;
//   - arrastar o icone com a janela aberta leva a janela junto, e arrastar
//     com ela fechada nao abre nada;
//   - o slot da direita vai e volta no layout gravado; janela que nao existe
//     nao volta.
Item {
    id: root

    ShellController {
        id: shell
    }

    function check(condition, message) {
        if (!condition) {
            console.error("FALHOU: " + message);
            return 1;
        }
        return 0;
    }

    Component.onCompleted: {
        let failures = 0;
        shell.updateViewport(1600, 900);
        shell.outlineCollapsed = true;
        // Com projeto aberto: sem ele, a tela inicial nao reabre janela (abaixo).
        shell.workspaceRoot = "/w";

        failures += check(shell.effectiveShowExplorer && shell.rightWindow === "", "padrao: Projeto a esquerda");
        shell.toggleDockWindow("database");
        failures += check(shell.leftWindow === "database" && shell.databaseWindowVisible
                          && !shell.effectiveShowExplorer && shell.rightWindow === "", "Banco no slot da esquerda");
        shell.toggleDockWindow("database");
        failures += check(!shell.databaseWindowVisible && !shell.showExplorer, "o icone do aberto fecha");
        shell.toggleExplorer();

        // O icone do Banco vai para o trilho da direita, com a janela fechada.
        shell.moveRailEntryToSide("database", "right", 0, []);
        failures += check(shell.rightWindow === "" && !shell.databaseWindowVisible,
                          "arrastar com a janela fechada nao abre");
        shell.toggleDockWindow("database");
        failures += check(shell.rightWindow === "database" && shell.effectiveShowExplorer
                          && shell.leftWindow === "explorer", "Banco a direita, Projeto continua a esquerda");
        failures += check(shell.docks.showing("database") === "right", "o lado de agora");
        failures += check(shell.rightWidth >= 240 && shell.rightWidth <= 640, "largura da direita: " + shell.rightWidth);
        shell.toggleDockWindow("database");
        failures += check(shell.rightWindow === "" && shell.effectiveShowExplorer, "fechar a direita nao mexe na esquerda");

        // Aberta a direita, o icone volta para a esquerda: a janela vai junto.
        shell.showDockWindow("database");
        shell.moveRailEntryToSide("database", "left", 0, []);
        failures += check(shell.rightWindow === "" && shell.leftWindow === "database" && shell.showExplorer,
                          "a janela seguiu o icone para a esquerda");
        // E de volta para a direita, aberta: sai da esquerda (nunca nos dois).
        shell.moveRailEntryToSide("database", "right", 0, []);
        failures += check(shell.rightWindow === "database" && !shell.showExplorer,
                          "a janela seguiu o icone para a direita e a esquerda fechou");

        // O Projeto com o icone a direita: abre la'.
        shell.moveRailEntryToSide("explorer", "right", 0, []);
        shell.toggleExplorer();
        failures += check(shell.rightWindow === "explorer" && !shell.databaseWindowVisible,
                          "um slot por lado: o Projeto tomou a direita");

        // Os Simbolos abertos escondem o slot da direita; fechados, ele volta.
        shell.showDockWindow("database");
        shell.toggleOutline();
        failures += check(!shell.databaseWindowVisible && shell.rightWindow === "database",
                          "Simbolos abertos: o Banco some por um momento");
        shell.toggleOutline();
        failures += check(shell.databaseWindowVisible, "Simbolos fechados: o Banco volta");
        // Clicar no icone do Banco escondido traz o Banco e fecha os Simbolos.
        shell.toggleOutline();
        shell.toggleDockWindow("database");
        failures += check(shell.databaseWindowVisible && shell.outlineCollapsed, "o icone traz de volta");
        shell.toggleDockWindow("database");

        // Layout antigo: Banco aberto a esquerda com o icone a direita.
        shell.rightWindow = "";
        shell.leftWindow = "database";
        shell.showExplorer = true;
        shell.moveRailEntryToSide("explorer", "left", 0, []);
        shell.docks.normalize();
        failures += check(shell.rightWindow === "database" && !shell.showExplorer,
                          "layout antigo: a janela vai para o lado do icone");
        shell.toggleExplorer();

        // A janela pede mais largura (grade larga): o slot so' alarga.
        shell.rightPreferredWidth = 300;
        shell.docks.widen("database", 420);
        failures += check(shell.rightPreferredWidth === 420, "alargou ate' o pedido");
        shell.docks.widen("database", 330);
        failures += check(shell.rightPreferredWidth === 420, "nunca encolhe por pedido");
        shell.docks.widen("git", 600);
        failures += check(shell.rightPreferredWidth === 420, "janela fechada nao mexe em nada");

        // TELA DE BOAS-VINDAS (sem projeto): nenhum slot aparece, nem com o
        // layout lembrando uma janela aberta (decisao do autor, 2026-10-03).
        shell.moveRailEntryToSide("containers", "left", 0, []);
        shell.showDockWindow("containers");
        failures += check(!shell.docks.slotVisible("left", false) && shell.docks.slotVisible("left", true),
                          "sem projeto, o slot nao aparece");
        shell.toggleExplorer();

        // Gravar e ler o slot da direita.
        shell.rightPreferredWidth = 360;
        const snapshot = shell.layoutSnapshot();
        failures += check(snapshot.rightWindow === "database" && snapshot.sizes.right === 360, "retrato");
        const values = shell.codec.decode({ rightWindow: "nada", sizes: { right: 9000 } }, shell);
        failures += check(values.rightWindow === undefined && values.rightPreferredWidth === 640,
                          "janela que nao existe nao volta; largura no teto");
        failures += check(shell.codec.decode({ rightWindow: "" }, shell).rightWindow === "", "fechado volta fechado");

        Qt.exit(failures === 0 ? 0 : 1);
    }
}
