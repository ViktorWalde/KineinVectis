import QtQuick
// Carrega o GrafanaStateRules REAL (arquivo do projeto, sem copia).
import "../../ui/qml/grafana"

// A MAQUINA DE ESTADOS DO GRAFANA (§6, fatia V7/G1).
//
// Quatro eixos ortogonais, e as regras que a especificacao escreve para que a
// tela nao minta. Cada uma delas e' uma frase que ja' apareceu quebrada em
// algum painel deste projeto — por isso viram teste antes de virar tela.
Item {
    id: root

    property int failures: 0

    GrafanaStateRules {
        id: rules
    }

    GrafanaActionRules {
        id: acoes
    }

    function check(ok, bit, message) {
        if (!ok) {
            root.failures += bit;
            console.error(message);
        }
    }

    // O fato "completo", para cada caso mudar so' o que importa.
    function fatos(extra) {
        const base = {
            "hasInstance": true, "editing": false, "probing": false,
            "probedAt": 0, "reachable": false, "authenticated": false,
            "tokenRequired": false, "authFailed": false,
            "dataSourceCount": 0, "dashboardCount": 0,
            "url": "http://a:3000", "resultUrl": "http://a:3000",
            "agora": 1000000
        };
        for (const k in extra) {
            base[k] = extra[k];
        }
        return base;
    }

    Component.onCompleted: {
        // SEM INSTANCIA: nada foi configurado, e nada foi medido.
        const zero = rules.stateFor(fatos({ "hasInstance": false }));
        check(zero.setup === "absent", 1, "sem instancia: " + zero.setup);
        check(zero.probe === "unknown", 2, "nunca sondado NAO e' 'falhou'");
        check(zero.content === "unknown", 4, "sem medida nao ha' conteudo");

        // NUNCA SONDADO com instancia salva continua "unknown" — configurar
        // nao e' medir.
        const salvo = rules.stateFor(fatos({}));
        check(salvo.setup === "saved" && salvo.probe === "unknown", 8,
              "configurado nao e' medido");

        // ALCANCAVEL NAO IMPLICA AUTENTICADO. E' a primeira regra da §6, e a
        // que mais aparece quebrada: um alvo de pe' que recusa a credencial.
        const dePe = rules.stateFor(fatos({ "probedAt": 999000, "reachable": true,
                                            "tokenRequired": true }));
        check(dePe.probe === "reachable", 16, "esta' de pe'");
        check(dePe.auth === "required", 32, "mas exige token: " + dePe.auth);
        check(dePe.auth !== "authenticated", 64, "de pe' nao e' autenticado");

        // TOKEN RECUSADO e' diferente de token exigido.
        const recusado = rules.stateFor(fatos({ "probedAt": 999000, "reachable": true,
                                                "tokenRequired": true, "authFailed": true }));
        check(recusado.auth === "failed", 128, "token recusado: " + recusado.auth);

        // TROCAR A URL INVALIDA O RESULTADO ANTERIOR. O que esta' na tela
        // pertence a outra instancia.
        const outraUrl = rules.stateFor(fatos({ "probedAt": 999000, "reachable": true,
                                                "dashboardCount": 5,
                                                "url": "http://b:3000" }));
        check(outraUrl.content === "stale", 256,
              "resultado de outra instancia: " + outraUrl.content);

        // FALHA NA ATUALIZACAO mantem o anterior como DESATUALIZADO, nunca
        // como resultado novo.
        const falhouComDado = rules.stateFor(fatos({ "probedAt": 999000, "reachable": false,
                                                     "dashboardCount": 5 }));
        check(falhouComDado.probe === "failed", 512, "a sonda falhou");
        check(falhouComDado.content === "stale", 1024,
              "o resultado anterior vira desatualizado: " + falhouComDado.content);

        // E falha SEM dado anterior nao inventa lista vazia: e' desconhecido.
        const falhouSemDado = rules.stateFor(fatos({ "probedAt": 999000, "reachable": false }));
        check(falhouSemDado.content === "unknown", 2048,
              "sem dado anterior: " + falhouSemDado.content);

        // ALCANCAVEL E VAZIO e' diferente de alcancavel e desconhecido.
        const vazio = rules.stateFor(fatos({ "probedAt": 999000, "reachable": true }));
        check(vazio.content === "empty", 4096, "alcancavel e vazio: " + vazio.content);

        const pronto = rules.stateFor(fatos({ "probedAt": 999000, "reachable": true,
                                              "dataSourceCount": 2, "dashboardCount": 3 }));
        check(pronto.content === "ready", 8192, "com conteudo: " + pronto.content);

        // SONDANDO e' um estado proprio nos dois eixos.
        const sondando = rules.stateFor(fatos({ "probing": true, "probedAt": 999000 }));
        check(sondando.probe === "probing" && sondando.content === "loading", 16384,
              "sondando: " + sondando.probe + "/" + sondando.content);

        // A IDADE: "medido agora" so' vale para medida recente desta sessao.
        check(rules.frescorFrase(fatos({ "probedAt": 0 })) === qsTr("não consultado nesta sessão"),
              32768, "nunca consultado diz isso");
        check(rules.frescorFrase(fatos({ "probedAt": 999000 })) === qsTr("medido agora"),
              65536, "medida de um segundo atras e' 'agora'");
        const velha = rules.frescorFrase(fatos({ "probedAt": 100000 }));
        check(velha.indexOf("há") >= 0, 131072, "medida velha carrega a idade: " + velha);

        // --- UMA ACAO PRIMARIA POR ESTADO -----------------------------------
        //
        // A tela de hoje mostra `Salvar · Sondar · Esquecer` com o mesmo peso.
        // A §3 nomeia o atrito: salvar antes de sondar e' exigencia do DESENHO.

        function primaria(extra) {
            const f = fatos(extra);
            return acoes.primaryFor(rules.stateFor(f), f);
        }

        // NENHUM ESTADO FICA SEM PROXIMO GESTO. A §4 exige que erro sempre
        // ofereca acao seguinte.
        const todos = [
            { "hasInstance": false },
            { "editing": true },
            { "probing": true },
            { "probedAt": 999000, "reachable": true },
            { "probedAt": 999000, "reachable": false },
            { "probedAt": 999000, "reachable": true, "tokenRequired": true },
            { "probedAt": 999000, "reachable": true, "tokenRequired": true, "authFailed": true },
            { "probedAt": 999000, "reachable": true, "dashboardCount": 3 },
            { "probedAt": 999000, "reachable": true, "dashboardCount": 3, "url": "http://b:3000" }
        ];
        for (const caso of todos) {
            const acao = primaria(caso);
            check(acao.kind !== undefined && acao.kind !== "", 262144,
                  "estado sem acao primaria: " + JSON.stringify(caso));
            check(acao.label !== "" && acao.hint !== "", 524288,
                  "acao sem rotulo ou sem motivo: " + acao.kind);
            check(acao.section !== "", 1048576, "acao sem secao: " + acao.kind);
        }

        // SEM INSTANCIA o gesto e' Conectar, e nao Salvar: testar vem antes de
        // virar trabalho diario.
        check(primaria({ "hasInstance": false }).kind === "connect", 2097152,
              "sem instancia: " + primaria({ "hasInstance": false }).kind);

        // TOKEN EXIGIDO vence a atualizacao: nao adianta oferecer Atualizar a
        // quem o servidor acabou de recusar.
        const exige = primaria({ "probedAt": 999000, "reachable": true, "tokenRequired": true });
        check(exige.kind === "provideToken", 4194304, "exige token: " + exige.kind);
        check(exige.section === "auth", 8388608, "e leva ate' a secao da credencial");

        // RECUSADO diz outra coisa de EXIGIDO — a pessoa ja' tentou uma vez.
        const recusou = primaria({ "probedAt": 999000, "reachable": true,
                                   "tokenRequired": true, "authFailed": true });
        check(recusou.label !== exige.label, 16777216,
              "recusado e exigido dizem a mesma coisa: " + recusou.label);

        // ENDERECO QUE NAO RESPONDE leva de volta ao endereco, e nao a um
        // "tentar de novo" cego.
        check(primaria({ "probedAt": 999000, "reachable": false }).section === "setup", 33554432,
              "falha de alcance volta ao endereco");

        // A CONFIGURACAO SE RECOLHE depois de pronta (§5.2), e reaparece quando
        // o proximo gesto vive nela.
        check(acoes.setupExpanded(rules.stateFor(fatos({ "hasInstance": false }))) === true,
              67108864, "sem instancia, a configuracao fica aberta");
        check(acoes.setupExpanded(rules.stateFor(fatos({ "probedAt": 999000,
                                                         "reachable": true }))) === false,
              134217728, "configurado e de pe': a configuracao recolhe");
        check(acoes.setupExpanded(rules.stateFor(fatos({ "probedAt": 999000,
                                                         "reachable": false }))) === true,
              268435456, "falhou: a configuracao reabre");

        // A POLITICA DE TOKEN SO' APARECE QUANDO O SERVIDOR PEDIU. Este e' o
        // defeito medido da tela de hoje — tres politicas antes de alguem
        // precisar de uma — virado regra.
        check(acoes.authVisible(rules.stateFor(fatos({ "hasInstance": false }))) === false,
              536870912, "sem instancia nao se escolhe politica de token");
        check(acoes.authVisible(rules.stateFor(fatos({ "probedAt": 999000,
                                                       "reachable": true }))) === false,
              1073741824, "alcancavel sem exigir token: nada de politica");
        check(acoes.authVisible(rules.stateFor(fatos({ "probedAt": 999000, "reachable": true,
                                                       "tokenRequired": true }))) === true,
              2147483648, "quando o servidor pede, ai' sim");

        Qt.exit(root.failures === 0 ? 0 : 1);
    }
}
