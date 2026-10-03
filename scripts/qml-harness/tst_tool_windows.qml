import QtQuick
// Pelo MODULO, e nao pela pasta: desde que a entrada carrega o painel dela
// (V3, campo `componente`), o `ToolWindows` referencia tipos de outras pastas —
// `GrafanaPanelHost`, `RemotePanelHost` — que um import de diretorio nao
// resolve. O espelho plano do harness tem todos.
import KineinVectis

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
        id: toolWindows

        shellController: shellFalso
        workspaceOpen: false
    }

    Component.onCompleted: {
        // O CONTRATO de uma entrada: todo campo que o trilho desenha.
        // Nove desde 0.3.9: os Simbolos (nascem no trilho da direita) e o
        // Terminal entraram.
        check(toolWindows.entries.length === 9, "nove entradas, deu " + toolWindows.entries.length);
        for (const e of toolWindows.entries) {
            check(e.id !== undefined && e.id !== "", "entrada sem id");
            check(e.icon !== undefined && e.icon !== "", e.id + " sem icone");
            check(e.tooltip !== undefined && e.tooltip !== "", e.id + " sem tooltip");
            check(e.area === "left", e.id + " fora do slot esquerdo: " + e.area);
            check(typeof e.order === "number", e.id + " sem ordem");
            check(typeof e.available === "boolean", e.id + " sem disponibilidade");
            check(typeof e.active === "boolean", e.id + " sem ativo");
        }

        // Cada area num trilho: os Simbolos nascem a' direita; o resto a'
        // esquerda; arrastar (sides) muda o lado.
        check(toolWindows.sideOf(toolWindows.entries[2]) === "right", "simbolos nascem a' direita");
        check(toolWindows.sideOf(toolWindows.entries[0]) === "left", "projeto a' esquerda");

        // A ordem e' a decidida pelo autor, e nao pode mudar por acidente.
        const ids = toolWindows.entries.map(function(e) { return e.id; });
        check(ids.join(",") === "explorer,terminal,outline,embedded,database,containers,remote,observability,tools",
              "ordem do trilho mudou: " + ids.join(","));
        let anterior = -1;
        for (const e of toolWindows.entries) {
            check(e.order > anterior, "ordem nao crescente em " + e.id);
            anterior = e.order;
        }

        // DISPONIBILIDADE: Projeto/Git/Embarcados exigem projeto aberto; banco,
        // containers e Grafana sao da MAQUINA e abrem sem projeto.
        function porId(id) {
            return toolWindows.entries.filter(function(e) { return e.id === id; })[0];
        }
        check(!porId("explorer").available && !porId("embedded").available,
              "sem projeto, Projeto e Embarcados ficam indisponiveis");
        check(porId("database").available && porId("containers").available
              && porId("observability").available && porId("tools").available
              && porId("remote").available,
              "os da maquina abrem sem projeto");
        // Os trilhos so' levam o que se usa agora (pente fino 0.3.9): sem
        // projeto, Projeto, Terminal e Simbolos ficam de fora dos dois.
        const railIds = function() {
            return toolWindows.leftEntries.concat(toolWindows.rightEntries)
                .map(function(e) { return e.id; });
        };
        check(railIds().indexOf("explorer") < 0 && railIds().indexOf("terminal") < 0
              && railIds().indexOf("outline") < 0,
              "sem projeto, o trilho nao mostra areas que nao abrem: " + railIds().join(","));
        toolWindows.workspaceOpen = true;
        check(porId("explorer").available && porId("embedded").available,
              "com projeto, Projeto e Embarcados liberam");
        check(railIds().indexOf("explorer") >= 0 && railIds().indexOf("terminal") >= 0,
              "com projeto, Projeto e Terminal voltam ao trilho: " + railIds().join(","));

        // O GIT NAO ESTA' no trilho desde 2026-09-24: o widget do cabecalho
        // abre o mesmo painel e diz mais. Travado aqui para nao voltar por
        // distracao — dois caminhos cegos para o mesmo gesto.
        check(porId("git") === undefined, "Git nao volta ao trilho");
        check(toolWindows.activate("git") === false, "e o trilho nao trata git");

        // ATIVO segue o estado real, nao a existencia do controller.
        check(!porId("tools").active, "tools exige a aba certa, nao so' o painel");
        shellFalso.showBottomPanel = true;
        shellFalso.bottomTab = "git";
        check(!porId("tools").active, "painel aberto noutra aba nao acende Ferramentas");
        shellFalso.bottomTab = "tools";
        check(porId("tools").active, "aba tools acende Ferramentas");

        // ATIVAR: um dono so' sabe o que cada id faz.
        check(toolWindows.activate("explorer") === true && chamadas[0] === "explorer", "explorer");
        check(toolWindows.activate("tools") === true && chamadas[1] === "tab:tools", "tools");

        // O REMOTO (V4): activate abre o painel do controller, e "active" segue o
        // painel — nao a existencia do controller. Sem controller, nao acende e
        // nao estoura.
        check(!porId("remote").active, "sem controller, o remoto nao acende");
        check(toolWindows.activate("remote") === false, "sem controller, nada a ativar");
        const remotoFalso = painel("remote");
        toolWindows.remoteController = remotoFalso;
        check(!porId("remote").active, "controller com painel fechado nao acende");
        check(toolWindows.activate("remote") === true
              && chamadasDoTeste[chamadasDoTeste.length - 1] === "remote",
              "activate('remote') chama o open do dono");
        remotoFalso.panelVisible = true;
        check(porId("remote").active, "painel aberto acende o remoto");

        // O CAMPO `componente` DA V3, que so' entrou quando ganhou consumidor:
        // a entrada carrega o painel dela. Quem monta os overlays le' esta
        // lista em vez de conhecer cada painel pelo nome.
        const comPainel = toolWindows.overlayEntries.map(function(e) { return e.id; });
        check(comPainel.join(",") === "embedded,database,containers,remote,observability",
              "os paineis de ambiente, na ordem do trilho: " + comPainel.join(","));
        for (const e of toolWindows.overlayEntries) {
            check(e.panel !== undefined && e.panel !== null, e.id + " sem componente");
        }
        // Explorer e Ferramentas NAO tem painel de ambiente: o primeiro e' o
        // slot esquerdo, o segundo e' aba do rodape. Dar-lhes `componente`
        // seria forcar a abstracao sobre quem nao e'.
        check(porId("explorer").panel === undefined, "explorer nao e' overlay");
        check(porId("tools").panel === undefined, "tools nao e' overlay");

        // Id sem dono e' resultado OBSERVAVEL, como no CommandDispatcher.
        check(toolWindows.activate("nao.existe") === false, "id sem dono devolve false");
        check(chamadas.length === 2 && chamadasDoTeste.length === 1,
              "id sem dono nao pode tocar em nada");

        Qt.exit(failures === 0 ? 0 : 1);
    }
}
