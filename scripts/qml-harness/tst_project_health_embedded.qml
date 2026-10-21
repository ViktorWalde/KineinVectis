// O reconhecimento de firmware nao depende de um sistema de build.
import QtQuick
import "../../ui/qml/workspace"
import "../../ui/qml/embedded"

Item {
    property int failures: 0
    EmbeddedController {
        id: embedded
        workspaceRoot: "/tmp/firmware"
    }
    ProjectHealthController {
        id: health
        workspaceRoot: embedded.workspaceRoot
        workspaceKind: "unknown"
        projectRecognized: embedded.projectEmbedded || embedded.projectLinux
    }
    function check(condition, description) {
        if (!condition) {
            console.error("FALHAS: " + description);
            failures += 1;
        }
    }
    Component.onCompleted: {
        embedded.handleProject({embedded: true, frameworks: [{framework: "microPython"}]});
        check(!health.active && health.status === "ok", "MicroPython reconhecido nao perde run");
        health.pythonNeedsStubs = true;
        health.pythonStubsMessage = "faltam stubs da placa";
        check(health.actionTarget === "pythonStubs", "sem build system ainda mostra stubs");
        health.pythonNeedsEnvironment = true;
        health.pythonMessage = "crie ambiente";
        check(health.actionTarget === "pythonEnvironment", "ambiente precede stubs");
        health.pythonCreating = true;
        check(health.actionTarget === "jobs", "criacao continua acionavel");
        health.pythonCreating = false;
        health.pythonNeedsEnvironment = false;
        health.pythonNeedsStubs = false;
        embedded.workspaceRoot = "/tmp/empty";
        check(!health.projectRecognized && health.active
              && health.message.indexOf("continuam disponíveis") >= 0, "troca remove reconhecimento e preserva comandos");
        embedded.handleProject({embedded: false, target: {family: "linux"}});
        check(!health.active, "Linux embarcado reconhecido sem build system");
        embedded.handleProject(null);
        check(health.active, "modelo removido nao deixa reconhecimento antigo");
        health.workspaceKind = "python";
        health.workspaceBuildSystems = ["python"];
        health.pythonNeedsEnvironment = true;
        check(health.actionTarget === "pythonEnvironment", "Python isolado nao ignora ambiente");
        health.pythonNeedsEnvironment = false;
        health.pythonNeedsStubs = true;
        check(health.actionTarget === "pythonStubs", "Python isolado nao ignora stubs");
        health.scanningEnvironment = true;
        check(health.status === "busy", "varredura precede aviso");
        health.workspaceRoot = "";
        check(!health.active && health.status === "idle", "sem workspace nao ha aviso");
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
