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
    property var pedidosContexto: []
    property var pedidosIndice: []
    property var pedidosLsp: []

    IndexController {
        id: indice

        onStatusRequested: root.pedidosStatus += 1
        onContextRequested: function(p) { root.pedidosContexto.push(p); }
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

        // O CONTEXTO DE COMPILADOR do arquivo ativo: pergunta ao trocar de
        // arquivo; resume por linguagem; descarta resposta atrasada de outro
        // arquivo; pergunta de novo quando o indice termina; some ao fechar.
        indice.handleFinished({ state: "ready", files: 3, lines: 3, symbols: 3 });
        indice.setActivePath("/tmp/proj/ui/src/a.cpp");
        if (root.pedidosContexto.join(",") !== "/tmp/proj/ui/src/a.cpp") failures += 1048576;
        if (indice.contextSummary() !== "") failures += 2097152;
        // Resposta de OUTRO arquivo (a aba ja' trocou): ignorada.
        indice.handleContext({ path: "/tmp/proj/src/velho.rs", language: "rust",
                               crate: { package: "velho", target: "velho", kind: "bin", edition: "2021", manifest: "/tmp/proj/Cargo.toml", srcPath: "/tmp/proj/src/main.rs", features: [] } });
        if (indice.contextSummary() !== "") failures += 4194304;
        indice.handleContext({ path: "/tmp/proj/ui/src/a.cpp", language: "cpp",
                               unit: { compiler: "/usr/lib64/ccache/c++", directory: "/tmp/proj/build/ui", standard: "gnu++23",
                                       includes: ["/a", "/b", "/c"], defines: ["QT_CORE_LIB", "QT_NO_DEBUG"], arguments: [] },
                               source: "compile_commands.json em build" });
        if (indice.contextSummary() !== "c++ · gnu++23 · 3 -I · 2 -D") failures += 8388608;
        if (indice.contextDetail().indexOf("/tmp/proj/build/ui") < 0 || indice.contextDetail().indexOf("compile_commands.json em build") < 0) failures += 16777216;
        // Unidade velha (CDB envelhecida) leva o aviso no resumo.
        indice.handleContext({ path: "/tmp/proj/ui/src/a.cpp", language: "cpp",
                               unit: { compiler: "cc", directory: "/tmp/proj/build", includes: [], defines: [], arguments: [] },
                               hint: "a compile_commands.json e' mais velha que ui/CMakeLists.txt: reconfigure" });
        if (indice.contextSummary() !== "cc · 0 -I · 0 -D ⚠") failures += 33554432;
        if (indice.contextDetail().indexOf("reconfigure") < 0) failures += 67108864;
        // O indice terminou de novo (configure): pergunta de novo pelo ativo.
        indice.handleFinished({ state: "ready", files: 3, lines: 3, symbols: 3 });
        if (root.pedidosContexto.length !== 2 || root.pedidosContexto[1] !== "/tmp/proj/ui/src/a.cpp") failures += 134217728;
        // Rust e Python: os resumos das outras linguagens.
        indice.setActivePath("/tmp/proj/src/lib.rs");
        // Ate' a resposta chegar, a barra NAO mostra o contexto do arquivo anterior.
        if (indice.contextSummary() !== "") failures += 34359738368;
        indice.handleContext({ path: "/tmp/proj/src/lib.rs", language: "rust",
                               crate: { package: "kinein-core", target: "kinein_core", kind: "lib", edition: "2024", manifest: "/tmp/proj/Cargo.toml", srcPath: "/tmp/proj/src/lib.rs", features: [] },
                               source: "cargo metadata --no-deps" });
        if (indice.contextSummary() !== "cargo · kinein-core (lib, 2024)") failures += 268435456;
        indice.setActivePath("/tmp/proj/tools/gera.py");
        indice.handleContext({ path: "/tmp/proj/tools/gera.py", language: "python",
                               python: { interpreter: "/usr/bin/python3", version: "Python 3.14.7", origin: "sistema", warning: "Python do sistema: instalar pacote nele quebra a distro; crie um .venv" } });
        if (indice.contextSummary() !== "python · sistema · 3.14.7 ⚠") failures += 536870912;
        if (indice.contextDetail().indexOf("quebra a distro") < 0) failures += 1073741824;
        // So' dica (sem CDB): resumo curto com aviso; o detalhe e' a dica.
        indice.setActivePath("/tmp/proj/src/solto.c");
        indice.handleContext({ path: "/tmp/proj/src/solto.c", language: "c", hint: "sem compile_commands.json: configure o projeto" });
        if (indice.contextSummary() !== "sem contexto ⚠") failures += 2147483648;
        // Mesmo caminho de novo NAO pergunta de novo; fechar a aba limpa.
        const antes = root.pedidosContexto.length;
        indice.setActivePath("/tmp/proj/src/solto.c");
        if (root.pedidosContexto.length !== antes) failures += 4294967296;
        indice.setActivePath("");
        if (indice.contextSummary() !== "" || root.pedidosContexto.length !== antes) failures += 8589934592;
        // Sem workspace nao ha' a quem perguntar.
        indice.workspaceRoot = "";
        indice.setActivePath("/tmp/proj/src/a.c");
        if (root.pedidosContexto.length !== antes) failures += 17179869184;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
