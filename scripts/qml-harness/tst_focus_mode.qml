import QtQuick
import KineinVectis

// O modo Foco (0.3.9 F4, roadmap 53 §5.8): entrar recolhe a esquerda, os
// Simbolos e o painel de baixo; sair restaura EXATAMENTE o anterior; abrir um
// painel a' mao durante o Foco encerra o modo sem restaurar.
Item {
    id: root

    ShellController {
        id: shell
    }

    function check(condition, label) {
        if (!condition) console.error("FALHOU: " + label);
        return condition ? 0 : 1;
    }

    function state() {
        return [shell.showExplorer, shell.showBottomPanel, shell.outlineCollapsed].join(",");
    }

    Component.onCompleted: {
        let failures = 0;
        shell.showExplorer = true;
        shell.showBottomPanel = true;
        shell.outlineCollapsed = false;
        const before = state();

        shell.showTab("focusMode");
        failures += check(shell.focusMode.active, "entrou no Foco");
        failures += check(state() === "false,false,true", "recolheu: " + state());

        shell.showTab("focusMode");
        failures += check(!shell.focusMode.active, "saiu do Foco");
        failures += check(state() === before, "restaurou EXATAMENTE: " + state());

        // Com o painel de baixo fechado antes, sair do Foco nao o abre.
        shell.showBottomPanel = false;
        shell.focusMode.toggle();
        shell.focusMode.toggle();
        failures += check(!shell.showBottomPanel, "o fechado continua fechado");

        // Abrir um painel a' mao durante o Foco: o modo acaba, sem restaurar.
        shell.showExplorer = true;
        shell.focusMode.toggle();
        shell.showBottomPanel = true;
        failures += check(!shell.focusMode.active, "abrir a' mao encerra o Foco");
        failures += check(!shell.showExplorer && shell.showBottomPanel,
                          "nada foi restaurado por cima do gesto: " + state());

        // Estreito (53 §5.8): abaixo de 1024 px os Simbolos recolhem sem mudar
        // a preferencia; alargar os traz de volta.
        shell.outlineCollapsed = false;
        shell.viewportWidth = 900;
        failures += check(shell.effectiveOutlineCollapsed && !shell.outlineCollapsed,
                          "estreito recolhe sem apagar a preferencia");
        shell.viewportWidth = 1366;
        failures += check(!shell.effectiveOutlineCollapsed, "alargar devolve os Simbolos");

        if (failures !== 0) console.error("FALHAS " + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
