import QtQuick
// Carrega o controller REAL do contexto de compilação.
import "../../ui/qml/project"

Item {
    id: root
    width: 100
    height: 100

    // Theme falso: o controller só lê tokens de cor, e carregar o Theme real
    // arrastaria o módulo QML inteiro para dentro de um teste de lógica.
    readonly property var temaFalso: ({
        successSoft: "verde", warningSoft: "ambar",
        textSecondary: "cinza", textMuted: "apagado"
    })

    CompileContextController {
        id: ctx
    }

    Component.onCompleted: {
        let failures = 0;

        // Estado inicial: sem contexto. O painel mostra o convite, não zeros.
        if (ctx.hasContext) failures += 1;
        if (ctx.originLabel !== "") failures += 2;

        // EXATO: o comando é do próprio arquivo. Verde.
        ctx.handleResolved({
            path: "src/a.cpp", origin: "exact", compiler: "/usr/bin/c++",
            standard: "gnu++20", defines: ["A=1", "B"], includes: ["/inc"],
            flags: ["-Wall"], database: ".kinein/build/compile_commands.json",
            command: "/usr/bin/c++ -DA=1 -c src/a.cpp"
        });
        if (!ctx.hasContext) failures += 4;
        if (ctx.originLabel.indexOf("próprio") < 0) failures += 8;
        if (ctx.originColor(root.temaFalso) !== "verde") failures += 16;
        if (ctx.defines.length !== 2) failures += 32;

        // EMPRESTADO: tem que DIZER de quem. É a razão de a fatia existir —
        // apresentar palpite como fato seria a IDE mentindo sobre o que sabe.
        ctx.handleResolved({
            path: "src/a.hpp", origin: "borrowed", borrowedFrom: "src/a.cpp",
            compiler: "/usr/bin/c++", standard: "gnu++20", defines: [],
            includes: [], flags: [], database: "d.json", command: "c++ -c a.cpp"
        });
        if (ctx.originLabel.indexOf("src/a.cpp") < 0) failures += 64;
        if (ctx.originLabel.indexOf("emprestado") < 0) failures += 128;
        if (ctx.originColor(root.temaFalso) !== "ambar") failures += 256;

        // Emprestado SEM saber de quem ainda avisa que é emprestado.
        ctx.handleResolved({ path: "x.hpp", origin: "borrowed" });
        if (ctx.originLabel.indexOf("emprestado") < 0) failures += 512;
        if (ctx.originColor(root.temaFalso) !== "ambar") failures += 1024;

        // NENHUM: estado próprio, com cor apagada — não pode parecer exato.
        ctx.handleResolved({
            path: "README.md", origin: "none",
            note: "o arquivo não está no banco de comandos"
        });
        if (ctx.originLabel.indexOf("sem contexto") < 0) failures += 2048;
        if (ctx.originColor(root.temaFalso) !== "apagado") failures += 4096;
        if (ctx.note === "") failures += 8192;
        // Campos do arquivo anterior NÃO podem vazar para este.
        if (ctx.compiler !== "") failures += 16384;
        if (ctx.defines.length !== 0) failures += 32768;

        // Resposta vazia limpa em vez de manter o anterior na tela.
        ctx.handleResolved({
            path: "src/b.cpp", origin: "exact", compiler: "cc"
        });
        ctx.handleResolved(null);
        if (ctx.hasContext) failures += 65536;
        if (ctx.path !== "") failures += 131072;

        // clear() do workspace leva o contexto junto: caminho relativo do
        // projeto anterior colidiria com o do próximo.
        ctx.handleResolved({ path: "src/c.cpp", origin: "exact" });
        ctx.clear();
        if (ctx.hasContext) failures += 262144;
        if (ctx.origin !== "") failures += 524288;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
