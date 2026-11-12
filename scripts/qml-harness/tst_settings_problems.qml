import QtQuick
import KineinVectis

// O settings.json que a IDE nao entende (INF-07, 0.220.0, 2026-11-12): o
// SettingsEventRouter, o SettingsController, o ProjectHealthController, o
// ProjectHealthActions e o SettingsDialog REAIS, com um cliente falso que
// emite o que o CoreClient emite do `settings.get`/`settings.set`: o mapa
// `problems` com o motivo de cada escopo, e o `requestFailed` do
// `settings.set` recusado. Antes o arquivo valia como vazio, calado, e a
// falha do `settings.set` sumia.
//
// O que se prova: o motivo do projeto vai para a faixa, depois dos avisos de
// build e antes dos informativos, com o botao que abre as Configuracoes; o
// dialogo mostra o de cada escopo e a gravacao recusada; a resposta boa
// seguinte limpa a recusa; o cliente sem o mapa (antes do 0.220.0) nao deixa
// problema; dispensar esconde a faixa daquele motivo.
Item {
    id: root
    width: 900
    height: 600
    property int failures: 0

    QtObject {
        id: bridge
        signal settingsResolved(var effective, var global, var workspace, var problems)
        signal requestFailed(string method, string message, string code)
    }
    SettingsController {
        id: settings
    }
    SettingsEventRouter {
        coreClient: bridge
        settingsController: settings
    }
    ProjectHealthController {
        id: health
        workspaceRoot: "/tmp/proj"
        workspaceKind: "unknown"
        projectRecognized: true
        settingsProblem: settings.workspaceProblem
    }
    ProjectHealthActions {
        id: actions
        settingsController: settings
    }
    SettingsDialog {
        id: dialog
        anchors.fill: parent
        visible: settings.dialogVisible
        problems: settings.problems
    }

    readonly property string projectProblem: ".kinein/settings.json não tem schemaVersion"
    readonly property string globalProblem: "/home/p/.config/kinein-vectis/settings.json tem schemaVersion 2, "
                                            + "e esta versão da IDE só conhece a 1"
    readonly property string refusal: "As configurações não foram gravadas: " + root.projectProblem
                                      + ". A IDE não grava por cima do que não entende; corrija ou remova "
                                      + "esse caminho e tente de novo."

    function find(item, name) {
        if (item.objectName === name) return item;
        for (const child of item.children) {
            const result = find(child, name);
            if (result) return result;
        }
        return null;
    }
    function check(ok, message) {
        if (!ok) { root.failures++; console.error("FALHOU: " + message); }
    }
    function resolve(problems) {
        bridge.settingsResolved({ editorFontSize: 14, autoSave: true }, {}, {}, problems);
    }

    Component.onCompleted: {
        const verdict = find(dialog, "settingsProblems");
        check(verdict !== null, "o veredito do dialogo existe");
        check(health.status === "ok" && !health.active, "sem problema, a faixa fica quieta: " + health.status);

        // O motivo do projeto: a faixa avisa, com o botao das Configuracoes.
        root.resolve({ workspace: root.projectProblem });
        check(settings.workspaceProblem === root.projectProblem && settings.globalProblem === "",
              "o motivo do projeto no controller: " + settings.workspaceProblem);
        check(health.active && health.status === "warning", "a faixa avisa: " + health.status);
        check(health.message === "configurações do projeto não lidas: " + root.projectProblem,
              "o texto da faixa: " + health.message);
        check(health.actionLabel === "Configurações" && health.actionTarget === "settings",
              "o botao da faixa: " + health.actionLabel + " " + health.actionTarget);

        // O botao abre o dialogo, que mostra o motivo.
        actions.perform(health.actionTarget);
        check(settings.dialogVisible, "o botao abre as Configuracoes");
        check(verdict.visible && verdict.message === root.projectProblem, "o dialogo mostra o motivo: " + verdict.message);

        // A gravacao recusada fica no dialogo; a falha de outro metodo nao.
        bridge.requestFailed("fs.read", "outra coisa", "INTERNAL_ERROR");
        check(settings.setFailure === "", "a falha de outro metodo nao entra: " + settings.setFailure);
        bridge.requestFailed("settings.set", root.refusal, "INVALID_REQUEST");
        check(settings.setFailure === root.refusal, "a recusa no controller: " + settings.setFailure);
        check(verdict.message === root.projectProblem + "\n" + root.refusal, "o dialogo mostra a recusa: " + verdict.message);

        // A ordem da faixa: o ambiente Python que falta vem antes; os stubs,
        // que so' informam, depois.
        health.pythonNeedsEnvironment = true;
        health.pythonMessage = "falta o ambiente";
        health.pythonActionLabel = "Criar";
        check(health.actionTarget === "pythonEnvironment", "o ambiente Python vem antes: " + health.actionTarget);
        health.pythonNeedsEnvironment = false;
        health.pythonNeedsStubs = true;
        health.pythonStubsMessage = "faltam os stubs";
        check(health.actionTarget === "settings", "o motivo vem antes dos stubs: " + health.actionTarget);
        health.pythonNeedsStubs = false;

        // Dispensar esconde a faixa daquele motivo.
        health.dismiss();
        check(!health.active && health.actionTarget === "settings", "dispensado, a faixa some: " + health.active);

        // O motivo do global vai so' para o dialogo; a resposta boa limpa a
        // recusa, e o do projeto sai da faixa.
        root.resolve({ global: root.globalProblem });
        check(settings.globalProblem === root.globalProblem && settings.workspaceProblem === "",
              "o motivo do global: " + settings.globalProblem);
        check(settings.setFailure === "", "a resposta boa limpa a recusa: " + settings.setFailure);
        check(health.status === "ok" && !health.active, "sem o do projeto, a faixa fica quieta: " + health.status);
        check(verdict.message === root.globalProblem, "o dialogo mostra o do global: " + verdict.message);

        // Os dois escopos juntos, na ordem global e projeto.
        root.resolve({ global: root.globalProblem, workspace: root.projectProblem });
        check(verdict.message === root.globalProblem + "\n" + root.projectProblem, "os dois motivos: " + verdict.message);
        check(!health.active && health.actionTarget === "settings", "o mesmo motivo segue dispensado");
        root.resolve({ workspace: ".kinein/settings.json não é um objeto JSON" });
        check(health.active && health.message === "configurações do projeto não lidas: .kinein/settings.json não é um objeto JSON",
              "um motivo novo volta a faixa: " + health.message);

        // O cliente sem o mapa (o sinal de antes do 0.220.0): nenhum problema.
        bridge.settingsResolved({ editorFontSize: 14 }, {}, {}, undefined);
        check(settings.problems.length === 0 && !verdict.visible, "sem o mapa, nada a mostrar: " + settings.problems);
        check(health.status === "ok", "sem o mapa, a faixa fica quieta: " + health.status);

        Qt.exit(root.failures === 0 ? 0 : 1);
    }
}
