// O indice do projeto inteiro na UI (pilar 0 do roadmaps/42, decisao do autor
// em 2026-09-12): o que a barra de status resume e como o `#nome` usa o
// indice antes de o LSP responder.
//
// Por que existe: a traducao "construindo -> pronto -> resumo" e a regra
// "indice preenche, LSP substitui" quebram sem compilador que reclame.
import QtQuick
import "../../ui/qml/index"
import "../../ui/qml/search"

Item {
    id: root

    property int pedidosStatus: 0
    property var pedidosIndice: []
    property var pedidosLsp: []

    IndexController {
        id: indice

        onStatusRequested: root.pedidosStatus += 1
    }

    SearchEverywhereController {
        id: busca

        workspaceRoot: "/tmp/proj"
        onIndexSymbolsRequested: function(q) { root.pedidosIndice.push(q); }
        onWorkspaceSymbolsRequested: function(q) { root.pedidosLsp.push(q); }
    }

    Component.onCompleted: {
        let failures = 0;

        // Sem workspace: nada a mostrar. Com workspace: "indexando" e pergunta.
        if (indice.summary() !== "") failures += 1;
        indice.workspaceRoot = "/tmp/proj";
        if (root.pedidosStatus !== 1) failures += 2;
        if (!indice.building || indice.summary().indexOf("indexando") !== 0) failures += 4;

        // Progresso aparece; o fim vira o resumo com numeros legiveis.
        indice.handleProgress(200, 950);
        if (indice.summary().indexOf("200") < 0 || indice.summary().indexOf("950") < 0) failures += 8;
        indice.handleFinished({ state: "ready", files: 1010, lines: 70030, symbols: 4658 });
        if (!indice.ready) failures += 16;
        if (indice.summary() !== "1.010 arquivos · 70.030 linhas · 4.658 símbolos") failures += 32;
        // Acima de cem mil, arredonda para "mil": o numero exato e' do painel.
        if (indice.formatCount(120400) !== "120 mil") failures += 262144;
        indice.handleFinished({ state: "failed", error: "indexacao cancelada" });
        if (indice.summary().indexOf("falhou") < 0 || indice.summary().indexOf("cancelada") < 0) failures += 64;
        // Campo ausente nao vira NaN/undefined.
        if (indice.formatCount(undefined) !== "0") failures += 128;

        // Trocar de workspace esquece o indice do anterior e pergunta de novo;
        // FECHAR o workspace deixa a barra vazia — nao o resumo do anterior.
        indice.handleFinished({ state: "ready", files: 3, lines: 3, symbols: 3 });
        indice.workspaceRoot = "/tmp/outro";
        if (indice.ready || root.pedidosStatus !== 2) failures += 256;
        indice.handleFinished({ state: "ready", files: 3, lines: 3, symbols: 3 });
        indice.workspaceRoot = "";
        if (indice.summary() !== "" || indice.ready) failures += 524288;
        indice.workspaceRoot = "/tmp/proj";

        // `#nome` SEM arquivo aberto: so' o indice responde — e responde.
        busca.everywhereVisible = true;
        busca.runSearchEverywhere("#ligar");
        if (root.pedidosIndice.join(",") !== "ligar" || root.pedidosLsp.length !== 0) failures += 512;
        if (busca.everywhereError !== "") failures += 65536;
        // Com arquivo aberto: pede aos DOIS; o indice preenche; o LSP substitui.
        busca.hasActiveEditorFile = true;
        busca.runSearchEverywhere("#ligar");
        if (root.pedidosIndice.join(",") !== "ligar,ligar" || root.pedidosLsp.join(",") !== "ligar") failures += 131072;
        busca.handleIndexSymbols([{ name: "ligar", kind: "method", path: "src/main.rs", line: 3, container: "Motor" }], 1, "ready");
        if (busca.everywhereModel.count !== 1) failures += 1024;
        // Caminho relativo virou absoluto (o clique abre o arquivo).
        if (busca.everywhereModel.get(0).path !== "/tmp/proj/src/main.rs") failures += 2048;
        if (busca.everywhereLoading) failures += 4096;
        busca.handleSymbolsResolved([{ name: "ligar", kind: "Method", path: "/tmp/proj/src/main.rs", line: 3 },
                                     { name: "ligar_tudo", kind: "Function", path: "/tmp/proj/src/main.rs", line: 6 }]);
        if (busca.everywhereModel.count !== 2) failures += 8192;
        // Depois do LSP, o indice NAO volta por cima.
        busca.handleIndexSymbols([{ name: "x", kind: "function", path: "a.rs", line: 1 }], 1, "ready");
        if (busca.everywhereModel.count !== 2) failures += 16384;
        // Nova busca: a regra recomeca.
        busca.runSearchEverywhere("#main");
        busca.handleIndexSymbols([{ name: "main", kind: "function", path: "src/main.rs", line: 5 }], 1, "ready");
        if (busca.everywhereModel.count !== 1) failures += 32768;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
