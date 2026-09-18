import QtQuick
import "../../ui/qml/jobs"

// O proximo passo de um problema (Etapa 2, F5, 2026-09-18): a regra pura.
Item {
    ProblemNextStep { id: regra }

    Component.onCompleted: {
        let failures = 0;
        let s = regra.stepFor({ source: "lsp", message: "unused variable `x`" });
        if (s.kind !== "codeActions" || s.label === "") failures += 1;
        s = regra.stepFor({ source: "quality", message: "clang-tidy: compile_commands.json nao encontrado em .kinein/build" });
        if (s.kind !== "health" || s.target !== "cmakeConfigure") failures += 2;
        s = regra.stepFor({ source: "build", message: "projeto CMake nao configurado" });
        if (s.target !== "cmakeConfigure") failures += 4;
        s = regra.stepFor({ source: "quality", message: "ruff nao foi encontrado no PATH — instale com uv" });
        if (s.kind !== "health" || s.target !== "tools") failures += 8;
        s = regra.stepFor({ source: "build", message: "error[E0308]: mismatched types" });
        if (s.kind !== "" || s.label !== "") failures += 16;
        s = regra.stepFor({});
        if (s.kind !== "") failures += 32;
        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
