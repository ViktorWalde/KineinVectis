// O indexController do projeto inteiro na UI (pilar 0 do roadmaps/42, decisao do autor
// em 2026-09-12): o que a barra de status resume e como o `#nome` usa o
// indexController antes de o LSP responder.
//
// Por que existe: a traducao "construindo -> pronto -> resumo" e a regra
// "indexController preenche, LSP substitui" quebram sem compilador que reclame.
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
        id: indexController

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
        if (indexController.summary() !== "") failures += 1;
        indexController.workspaceRoot = "/tmp/proj";
        if (root.pedidosStatus !== 1) failures += 2;
        if (!indexController.building || indexController.summary().indexOf("indexando") !== 0) failures += 4;

        // Progresso aparece; o fim vira o resumo com numeros legiveis.
        indexController.handleProgress(200, 950);
        if (indexController.summary().indexOf("200") < 0 || indexController.summary().indexOf("950") < 0) failures += 8;
        indexController.handleFinished({ state: "ready", files: 1022, lines: 72000, symbols: 3939 });
        if (!indexController.ready) failures += 16;
        if (indexController.summary() !== "1.022 arquivos · 72.000 linhas · 3.939 símbolos") failures += 32;
        // Acima de cem mil, arredonda para "mil": o numero exato e' do painel.
        if (indexController.formatCount(120400) !== "120 mil") failures += 262144;
        indexController.handleFinished({ state: "failed", error: "indexacao cancelada" });
        if (indexController.summary().indexOf("falhou") < 0 || indexController.summary().indexOf("cancelada") < 0) failures += 64;
        // Campo ausente nao vira NaN/undefined.
        if (indexController.formatCount(undefined) !== "0") failures += 128;

        // Trocar de workspace esquece o indexController do anterior e pergunta de novo;
        // FECHAR o workspace deixa a barra vazia — nao o resumo do anterior.
        indexController.handleFinished({ state: "ready", files: 3, lines: 3, symbols: 3 });
        indexController.workspaceRoot = "/tmp/outro";
        if (indexController.ready || root.pedidosStatus !== 2) failures += 256;
        indexController.handleFinished({ state: "ready", files: 3, lines: 3, symbols: 3 });
        indexController.workspaceRoot = "";
        if (indexController.summary() !== "" || indexController.ready) failures += 524288;
        indexController.workspaceRoot = "/tmp/proj";

        // `#nome` SEM arquivo aberto: so' o indexController responde — e responde.
        busca.everywhereVisible = true;
        busca.runSearchEverywhere("#ligar");
        if (root.pedidosIndice.join(",") !== "ligar" || root.pedidosLsp.length !== 0) failures += 512;
        if (busca.everywhereError !== "") failures += 65536;
        // Com arquivo aberto: pede aos DOIS; o indexController preenche; o LSP substitui.
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

        // Depois do LSP, o indexController NAO volta por cima.
        busca.handleIndexSymbols([{ name: "x", kind: "function", path: "a.rs", line: 1 }], 1, "ready");
        if (busca.everywhereModel.count !== 2) failures += 16384;
        // Nova busca: a regra recomeca.
        busca.runSearchEverywhere("#main");
        busca.handleIndexSymbols([{ name: "main", kind: "function", path: "src/main.rs", line: 5 }], 1, "ready");
        if (busca.everywhereModel.count !== 1) failures += 32768;

        // O CONTEXTO DE COMPILADOR do arquivo ativo: pergunta ao trocar de
        // arquivo; resume por linguagem; descarta resposta atrasada de outro
        // arquivo; pergunta de novo quando o indexController termina; some ao fechar.
        indexController.handleFinished({ state: "ready", files: 3, lines: 3, symbols: 3 });
        indexController.setActivePath("/tmp/proj/ui/src/a.cpp");
        if (root.pedidosContexto.join(",") !== "/tmp/proj/ui/src/a.cpp") failures += 1048576;
        if (indexController.contextSummary() !== "") failures += 2097152;
        // Resposta de OUTRO arquivo (a aba ja' trocou): ignorada.
        indexController.handleContext({ path: "/tmp/proj/src/velho.rs", language: "rust",
                               crate: { package: "velho", target: "velho", kind: "bin", edition: "2021", manifest: "/tmp/proj/Cargo.toml", srcPath: "/tmp/proj/src/main.rs", features: [] } });
        if (indexController.contextSummary() !== "") failures += 4194304;
        indexController.handleContext({ path: "/tmp/proj/ui/src/a.cpp", language: "cpp",
                               unit: { compiler: "/usr/lib64/ccache/c++", directory: "/tmp/proj/build/ui", standard: "gnu++23",
                                       includes: ["/a", "/b", "/c"], defines: ["QT_CORE_LIB", "QT_NO_DEBUG"], arguments: [] },
                               source: "compile_commands.json em build" });
        if (indexController.contextSummary() !== "c++ · gnu++23 · 3 -I · 2 -D") failures += 8388608;
        if (indexController.contextDetail().indexOf("/tmp/proj/build/ui") < 0 || indexController.contextDetail().indexOf("compile_commands.json em build") < 0) failures += 16777216;
        // O target dono do arquivo (file-api) entra no detalhe; sem targets, nada e' inventado.
        if (indexController.contextDetail().indexOf("target") >= 0) failures += 68719476736;
        indexController.handleContext({ path: "/tmp/proj/ui/src/a.cpp", language: "cpp",
                               unit: { compiler: "cc", directory: "/tmp/proj/build", includes: [], defines: [], arguments: [] },
                               targets: ["kinein-vectis", "kinein-tests"] });
        if (indexController.contextDetail().indexOf("targets kinein-vectis, kinein-tests") < 0) failures += 137438953472;
        // Unidade velha (CDB envelhecida) leva o aviso no resumo.
        indexController.handleContext({ path: "/tmp/proj/ui/src/a.cpp", language: "cpp",
                               unit: { compiler: "cc", directory: "/tmp/proj/build", includes: [], defines: [], arguments: [] },
                               hint: "a compile_commands.json e' mais velha que ui/CMakeLists.txt: reconfigure" });
        if (indexController.contextSummary() !== "cc · 0 -I · 0 -D ⚠") failures += 33554432;
        if (indexController.contextDetail().indexOf("reconfigure") < 0) failures += 67108864;
        // O efetivo com a origem (0.4.0a, roadmaps/52 §14.1). Sem unidade, o
        // compilador vem do cache do CMake e entra no detalhe; o kit padrao
        // (preset vazio) nao e' dito; campo ausente nao vira "undefined".
        indexController.handleContext({ path: "/tmp/proj/ui/src/a.cpp", language: "cpp",
                               hint: "sem compile_commands.json: configure",
                               effective: { compiler: { value: "/usr/bin/c++", origin: "cmakeCache" },
                                            target: { value: "thumbv7em-none-eabihf", origin: "kit" },
                                            kit: { preset: "" } } });
        const detail = indexController.contextDetail();
        if (detail.indexOf("compilador /usr/bin/c++ (do cache do CMake)") < 0
                || detail.indexOf("alvo thumbv7em-none-eabihf (do kit)") < 0) failures += 274877906944;
        if (detail.indexOf("undefined") >= 0 || detail.indexOf("sysroot") >= 0
                || /(^| — )kit /.test(detail)) failures += 549755813888;
        // Com unidade, o compilador ja' esta' no resumo e nao se repete; o
        // sysroot da CDB e o kit com nome entram.
        indexController.handleContext({ path: "/tmp/proj/ui/src/a.cpp", language: "cpp",
                               unit: { compiler: "cc", directory: "/tmp/proj/build", includes: [], defines: [], arguments: [] },
                               effective: { compiler: { value: "cc", origin: "cdb" },
                                            sysroot: { value: "/srv/raiz", origin: "cdb" },
                                            kit: { preset: "nucleo" } } });
        const withUnit = indexController.contextDetail();
        if (withUnit.indexOf("compilador") >= 0 || withUnit.indexOf("sysroot /srv/raiz (da compile_commands.json)") < 0
                || withUnit.indexOf("kit nucleo") < 0) failures += 1099511627776;
        // O indexController terminou de novo (configure): pergunta de novo pelo ativo.
        indexController.handleFinished({ state: "ready", files: 3, lines: 3, symbols: 3 });
        if (root.pedidosContexto.length !== 2 || root.pedidosContexto[1] !== "/tmp/proj/ui/src/a.cpp") failures += 134217728;
        // Rust e Python: os resumos das outras linguagens.
        indexController.setActivePath("/tmp/proj/src/lib.rs");
        // Ate' a resposta chegar, a barra NAO mostra o contexto do arquivo anterior.
        if (indexController.contextSummary() !== "") failures += 34359738368;
        indexController.handleContext({ path: "/tmp/proj/src/lib.rs", language: "rust",
                               crate: { package: "kinein-core", target: "kinein_core", kind: "lib", edition: "2024", manifest: "/tmp/proj/Cargo.toml", srcPath: "/tmp/proj/src/lib.rs", features: [] },
                               source: "cargo metadata --no-deps" });
        if (indexController.contextSummary() !== "cargo · kinein-core (lib, 2024)") failures += 268435456;
        indexController.setActivePath("/tmp/proj/tools/gera.py");
        indexController.handleContext({ path: "/tmp/proj/tools/gera.py", language: "python",
                               python: { interpreter: "/usr/bin/python3", version: "Python 3.14.7", origin: "sistema", warning: "Python do sistema: instalar pacote nele quebra a distro; crie um .venv" } });
        if (indexController.contextSummary() !== "python · sistema · 3.14.7 ⚠") failures += 536870912;
        if (indexController.contextDetail().indexOf("quebra a distro") < 0) failures += 1073741824;
        // So' dica (sem CDB): resumo curto com aviso; o detalhe e' a dica.
        indexController.setActivePath("/tmp/proj/src/solto.c");
        indexController.handleContext({ path: "/tmp/proj/src/solto.c", language: "c", hint: "sem compile_commands.json: configure o projeto" });
        if (indexController.contextSummary() !== "sem contexto ⚠") failures += 2147483648;
        // Mesmo caminho de novo NAO pergunta de novo; fechar a aba limpa.
        const antes = root.pedidosContexto.length;
        indexController.setActivePath("/tmp/proj/src/solto.c");
        if (root.pedidosContexto.length !== antes) failures += 4294967296;
        indexController.setActivePath("");
        if (indexController.contextSummary() !== "" || root.pedidosContexto.length !== antes) failures += 8589934592;
        // Sem workspace nao ha' a quem perguntar.
        indexController.workspaceRoot = "";
        indexController.setActivePath("/tmp/proj/src/a.c");
        if (root.pedidosContexto.length !== antes) failures += 17179869184;

        // A aba Simbolos (E3-2) e' filha: o caminho relativo vem do arquivo
        // ativo; abrir devolve o ABSOLUTO; o indexController pronto refaz a busca
        // que foi feita com ele ainda lendo.
        indexController.workspaceRoot = "/tmp/proj";
        indexController.setActivePath("/tmp/proj/crates/x/src/lib.rs");
        if (indexController.symbols.activeRelativePath !== "crates/x/src/lib.rs" || indexController.symbols.activeFolder !== "crates/x/src") failures += 137438953472;
        indexController.symbols.setQuery("abrir");
        indexController.symbols.requestNow();
        const pedidosAntes = root.pedidosSimbolos.length;
        indexController.symbols.handleIndexSymbols([], 0, "building");
        indexController.handleFinished({ state: "ready", files: 3, lines: 3, symbols: 3 });
        if (root.pedidosSimbolos.length !== pedidosAntes + 1 || root.pedidosSimbolos[pedidosAntes] !== "abrir") failures += 274877906944;
        indexController.symbols.handleIndexSymbols([{ name: "abrir", kind: "function", path: "crates/x/src/lib.rs", line: 9 }], 1, "ready");
        indexController.symbols.open(indexController.symbols.results[0]);
        if (!root.aberto || root.aberto.file !== "/tmp/proj/crates/x/src/lib.rs" || root.aberto.line !== 9) failures += 549755813888;
        // Sem texto, o indexController pronto nao pede nada.
        indexController.symbols.setQuery("");
        indexController.handleFinished({ state: "ready", files: 3, lines: 3, symbols: 3 });
        if (root.pedidosSimbolos.length !== pedidosAntes + 1) failures += 1099511627776;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
