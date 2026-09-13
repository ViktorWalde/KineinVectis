pragma ComponentBehavior: Bound
import QtQuick

// O AMBIENTE Python do projeto (bloco B do roadmaps/41, fatia 1 em
// 2026-09-12). Guarda o que o core resolveu — interpretador, se e' ambiente
// proprio ou o Python do sistema, que ferramenta cria o .venv — e pede a
// criacao. NAO decide nada: a precedencia do interpretador e a escolha da
// ferramenta sao do core; a tela mostra, e o unico botao dela e' "criar".
Item {
    id: root

    property string workspaceRoot: ""
    // Os build systems do workspace: "python" entra tambem num projeto que e'
    // Cargo/CMake por fora e Python por dentro (o kind seria o outro).
    property var workspaceBuildSystems: []
    property var status: ({})
    property bool creating: false
    property string lastOutcome: ""

    readonly property bool isPython: buildsPython()
    readonly property bool known: status.hasEnvironment !== undefined
    readonly property bool hasEnvironment: status.hasEnvironment === true
    readonly property string environmentTool: status.environmentTool !== undefined
                                              && status.environmentTool !== null
                                              ? status.environmentTool : ""
    // Falta ambiente E a maquina consegue criar um: e' o caso do botao.
    readonly property bool needsEnvironment: isPython && known && !hasEnvironment
                                             && environmentTool !== ""

    signal statusRequested()
    signal createEnvironmentRequested(string tool)

    visible: false

    // Nos handlers, a funcao e nao a propriedade derivada: dentro de um
    // onXChanged, o binding de `isPython` pode ainda nao ter sido reavaliado.
    function buildsPython() {
        return workspaceBuildSystems !== undefined && workspaceBuildSystems !== null
                && workspaceBuildSystems.indexOf("python") >= 0;
    }

    onWorkspaceRootChanged: {
        status = ({});
        creating = false;
        lastOutcome = "";
        if (workspaceRoot !== "" && buildsPython()) {
            statusRequested();
        }
    }
    onWorkspaceBuildSystemsChanged: {
        if (workspaceRoot !== "" && buildsPython() && !known) {
            statusRequested();
        }
    }

    function handleStatus(newStatus) {
        status = newStatus === undefined || newStatus === null ? ({}) : newStatus;
    }

    function createEnvironment() {
        if (!needsEnvironment || creating) return;
        creating = true;
        lastOutcome = "";
        createEnvironmentRequested(environmentTool);
    }

    function handleEnvironmentFinished(outcome) {
        creating = false;
        if (outcome === undefined || outcome === null) return;
        lastOutcome = outcome.success === true
                ? qsTr("ambiente criado em %1 (%2)").arg(outcome.path).arg(outcome.command)
                : qsTr("falhou: %1 — veja o job").arg(outcome.command);
        statusRequested();
    }

    // Uma linha: "python: .venv · Python 3.14.7" / "python: sistema (3.14.7) ⚠"
    // / "python: nenhum ⚠" — e, num projeto com extensao nativa, " · pybind11
    // (scikit-build-core)" (fatia 5 da cadeia Python, 2026-09-13).
    function summary() {
        if (!isPython || !known) return "";
        const i = status.interpreter;
        if (i === undefined || i === null) return qsTr("python: nenhum ⚠");
        const versao = i.version !== undefined ? " · " + String(i.version).replace(/^Python /, "") : "";
        const base = hasEnvironment ? qsTr("python: %1%2").arg(i.origin).arg(versao)
                                    : qsTr("python: sistema%1 ⚠").arg(versao);
        const nativo = nativeModuleLine();
        return nativo === "" ? base : base + " · " + nativo;
    }

    // "pybind11 (scikit-build-core)" / "PyO3 (maturin)"; vazio sem modulo nativo.
    function nativeModuleLine() {
        const m = status.nativeModule;
        if (m === undefined || m === null) return "";
        return qsTr("%1 (%2)").arg(m.kind).arg(m.tool);
    }

    // O comando oficial que compila a extensao no ambiente — o core o escreve.
    function nativeModuleBuildHint() {
        const m = status.nativeModule;
        return m === undefined || m === null ? "" : String(m.buildHint);
    }

    // O rotulo do botao diz a FERRAMENTA e a pasta: nada de "configurar".
    function actionLabel() {
        if (environmentTool === "uv") return qsTr("Criar .venv com uv");
        if (environmentTool === "venv") return qsTr("Criar .venv (python3 -m venv)");
        return "";
    }

    function bannerMessage() {
        if (creating) return qsTr("criando o ambiente Python…");
        const h = status.hint;
        return h !== undefined && h !== null ? h : qsTr("Python sem ambiente próprio");
    }
}
