import QtQuick
import "../../ui/qml/remote"

// O gesto para a chave recusada (0.133.0): o `RemoteController` REAL com o
// roteador falso.
//
// A propriedade que este harness existe para travar: **copiar chave nunca
// acontece em silencio**. Compor a linha nao roda nada; so' um gesto explicito
// manda ao terminal, e a linha esteve visivel antes. Prova tambem que o gesto
// so' e' oferecido para a falha que ele resolve, e que a linha armada nao
// sobrevive a troca de alvo.
Item {
    id: root

    property var pedidos: []
    property var terminal: []
    property int failures: 0

    function check(ok, mensagem) {
        if (!ok) {
            failures++;
            console.error(mensagem);
        }
    }

    RemoteController {
        id: c

        onCommandRequested: function(name, kind, program, port) {
            root.pedidos.push("command:" + name + ":" + kind);
        }
        onShellRequested: function(command) { root.terminal.push(command); }
        onListRequested: root.pedidos.push("list")
    }

    function sondaFalhou(tipo) {
        c.handleProbed({ name: "pi", success: false, failure: tipo,
                         error: "o alvo recusou a chave" });
    }

    Component.onCompleted: {
        c.workspaceRoot = "/w";
        c.handleTargets([{ name: "pi", host: "192.168.0.42", user: "pi", port: 2222 }]);
        c.select("pi");
        check(c.selectedSaved, "alvo salvo e selecionado");
        check(!c.canCopyId, "sem sonda, nao ha' gesto a oferecer");

        // So' a falha que o `ssh-copy-id` RESOLVE oferece o gesto.
        for (const tipo of ["host", "network", "other"]) {
            sondaFalhou(tipo);
            check(c.probeFailure === tipo, "a falha tipada chega: " + tipo);
            check(!c.canCopyId, "copiar chave nao resolve `" + tipo + "`");
        }
        sondaFalhou("authentication");
        check(c.canCopyId, "chave recusada tem de oferecer o gesto");

        // Compor NAO roda.
        c.copyId();
        check(pedidos[pedidos.length - 1] === "command:pi:copyId", "pediu a linha ao core");
        check(terminal.length === 0, "compor a linha nao pode tocar o terminal");
        check(c.armedCommand === "", "nada armado antes da resposta do core");

        // A resposta ARMA; continua sem rodar.
        c.handleCommand({ command: "ssh-copy-id -p 2222 pi@192.168.0.42",
                          name: "Copiar chave para pi" });
        check(c.armedCommand === "ssh-copy-id -p 2222 pi@192.168.0.42", "a linha ficou visivel");
        check(c.armedName === "Copiar chave para pi", "o nome do gesto acompanha");
        check(terminal.length === 0, "ARMAR nao e' RODAR");
        check(!c.canCopyId, "com a linha armada, o botao sai de cena");

        // Cancelar desarma sem rodar.
        c.disarm();
        check(c.armedCommand === "" && terminal.length === 0, "cancelar nao roda");

        // So' o gesto explicito manda ao terminal, e desarma.
        c.copyId();
        c.handleCommand({ command: "ssh-copy-id pi@192.168.0.42", name: "Copiar chave para pi" });
        c.runArmed();
        check(terminal.length === 1 && terminal[0] === "ssh-copy-id pi@192.168.0.42",
              "rodar manda exatamente a linha que esteve na tela");
        check(c.armedCommand === "", "rodar desarma");
        c.runArmed();
        check(terminal.length === 1, "sem linha armada, rodar nao faz nada");

        // Uma linha armada NAO pode sobreviver a troca de alvo: ela carrega um
        // host, e rodar no alvo errado copiaria a chave para outra maquina.
        c.handleTargets([{ name: "pi", host: "192.168.0.42", user: "pi", port: 2222 },
                         { name: "bancada", host: "10.0.0.7" }]);
        sondaFalhou("authentication");
        c.copyId();
        c.handleCommand({ command: "ssh-copy-id pi@192.168.0.42", name: "Copiar chave para pi" });
        check(c.armedCommand !== "", "armado no pi");
        c.select("bancada");
        check(c.armedCommand === "", "trocar de alvo desarma");
        c.runArmed();
        check(terminal.length === 1, "nada foi para o terminal depois da troca");

        Qt.exit(failures === 0 ? 0 : 1);
    }
}
