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

        onDiscoverRequested: root.pedidos.push("discover")
        onResolveRequested: function(host) { root.pedidos.push("resolve:" + host); }
    }

    Component.onCompleted: {
        check(c.discovery === "idle" && c.aliases.length === 0,
              "sem pedir nada, a IDE nao pode afirmar nada sobre a maquina");

        c.discover();
        check(c.discovery === "loading" && c.discovering && pedidos.length === 1,
              "descobrir pede uma vez e anuncia que esta' carregando");
        c.discover();
        check(pedidos.length === 1, "pedido em curso nao duplica");

        c.handleAliases([{ name: "pi", source: "~/.ssh/config" },
                         { name: "bancada", source: "~/.ssh/config.d/10-lab.conf" }],
                        ["~/.ssh/config", "~/.ssh/config.d/10-lab.conf"]);
        check(c.discovery === "ready" && !c.discovering && c.aliases.length === 2,
              "a lista vem do core e o carregamento termina");
        check(c.aliases[1].source === "~/.ssh/config.d/10-lab.conf",
              "a origem de cada alias tem de ser visivel");
        check(c.aliasSources.length === 2, "os arquivos lidos sao ditos");

        // O CRITERIO DE ACEITE: se `ssh pi` ja' funciona, o alvo nasce sem
        // usuario, porta nem chave — quem decide continua sendo o OpenSSH.
        c.useAlias("  pi  ");
        check(c.draft.name === "pi" && c.draft.host === "pi", "o alias vira nome e host");
        check(c.draft.user === "" && c.draft.identityFile === "" && c.draft.deployDir === "",
              "escolher alias nao pode redigitar o que o ~/.ssh/config ja' diz");
        check(c.draft.port === 0, "porta 0 e' descartada pela ponte: o perfil fica sem override");
        c.useAlias("   ");
        check(c.draft.name === "pi", "alias vazio nao apaga o rascunho");

        c.resolve("pi");
        check(c.resolving === "pi" && pedidos[1] === "resolve:pi", "resolver pede pelo host");
        c.resolve("pi");
        check(pedidos.length === 2, "o mesmo host em curso nao duplica");
        c.resolve("");
        check(pedidos.length === 2, "host vazio nao vira pedido");

        // Resposta de outro host nao pode virar o resumo deste.
        c.handleResolved({ host: "bancada", user: "outro", port: 22 });
        check(c.resolved === null && c.resolving === "pi", "resumo de outro host foi aceito");
        c.handleResolved({ host: "pi", user: "pi", hostName: "192.168.0.42",
                           port: 2222, identities: ["~/.ssh/pi"], proxyCommand: true });
        check(c.resolving === "" && c.resolved.hostName === "192.168.0.42", "o resumo chega");
        check(c.resolved.proxyCommand === true && c.resolved.proxyJump === undefined,
              "existe proxy, e o texto do comando nao vem do core");

        // Cada falha no seu estado: descobrir falhar nao pode dizer que a
        // sonda parou, nem o contrario.
        c.probing = true;
        c.handleFailed("remote.discover", "sem HOME");
        check(c.discovery === "failed" && c.errorText === "sem HOME", "falha da descoberta");
        check(c.probing === true, "falha da descoberta nao mexe na sonda");

        c.resolve("bancada");
        c.handleFailed("remote.resolve", "o `ssh` nao aceitou `bancada`");
        check(c.resolving === "" && c.discovery === "failed", "falha ao resolver libera o gesto");

        c.handleFailed("remote.probe", "sem chave");
        check(c.probing === false, "falha da sonda continua zerando a sonda");

        // A lista e' da MAQUINA, nao do projeto: trocar de workspace nao a perde.
        c.workspaceRoot = "/tmp/outro";
        check(c.aliases.length === 2, "trocar de projeto nao apaga o que a maquina tem");

        Qt.exit(failures === 0 ? 0 : 1);
    }
}
