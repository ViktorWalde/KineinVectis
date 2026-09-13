// O ambiente Python na UI (bloco B do roadmaps/41, fatia 1 em 2026-09-12): a
// traducao do `python.status` em resumo, faixa e botao — e a regra de que o
// controller so' pergunta em projeto Python e so' pede criar quando falta
// ambiente e ha' ferramenta.
//
// Por que existe: "precisa de ambiente" e' uma conjuncao de tres coisas que o
// core manda separadas; errar uma delas mostra o botao a quem nao precisa ou
// esconde de quem precisa, e nenhum compilador reclama.
import QtQuick
import "../../ui/qml/python"

Item {
    id: root

    property int pedidosStatus: 0
    property var pedidosCriar: []

    PythonController {
        id: py

        onStatusRequested: root.pedidosStatus += 1
        onCreateEnvironmentRequested: function(tool) { root.pedidosCriar.push(tool); }
    }

    Component.onCompleted: {
        let failures = 0;

        // Projeto sem Python: nao pergunta, nao resume, nao precisa de nada.
        py.workspaceBuildSystems = ["cmake"];
        py.workspaceRoot = "/tmp/proj";
        if (root.pedidosStatus !== 0 || py.isPython || py.summary() !== "") failures += 1;

        // Projeto Python (ainda que Cargo por fora): pergunta ao abrir.
        py.workspaceBuildSystems = ["cargo", "python"];
        if (root.pedidosStatus !== 1 || !py.isPython) failures += 2;
        // Antes da resposta: nada a dizer, e o botao NAO aparece.
        if (py.known || py.needsEnvironment || py.summary() !== "") failures += 4;

        // Python do sistema + uv na maquina: precisa de ambiente, o botao diz a
        // ferramenta, o resumo avisa.
        py.handleStatus({ interpreter: { interpreter: "/usr/bin/python3", version: "Python 3.14.7", origin: "sistema", warning: "quebra a distro" },
                          hasEnvironment: false, environmentTool: "uv", uv: "/usr/bin/uv",
                          projectFiles: ["pyproject.toml"], hint: "o projeto usa o Python do SISTEMA: crie um ambiente proprio (.venv) com o uv" });
        if (!py.needsEnvironment) failures += 8;
        if (py.actionLabel() !== "Criar .venv com uv") failures += 16;
        if (py.summary() !== "python: sistema · 3.14.7 ⚠") failures += 32;
        if (py.bannerMessage().indexOf("SISTEMA") < 0) failures += 64;
        // So' python3: o botao diz venv.
        py.handleStatus({ interpreter: { interpreter: "/usr/bin/python3", origin: "sistema" },
                          hasEnvironment: false, environmentTool: "venv" });
        if (py.actionLabel() !== "Criar .venv (python3 -m venv)") failures += 128;
        // Sem ferramenta nenhuma: precisa, mas NAO ha' botao — nada a pedir.
        py.handleStatus({ hasEnvironment: false, hint: "nenhum Python encontrado" });
        if (py.needsEnvironment || py.actionLabel() !== "" || py.summary() !== "python: nenhum ⚠") failures += 256;

        // Criar: pede UMA vez com a ferramenta do status; enquanto cria, nao
        // pede de novo; ao terminar com sucesso, pergunta o status de novo e
        // registra o resultado.
        py.handleStatus({ interpreter: { interpreter: "/usr/bin/python3", origin: "sistema" },
                          hasEnvironment: false, environmentTool: "uv" });
        py.createEnvironment();
        py.createEnvironment();
        if (root.pedidosCriar.join(",") !== "uv" || !py.creating) failures += 512;
        if (py.bannerMessage().indexOf("criando") < 0) failures += 1024;
        const antes = root.pedidosStatus;
        py.handleEnvironmentFinished({ success: true, tool: "uv", command: "uv venv .venv", path: "/tmp/proj/.venv" });
        if (py.creating || root.pedidosStatus !== antes + 1) failures += 2048;
        if (py.lastOutcome.indexOf("/tmp/proj/.venv") < 0) failures += 4096;
        // Ambiente proprio: sem botao, resumo limpo.
        py.handleStatus({ interpreter: { interpreter: "/tmp/proj/.venv/bin/python", version: "Python 3.14.7", origin: ".venv" },
                          hasEnvironment: true, environmentTool: "uv" });
        if (py.needsEnvironment || py.summary() !== "python: .venv · 3.14.7") failures += 8192;
        // Falha: diz que falhou e aponta o job; volta a permitir criar.
        py.handleStatus({ interpreter: { origin: "sistema" }, hasEnvironment: false, environmentTool: "uv" });
        py.createEnvironment();
        py.handleEnvironmentFinished({ success: false, tool: "uv", command: "uv venv .venv", path: "/tmp/proj/.venv" });
        if (py.creating || py.lastOutcome.indexOf("falhou") < 0) failures += 16384;
        // Com ambiente, criar e' recusado (nao pede).
        py.handleStatus({ interpreter: { origin: ".venv" }, hasEnvironment: true, environmentTool: "uv" });
        const pedidos = root.pedidosCriar.length;
        py.createEnvironment();
        if (root.pedidosCriar.length !== pedidos) failures += 32768;

        // Modulo nativo (fatia 5): entra no resumo e a dica e' a do core.
        py.handleStatus({ interpreter: { interpreter: "/tmp/proj/.venv/bin/python", version: "Python 3.14.7", origin: ".venv" },
                          hasEnvironment: true, environmentTool: "uv",
                          nativeModule: { kind: "pybind11", tool: "scikit-build-core", evidence: ["CMakeLists.txt: pybind11"], buildHint: "pip install -e . (o scikit-build-core chama o CMake)" } });
        if (py.summary() !== "python: .venv · 3.14.7 · pybind11 (scikit-build-core)") failures += 131072;
        if (py.nativeModuleBuildHint().indexOf("pip install -e .") !== 0) failures += 262144;
        py.handleStatus({ interpreter: { origin: ".venv" }, hasEnvironment: true, environmentTool: "uv" });
        if (py.nativeModuleLine() !== "" || py.summary() !== "python: .venv") failures += 524288;

        // Trocar de workspace esquece tudo.
        py.workspaceRoot = "/tmp/outro";
        if (py.known || py.creating || py.lastOutcome !== "") failures += 65536;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
