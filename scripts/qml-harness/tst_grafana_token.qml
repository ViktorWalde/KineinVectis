// CICLO DE VIDA DO TOKEN DA SESSAO — bloqueador de entrega (§7.2 regra 9 da
// `especificacoes/grafana-ui-ux-0.3.5.md`).
//
// A §7 mudou a fronteira de proposito em 2026-09-22: fechar a tool window
// DEIXA de apagar a credencial, porque abrir e fechar um painel virava um
// login novo a cada vez. Em troca, tudo que significa "outra instancia ou
// outro projeto" passa a apagar — e isso so' vale se estiver medido.
//
// A tabela que este teste cobra, linha por linha:
//
//   fechar/reabrir painel       mantem
//   trocar/fechar workspace     apaga
//   confirmar outra URL         apaga ANTES de falar com a URL nova
//   esquecer credencial         apaga na hora
//   esquecer instancia          apaga antes de remover o perfil
//   token rejeitado             apaga e pede outro
//
// Mais o vinculo de custodia (§7.1): so' uma requisicao para o MESMO par
// workspace + URL confirmada reutiliza o token, e resposta atrasada da URL
// antiga e' descartada por identidade.
//
// MUTACOES QUE PROVAM O GATE: devolva `clearToken()` ao `close()` e a primeira
// assercao cai; tire o `clearToken()` do `save()` com URL diferente e a
// credencial antiga viaja para a instancia nova.
import QtQuick
import KineinVectis

Item {
    id: root

    property int falhas: 0

    // O que o controller PEDIU ao core, na ordem. E' aqui que se ve se o token
    // viajou: o painel nunca ve a string, e o core so' recebe o que vem por
    // estes sinais.
    property var tokensEnviados: []
    property var perfisSalvos: []

    GrafanaController {
        id: controlador

        workspaceRoot: "/tmp/projeto"
        panelVisible: true

        onProbeRequested: token => root.tokensEnviados.push(token)
        onSaveRequested: perfil => root.perfisSalvos.push(perfil)
    }

    function conferir(condicao, mensagem) {
        if (!condicao) {
            console.error("FALHOU: " + mensagem);
            root.falhas += 1;
        }
    }

    function ultimoToken() {
        return root.tokensEnviados.length === 0
             ? "(nenhum pedido)"
             : root.tokensEnviados[root.tokensEnviados.length - 1];
    }

    // Deixa o controller com instancia salva em `url` e token de sessao aceito.
    function autenticar(url, token) {
        controlador.handleProfile({ url: url, tokenSource: "prompt" }, true);
        controlador.probeWithToken(token);
        controlador.handleProbed({ reachable: true, authenticated: true, version: "11.2.0" });
    }

    Component.onCompleted: {
        root.autenticar("http://grafana.lab:3000", "token-bom");
        root.conferir(controlador.hasSessionToken, "o token nao ficou na sessao");
        root.conferir(root.ultimoToken() === "token-bom",
                      "o token nao chegou ao core na sonda que o pediu");

        // FECHAR E REABRIR MANTEM. Era isto que fazia do painel um login novo.
        controlador.close();
        controlador.open();
        root.conferir(controlador.hasSessionToken,
                      "fechar o painel apagou a credencial da sessao");

        // MESMO PAR => o token viaja de novo, sem perguntar nada.
        controlador.probe();
        root.conferir(root.ultimoToken() === "token-bom",
                      "o mesmo par workspace+URL nao reaproveitou o token");

        // EDITAR O RASCUNHO NAO MUDA O CONTEXTO nem manda o token embora: a
        // intencao ainda nao foi confirmada.
        controlador.setDraftField("url", "http://outro.lab:3000");
        controlador.probe();
        root.conferir(root.ultimoToken() === "token-bom",
                      "editar o rascunho ja' mudou o dono do token");

        // CONFIRMAR A OUTRA URL APAGA — antes de qualquer conversa com ela.
        controlador.save();
        root.conferir(!controlador.hasSessionToken,
                      "confirmar outra instancia manteve o token da anterior");
        controlador.handleProfile({ url: "http://outro.lab:3000", tokenSource: "prompt" }, true);
        controlador.probe();
        root.conferir(root.ultimoToken() === "",
                      "a credencial da instancia antiga viajou para a nova: "
                      + root.ultimoToken());

        // O TOKEN NUNCA ENTRA NO PERFIL, que e' o que o core persiste.
        let vazou = false;
        for (let indice = 0; indice < root.perfisSalvos.length; ++indice) {
            const perfil = root.perfisSalvos[indice];
            for (const chave in perfil) {
                if (String(perfil[chave]).indexOf("token-bom") >= 0) {
                    vazou = true;
                }
            }
        }
        root.conferir(!vazou, "o token apareceu no perfil enviado ao core");

        // TOKEN REJEITADO: apaga e pede outro.
        root.autenticar("http://outro.lab:3000", "token-ruim");
        controlador.probe();
        controlador.handleFailed("grafana.probe", "nao autorizado", "SECRET_REQUIRED");
        root.conferir(!controlador.hasSessionToken,
                      "o token recusado continuou em memoria");
        root.conferir(controlador.tokenRequired && controlador.authFailed,
                      "recusa nao pediu outro token, ou nao se distinguiu de um pedido");

        // ESQUECER CREDENCIAL: na hora, e o resultado obtido com ela para de
        // valer como prova do que a IDE ve agora.
        root.autenticar("http://outro.lab:3000", "token-bom-2");
        controlador.forgetCredential();
        root.conferir(!controlador.hasSessionToken, "esquecer credencial nao apagou");
        root.conferir(!controlador.authenticated,
                      "o resultado autenticado sobreviveu a credencial que o obteve");

        // ESQUECER A INSTANCIA leva a credencial junto.
        root.autenticar("http://outro.lab:3000", "token-bom-3");
        controlador.forget();
        root.conferir(!controlador.hasSessionToken, "esquecer a instancia manteve o token");

        // TROCAR DE WORKSPACE apaga.
        root.autenticar("http://outro.lab:3000", "token-bom-4");
        controlador.workspaceRoot = "/tmp/outro-projeto";
        root.conferir(!controlador.hasSessionToken, "trocar de workspace manteve o token");

        // O PERFIL CONFIRMADO PODE MUDAR SEM PASSAR PELO `save` DAQUI: o core
        // responde `get` quando quer, e pode devolver a URL normalizada ou o
        // perfil gravado por outra janela. Nesse caso o token continua em
        // memoria, mas o par a que ele pertence deixou de ser o atual — e a
        // conferencia acontece na hora de por a credencial no fio, que e' o
        // unico lugar onde errar custa vazamento.
        root.autenticar("http://c.lab:3000", "token-de-c");
        controlador.handleProfile({ url: "http://d.lab:3000", tokenSource: "prompt" }, true);
        controlador.probe();
        root.conferir(root.ultimoToken() === "",
                      "o token de outra instancia foi para o fio: " + root.ultimoToken());

        // RESPOSTA ATRASADA DA URL ANTIGA e' descartada por identidade: entre
        // pedir e responder, o autor confirmou outra instancia.
        controlador.handleProfile({ url: "http://a.lab:3000", tokenSource: "none" }, true);
        controlador.probe();
        controlador.handleProfile({ url: "http://b.lab:3000", tokenSource: "none" }, true);
        controlador.handleProbed({ reachable: true, authenticated: true, version: "9.9.9" });
        // O QUE SE MEDE E' A RESPOSTA NAO TER ENTRADO — e nao a tela estar
        // vazia. Desde que a §6 passou a exigir "ultimo resultado como
        // DESATUALIZADO", pedir de novo nao apaga o que ja' estava ali.
        root.conferir(controlador.version !== "9.9.9",
                      "a resposta da instancia antiga foi aceita como medida da nova");
        root.conferir(controlador.panelState.content === "stale",
                      "o resultado da instancia anterior nao foi marcado como velho: "
                      + controlador.panelState.content);

        // TOKEN RECUSADO PELA SONDA (protocolo 0.136.0). Este caminho nao e'
        // o do `SECRET_REQUIRED`: o servidor RESPONDEU, e disse que a
        // credencial nao serve. Achado contra um Grafana real em 2026-09-26 —
        // antes disso a tela lia "sem autenticacao" e nao oferecia saida.
        controlador.workspaceRoot = "/tmp/quarto-projeto";
        controlador.handleProfile({ url: "http://f.lab:3000", tokenSource: "prompt" }, true);
        controlador.probeWithToken("token-que-o-servidor-nega");
        controlador.handleProbed({
            reachable: true, authenticated: false, authRefused: true,
            version: "11.2.0", message: "recusou o token"
        });
        root.conferir(!controlador.hasSessionToken,
                      "o token recusado pela sonda ficou em memoria");
        root.conferir(controlador.panelState.auth === "failed",
                      "recusa na sonda nao virou estado de falha: "
                      + controlador.panelState.auth);
        root.conferir(controlador.primaryAction.kind === "provideToken",
                      "sem caminho de volta depois da recusa: "
                      + controlador.primaryAction.kind);

        // ALCANCAR SEM OFERECER NADA nao e' recusa, e nao pode pedir token.
        controlador.workspaceRoot = "/tmp/quinto-projeto";
        controlador.handleProfile({ url: "http://g.lab:3000", tokenSource: "none" }, true);
        controlador.probe();
        controlador.handleProbed({ reachable: true, authenticated: false, version: "11.2.0" });
        root.conferir(controlador.panelState.auth === "not_required",
                      "sem token virou recusa: " + controlador.panelState.auth);

        // PEDIR TOKEN NAO E' TER MEDIDO. O core recusa `SECRET_REQUIRED` ao
        // resolver a POLITICA, antes de falar com o Grafana: carimbar hora
        // aqui faria a tela dizer "medido agora" sobre uma instancia com a
        // qual ninguem falou.
        controlador.workspaceRoot = "/tmp/terceiro-projeto";
        controlador.handleProfile({ url: "http://e.lab:3000", tokenSource: "prompt" }, true);
        controlador.probe();
        controlador.handleFailed("grafana.probe", "token exigido", "SECRET_REQUIRED");
        root.conferir(controlador.probedAt === 0,
                      "o pedido de token virou medida: probedAt=" + controlador.probedAt);
        root.conferir(controlador.panelState.probe === "unknown",
                      "pedir token contou como sonda que falhou: " + controlador.panelState.probe);

        Qt.exit(root.falhas === 0 ? 0 : 1);
    }
}
