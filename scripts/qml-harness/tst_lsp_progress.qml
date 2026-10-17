import QtQuick
import "../../ui/qml/workspace"

// L6 (0.4.0d, roadmaps/52 §14.4): a frente de trabalho do servidor na barra.
// O LspStatusController REAL: "LSP … rust · Indexing 43%" enquanto a frente
// dura; sem porcentagem, sem "undefined"; o fim volta ao "LSP ● N"; o servidor
// que cai leva a frente junto; a falha vence o progresso; trocar de projeto
// esquece.
Item {
    id: root

    LspStatusController {
        id: status
    }

    function check(condition, label) {
        if (!condition) console.error("FALHOU: " + label);
        return condition ? 0 : 1;
    }

    Component.onCompleted: {
        let failures = 0;
        status.workspaceRoot = "/tmp/p";
        status.handleStatus("rust", "running", "");
        status.handleStatus("cpp", "running", "");
        failures += check(status.summary() === "LSP ● 2", "dois rodando: " + status.summary());

        status.handleProgress("rust", "Indexing", "12/40 (core)", 43, false);
        failures += check(status.summary() === "LSP … rust · Indexing 43%", "a frente: " + status.summary());
        failures += check(status.detail().indexOf("Indexing (12/40 (core))") >= 0, "o detalhe leva a mensagem: " + status.detail());

        status.handleProgress("cpp", "indexing", "", -1, false);
        failures += check(status.summary() === "LSP … cpp · indexing, rust · Indexing 43%",
                          "duas frentes, sem porcentagem: " + status.summary());
        failures += check(status.summary().indexOf("undefined") < 0, "nada de undefined");

        status.handleProgress("cpp", "indexing", "", -1, true);
        status.handleProgress("rust", "Indexing", "", 100, true);
        failures += check(status.summary() === "LSP ● 2", "fim: volta ao resumo: " + status.summary());

        status.handleProgress("rust", "cargo check", "", -1, false);
        status.handleStatus("rust", "exited", "caiu");
        failures += check(status.progress.rust === undefined, "o servidor que cai leva a frente");
        failures += check(status.summary() === "LSP ✗ rust", "a falha vence: " + status.summary());

        status.handleProgress("cpp", "indexing", "", 10, false);
        status.workspaceRoot = "/tmp/outro";
        failures += check(Object.keys(status.progress).length === 0 && status.summary() === "",
                          "outro projeto: esquece");
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
