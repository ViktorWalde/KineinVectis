import QtQuick
import "../../ui/qml/shell"

// As tool windows como DADO (fatia V3, 2026-09-24).
//
// O aceite da V3: acrescentar uma janela ao trilho nao pode exigir branching
// nominal em varios arquivos ao mesmo tempo. Antes eram tres lugares no
// `SideRail` e dois no `ShellWorkspaceHost`, por entrada. Agora e' UMA entrada
// aqui — e este harness trava o contrato dessa entrada.
Item {
    id: root

    property int failures: 0
    property var chamadas: []

    function check(ok, mensagem) {
        if (!ok) {
            failures++;
            console.error(mensagem);
        }
    }

    QtObject {
        id: shellFalso
        property bool effectiveShowExplorer: false
        property bool gitWindowVisible: false
        property bool showBottomPanel: false
        property string bottomTab: ""
        function toggleExplorer() { root.chamadas.push("explorer"); }
        function toggleBottomTab(tab) { root.chamadas.push("tab:" + tab); }
    }

    function painel(nome) {
        return Qt.createQmlObject(
            'import QtQuick; QtObject { property bool panelVisible: false; '
            + 'function open() { chamadasDoTeste.push("' + nome + '"); } }',
            root);
    }

    property var chamadasDoTeste: []

    ToolWindows {
        id: janelas

        shellController: shellFalso
        workspaceOpen: false
    }

    Component.onCompleted: {
        // O CONTRATO de uma entrada: todo campo que o trilho desenha.
        check(janelas.entries.length === 6, "seis entradas, deu " + janelas.entries.length);
        for (const e of janelas.entries) {
            check(e.id !== undefined && e.id !== "", "entrada sem id");
            check(e.icon !== undefined && e.icon !== "", e.id + " sem icone");
            check(e.tooltip !== undefined && e.tooltip !== "", e.id + " sem tooltip");
            check(e.area === "left", e.id + " fora do slot esquerdo: " + e.area);
            check(typeof e.order === "number", e.id + " sem ordem");
            check(typeof e.available === "boolean", e.id + " sem disponibilidade");
            check(typeof e.active === "boolean", e.id + " sem ativo");
        }

        // A ordem e' a decidida pelo autor, e nao pode mudar por acidente.
        const ids = janelas.entries.map(function(e) { return e.id; });
        check(ids.join(",") === "explorer,embedded,database,containers,observability,tools",
              "ordem do trilho mudou: " + ids.join(","));
        let anterior = -1;
        for (const e of janelas.entries) {
            check(e.order > anterior, "ordem nao crescente em " + e.id);
            anterior = e.order;
        }

        // DISPONIBILIDADE: Projeto/Git/Embarcados exigem projeto aberto; banco,
        // containers e Grafana sao da MAQUINA e abrem sem projeto.
        function porId(id) {
            return janelas.entries.filter(function(e) { return e.id === id; })[0];
        }
        check(!porId("explorer").available && !porId("embedded").available,
              "sem projeto, Projeto e Embarcados ficam indisponiveis");
        check(porId("database").available && porId("containers").available
              && porId("observability").available && porId("tools").available,
              "os da maquina abrem sem projeto");
        janelas.workspaceOpen = true;
        check(porId("explorer").available && porId("embedded").available,
              "com projeto, Projeto e Embarcados liberam");

        // O GIT NAO ESTA' no trilho desde 2026-09-24: o widget do cabecalho
        // abre o mesmo painel e diz mais. Travado aqui para nao voltar por
        // distracao — dois caminhos cegos para o mesmo gesto.
        check(porId("git") === undefined, "Git nao volta ao trilho");
        check(janelas.activate("git") === false, "e o trilho nao trata git");

        // ATIVO segue o estado real, nao a existencia do controller.
        check(!porId("tools").active, "tools exige a aba certa, nao so' o painel");
        shellFalso.showBottomPanel = true;
        shellFalso.bottomTab = "git";
        check(!porId("tools").active, "painel aberto noutra aba nao acende Ferramentas");
        shellFalso.bottomTab = "tools";
        check(porId("tools").active, "aba tools acende Ferramentas");

        // ATIVAR: um dono so' sabe o que cada id faz.
        check(janelas.activate("explorer") === true && chamadas[0] === "explorer", "explorer");
        check(janelas.activate("tools") === true && chamadas[1] === "tab:tools", "tools");

        // Id sem dono e' resultado OBSERVAVEL, como no CommandDispatcher.
        check(janelas.activate("nao.existe") === false, "id sem dono devolve false");
        check(chamadas.length === 2, "id sem dono nao pode tocar em nada");

        Qt.exit(failures === 0 ? 0 : 1);
    }
}
