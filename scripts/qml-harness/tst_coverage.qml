import QtQuick
import "../../ui/qml/coverage"

// A cobertura dos testes (D8, 2026-09-17): o CoverageController REAL com o
// roteador falso.
//
// O que se prova: sem relatorio, trocar de arquivo nao pede nada; o
// desfecho guarda o resumo e pede as linhas do arquivo ativo; as linhas de
// OUTRO arquivo nao entram; o mapa linha -> covered|missed e a revisao
// avancam; o resumo do arquivo ativo e' a porcentagem; a falha vira texto;
// trocar de workspace esquece tudo.
Item {
    id: root

    property var pedidos: []

    CoverageController {
        id: c

        onLinesRequested: function(file) { root.pedidos.push(file); }
    }

    Component.onCompleted: {
        let failures = 0;
        c.workspaceRoot = "/w";

        // Sem relatorio: o arquivo ativo muda, nada e' pedido.
        c.setActivePath("/w/src/lib.rs");
        if (root.pedidos.length !== 0 || c.hasReport || c.activeSummary() !== "") failures += 1;

        // O desfecho: resumo, e as linhas do arquivo ativo pedidas.
        const rev = c.revision;
        c.handleFinished({ success: true, tool: "cargo llvm-cov", path: "/w/.kinein/coverage.lcov",
                           files: [{ file: "/w/src/lib.rs", linesFound: 10, linesHit: 7 }, { file: "/w/src/b.rs", linesFound: 4, linesHit: 0 }] });
        if (!c.hasReport || c.files.length !== 2 || c.totalFound() !== 14 || c.totalHit() !== 7) failures += 2;
        if (c.lastOutcome.indexOf("2 arquivo(s), 7/14") < 0) failures += 4;
        if (root.pedidos.join(",") !== "/w/src/lib.rs") failures += 8;

        // Linhas de OUTRO arquivo nao entram; as do ativo pintam.
        c.handleLines("/w/src/b.rs", true, [1], [2, 3, 4]);
        if (c.activeKnown || Object.keys(c.lineKinds).length !== 0) failures += 16;
        c.handleLines("/w/src/lib.rs", true, [1, 2, 5], [3]);
        if (!c.activeKnown || c.lineKinds[1] !== "covered" || c.lineKinds[3] !== "missed" || c.lineKinds[4] !== undefined) failures += 32;
        if (c.revision <= rev) failures += 64;
        if (c.activeSummary() !== "cobertura 75%") failures += 128;

        // Trocar de arquivo pede as linhas dele e limpa o mapa ate' chegarem.
        c.setActivePath("/w/src/b.rs");
        if (root.pedidos.length !== 2 || root.pedidos[1] !== "/w/src/b.rs" || Object.keys(c.lineKinds).length !== 0) failures += 256;
        c.handleLines("/w/src/b.rs", false, [], []);
        if (c.activeKnown || c.activeSummary() !== "") failures += 512;
        // Arquivo vazio (nenhuma aba): nao pede.
        c.setActivePath("");
        if (root.pedidos.length !== 2) failures += 1024;

        // A falha vira texto, e o relatorio deixa de valer.
        c.handleFinished({ success: false, tool: "", error: "cargo-llvm-cov nao esta' nesta maquina" });
        if (c.hasReport || c.lastOutcome.indexOf("falhou") !== 0 || c.files.length !== 0) failures += 2048;

        // Trocar de workspace esquece tudo.
        c.handleFinished({ success: true, tool: "coverage.py", files: [{ file: "/w/app.py", linesFound: 1, linesHit: 1 }] });
        c.workspaceRoot = "/outro";
        if (c.hasReport || c.files.length !== 0 || c.lastOutcome !== "" || c.activePath !== "") failures += 4096;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
