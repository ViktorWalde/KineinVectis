import QtQuick
import "../../ui/qml/remote"

// Descoberta e explicacao do SSH que a maquina JA' tem (0.132.0, fatia R0.5):
// o RemoteController REAL com o roteador falso.
//
// O que se prova: descobrir pede uma vez e nao repete enquanto carrega; o
// estado de apresentacao segue o que o core respondeu, nunca um palpite;
// escolher um alias NAO copia usuario/porta/chave — o criterio de aceite da
// R0.5; resposta atrasada de outro host nao troca o resumo em tela; a falha
// de cada metodo cai no seu proprio estado, sem zerar o resto do painel.
Item {
    id: root

    property var pedidos: []
    property int failures: 0

    function check(ok, mensagem) {
        if (!ok) {
            failures++;
            console.error(mensagem);
        }
    }

    RemoteController {
        id: c
    }

    // O setup e' filho da fachada (roadmap 48 §8.3): o harness fala com ele
    // ATRAVES da composicao real, nao com uma copia solta.
    Connections {
        target: c.setup

        function onDiscoverRequested() { root.pedidos.push("discover"); }
        function onResolveRequested(host) { root.pedidos.push("resolve:" + host); }
        function onParseRequested(command) { root.pedidos.push("parse:" + command); }
    }

    Component.onCompleted: {
        check(c.setup.discovery === "idle" && c.setup.aliases.length === 0,
              "sem pedir nada, a IDE nao pode afirmar nada sobre a maquina");

        c.setup.discover();
        check(c.setup.discovery === "loading" && c.setup.discovering && pedidos.length === 1,
              "descobrir pede uma vez e anuncia que esta' carregando");
        c.setup.discover();
        check(pedidos.length === 1, "pedido em curso nao duplica");

        c.setup.handleAliases([{ name: "pi", source: "~/.ssh/config" },
                         { name: "bancada", source: "~/.ssh/config.d/10-lab.conf" }],
                        ["~/.ssh/config", "~/.ssh/config.d/10-lab.conf"]);
        check(c.setup.discovery === "ready" && !c.setup.discovering && c.setup.aliases.length === 2,
              "a lista vem do core e o carregamento termina");
        check(c.setup.aliases[1].source === "~/.ssh/config.d/10-lab.conf",
              "a origem de cada alias tem de ser visivel");
        check(c.setup.aliasSources.length === 2, "os arquivos lidos sao ditos");

        // O CRITERIO DE ACEITE: se `ssh pi` ja' funciona, o alvo nasce sem
        // usuario, porta nem chave — quem decide continua sendo o OpenSSH.
        c.useAlias("  pi  ");
        check(c.draft.name === "pi" && c.draft.host === "pi", "o alias vira nome e host");
        check(c.draft.user === "" && c.draft.identityFile === "" && c.draft.deployDir === "",
              "escolher alias nao pode redigitar o que o ~/.ssh/config ja' diz");
        check(c.draft.port === 0, "porta 0 e' descartada pela ponte: o perfil fica sem override");
        c.useAlias("   ");
        check(c.draft.name === "pi", "alias vazio nao apaga o rascunho");

        c.setup.resolve("pi");
        check(c.setup.resolving === "pi" && pedidos[1] === "resolve:pi", "resolver pede pelo host");
        c.setup.resolve("pi");
        check(pedidos.length === 2, "o mesmo host em curso nao duplica");
        c.setup.resolve("");
        check(pedidos.length === 2, "host vazio nao vira pedido");

        // Resposta de outro host nao pode virar o resumo deste.
        c.setup.handleResolved({ host: "bancada", user: "outro", port: 22 });
        check(c.setup.resolved === null && c.setup.resolving === "pi", "resumo de outro host foi aceito");
        c.setup.handleResolved({ host: "pi", user: "pi", hostName: "192.168.0.42",
                           port: 2222, identities: ["~/.ssh/pi"], proxyCommand: true });
        check(c.setup.resolving === "" && c.setup.resolved.hostName === "192.168.0.42", "o resumo chega");
        check(c.setup.resolved.proxyCommand === true && c.setup.resolved.proxyJump === undefined,
              "existe proxy, e o texto do comando nao vem do core");

        // Cada falha no seu estado: descobrir falhar nao pode dizer que a
        // sonda parou, nem o contrario.
        c.probing = true;
        c.handleFailed("remote.discover", "sem HOME");
        check(c.setup.discovery === "failed" && c.errorText === "sem HOME", "falha da descoberta");
        check(c.probing === true, "falha da descoberta nao mexe na sonda");

        c.setup.resolve("bancada");
        c.handleFailed("remote.resolve", "o `ssh` nao aceitou `bancada`");
        check(c.setup.resolving === "" && c.setup.discovery === "failed", "falha ao resolver libera o gesto");

        c.handleFailed("remote.probe", "sem chave");
        check(c.probing === false, "falha da sonda continua zerando a sonda");

        // "Configurar servidor": a linha colada vira RASCUNHO, nunca alvo salvo.
        const antesDoParse = pedidos.length;
        check(!c.setup.canParse, "sem texto, nao ha' o que interpretar");
        c.setup.pasted = "   ";
        check(!c.setup.canParse, "so' espaco nao e' comando");
        c.setup.parse();
        check(pedidos.length === antesDoParse, "espaco em branco nao vira pedido");

        c.setup.pasted = "  ssh -p 2222 pi@192.168.0.42  ";
        check(c.setup.canParse, "com texto, da' para interpretar");
        c.setup.parse();
        check(pedidos[antesDoParse] === "parse:ssh -p 2222 pi@192.168.0.42",
              "a linha vai aparada, sem o que a pessoa colou por acidente");

        // Quem le' e' o core; o que volta e' proposta.
        c.setup.handleParsed({
            target: { name: "192.168.0.42", host: "192.168.0.42", user: "pi", port: 2222 },
            source: ["porta do `-p` da linha colada", "host `192.168.0.42` da linha colada"]
        });
        check(c.draft.host === "192.168.0.42" && c.draft.user === "pi" && c.draft.port === 2222,
              "a proposta preenche o rascunho");
        check(c.setup.proposalSource.length === 2, "a procedencia fica na tela");
        check(c.targets.length === 0, "LER NAO E' GRAVAR: nada foi salvo");

        // Proposta sem host nao pode apagar o que a pessoa ja' tinha digitado.
        c.setup.handleParsed({ target: { name: "x" }, source: [] });
        check(c.draft.host === "192.168.0.42", "proposta incompleta nao destroi o rascunho");

        // A recusa do core limpa a procedencia: ela era de outra leitura.
        c.handleFailed("remote.parseCommand", "a linha tem `;`");
        check(c.setup.proposalSource.length === 0 && c.errorText === "a linha tem `;`",
              "a recusa aparece e a procedencia antiga sai");

        // A lista e' da MAQUINA, nao do projeto: trocar de workspace nao a perde.
        c.workspaceRoot = "/tmp/outro";
        check(c.setup.aliases.length === 2, "trocar de projeto nao apaga o que a maquina tem");

        Qt.exit(failures === 0 ? 0 : 1);
    }
}
