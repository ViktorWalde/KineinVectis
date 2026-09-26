// O indice do projeto inteiro na UI (pilar 0 do roadmaps/42, decisao do autor
// em 2026-09-12): o que a barra de status resume e como o `#nome` usa o
// indice antes de o LSP responder.
//
// Por que existe: a traducao "construindo -> pronto -> resumo" e a regra
// "indice preenche, LSP substitui" quebram sem compilador que reclame.
import QtQuick
// Pelo MODULO, e nao pela pasta: o SearchEverywhereController passou a usar o
// `PathRules` (que mora em `editor/`) e o `EverywhereSymbolOrigin`, e um import
// de diretorio nao alcanca as duas pastas de uma vez. O espelho plano do
// harness tem todos.
import KineinVectis

Item {
    id: root

    property int pedidosStatus: 0
    property var pedidosContexto: []
    property var pedidosIndice: []
    property var pedidosLsp: []
    property var pedidosSimbolos: []
    property var aberto: null

    IndexController {
        id: indice

        onStatusRequested: root.pedidosStatus += 1
        onContextRequested: function(p) { root.pedidosContexto.push(p); }
        symbols.onIndexSymbolsRequested: function(q) { root.pedidosSimbolos.push(q); }
        onOpenSymbolRequested: function(file, line, column) { root.aberto = { file: file, line: line, column: column }; }
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
        indice.handleFinished({ state: "ready", files: 1022, lines: 72000, symbols: 3939 });
        if (!indice.ready) failures += 16;
        if (indice.summary() !== "1.022 arquivos · 72.000 linhas · 3.939 símbolos") failures += 32;
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
        // A resposta do WORKSPACE agora se identifica pela query (L1, 0.135.0):
        // uma resposta de outra pergunta nao entra mais nesta lista.
        busca.handleWorkspaceSymbols("ligar",
                                     [{ name: "ligar", kind: "Method", path: "/tmp/proj/src/main.rs", line: 3 },
                                      { name: "ligar_tudo", kind: "Function", path: "/tmp/proj/src/main.rs", line: 6 }]);
        if (busca.everywhereModel.count !== 2) failures += 8192;
        // E a resposta de uma query ANTIGA e' descartada.
        busca.handleWorkspaceSymbols("outra", [{ name: "z", kind: "Function", path: "/tmp/proj/z.rs", line: 1 }]);
        if (busca.everywhereModel.count !== 2) failures += 262144;

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
        // O target dono do arquivo (file-api) entra no detalhe; sem targets, nada e' inventado.
        if (indice.contextDetail().indexOf("target") >= 0) failures += 68719476736;
        indice.handleContext({ path: "/tmp/proj/ui/src/a.cpp", language: "cpp",
                               unit: { compiler: "cc", directory: "/tmp/proj/build", includes: [], defines: [], arguments: [] },
                               targets: ["kinein-vectis", "kinein-tests"] });
        if (indice.contextDetail().indexOf("targets kinein-vectis, kinein-tests") < 0) failures += 137438953472;
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

        // A aba Simbolos (E3-2) e' filha: o caminho relativo vem do arquivo
        // ativo; abrir devolve o ABSOLUTO; o indice pronto refaz a busca
        // que foi feita com ele ainda lendo.
        indice.workspaceRoot = "/tmp/proj";
        indice.setActivePath("/tmp/proj/crates/x/src/lib.rs");
        if (indice.symbols.activeRelativePath !== "crates/x/src/lib.rs" || indice.symbols.activeFolder !== "crates/x/src") failures += 137438953472;
        indice.symbols.setQuery("abrir");
        indice.symbols.requestNow();
        const pedidosAntes = root.pedidosSimbolos.length;
        indice.symbols.handleIndexSymbols([], 0, "building");
        indice.handleFinished({ state: "ready", files: 3, lines: 3, symbols: 3 });
        if (root.pedidosSimbolos.length !== pedidosAntes + 1 || root.pedidosSimbolos[pedidosAntes] !== "abrir") failures += 274877906944;
        indice.symbols.handleIndexSymbols([{ name: "abrir", kind: "function", path: "crates/x/src/lib.rs", line: 9 }], 1, "ready");
        indice.symbols.open(indice.symbols.results[0]);
        if (!root.aberto || root.aberto.file !== "/tmp/proj/crates/x/src/lib.rs" || root.aberto.line !== 9) failures += 549755813888;
        // Sem texto, o indice pronto nao pede nada.
        indice.symbols.setQuery("");
        indice.handleFinished({ state: "ready", files: 3, lines: 3, symbols: 3 });
        if (root.pedidosSimbolos.length !== pedidosAntes + 1) failures += 1099511627776;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
