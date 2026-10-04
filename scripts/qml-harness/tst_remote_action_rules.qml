import QtQuick
import "../../ui/qml/remote"

// A regra de UMA acao primaria por estado (R1/V2). As regras REAIS, puras.
//
// O que se trava aqui: a ORDEM do fluxo. Oferecer "Sondar" a quem nao tem alvo,
// ou "Abrir pasta" a quem a sonda acabou de recusar, e' o painel mandando a
// pessoa para um beco. E toda acao carrega o MOTIVO de ser o proximo passo —
// acao primaria sem motivo vira adivinhacao.
Item {
    id: root

    property int failures: 0

    function check(ok, message) {
        if (!ok) {
            failures++;
            console.error(message);
        }
    }

    RemoteActionRules {
        id: rules
    }

    function stateWith(extra) {
        const base = {
            "savedTarget": true, "namedDraft": false, "probing": false,
            "probed": true, "probeOk": true, "failure": "", "isMirror": false,
            "syncing": false, "hasRemoteFolder": false
        };
        for (const k in extra) {
            base[k] = extra[k];
        }
        return base;
    }

    Component.onCompleted: {
        // Sem alvo salvo: o unico gesto possivel e' criar um, e ele fica
        // DESABILITADO ate' o rascunho ter nome — com a frase dizendo o que falta.
        // "Visao geral" e' o padrao (decisao do autor): sem alvo, a acao NAO
        // pode ser um botao cinza — isso seria um beco. Ela leva a Configurar.
        let a = rules.primaryFor(stateWith({ "savedTarget": false }));
        check(a.kind === "configurar" && a.enabled, "sem alvo: leva a Configurar, habilitada");
        check(rules.sectionFor(a.kind) === "configurar", "e a seccao dela e' Configurar");
        check(a.hint.indexOf("ssh") >= 0, "diz como resolver: " + a.hint);
        a = rules.primaryFor(stateWith({ "savedTarget": false, "namedDraft": true }));
        check(a.kind === "salvar" && a.enabled, "com rascunho nomeado, salvar habilita");

        // Chave recusada tem UM gesto que resolve, e nao e' sondar de novo.
        a = rules.primaryFor(stateWith({ "probeOk": false, "failure": "authentication" }));
        check(a.kind === "copiarChave", "chave recusada -> copiar chave, nao sondar");
        check(a.hint.indexOf("antes de rodar") >= 0, "avisa que a linha aparece antes");

        // As outras falhas NAO viram copiar chave: copiar chave nao resolve DNS.
        for (const f of ["host", "network", "other"]) {
            const r = rules.primaryFor(stateWith({ "probeOk": false, "failure": f }));
            check(r.kind === "sondar", "falha `" + f + "` -> sondar, deu " + r.kind);
        }

        // Primeira conexao (0.153.0): confiar no servidor, so' depois de a
        // impressao digital chegar; nunca "copiar chave" nem digitar `yes`.
        a = rules.primaryFor(stateWith({ "probeOk": false, "firstContact": true }));
        check(a.kind === "confiar" && !a.enabled, "sem a impressao ainda, confiar espera");
        a = rules.primaryFor(stateWith({ "probeOk": false, "firstContact": true, "serverKeyRead": true }));
        check(a.kind === "confiar" && a.enabled, "com a impressao, confiar habilita");
        a = rules.primaryFor(stateWith({ "probeOk": false, "firstContact": true, "serverKeyRead": true,
                                       "trusting": true }));
        check(a.busy && !a.enabled, "confiando: ocupado");
        // A identidade MUDOU: a IDE nao oferece confiar de novo.
        a = rules.primaryFor(stateWith({ "probeOk": false, "failure": "hostKeyChanged" }));
        check(a.kind === "sondar" && a.hint.indexOf("mudou") >= 0, "identidade mudada nao vira confiar");
        // A linha da chave armada vira o gesto; depois de rodar, sondar.
        a = rules.primaryFor(stateWith({ "probeOk": false, "failure": "authentication", "armedLine": true }));
        check(a.kind === "rodarArmada", "linha armada -> rodar no terminal");
        a = rules.primaryFor(stateWith({ "probeOk": false, "failure": "authentication", "keySent": true }));
        check(a.kind === "sondar", "chave enviada -> sondar de novo");

        // Nunca sondado.
        a = rules.primaryFor(stateWith({ "probed": false, "probeOk": false }));
        check(a.kind === "sondar" && a.enabled, "alvo novo -> sondar");
        check(a.hint.indexOf("mede") >= 0, "diz o que a sonda faz");

        // Sondado e sem espelho: sem path, o gesto começa na home do alvo.
        a = rules.primaryFor(stateWith({}));
        check(a.kind === "abrirPasta" && a.enabled && a.label.indexOf("Escolher") >= 0,
              "sem pasta remota, oferece escolha desde a home");
        check(a.hint.indexOf("home") >= 0, "diz por onde começa: " + a.hint);
        a = rules.primaryFor(stateWith({ "hasRemoteFolder": true }));
        check(a.kind === "abrirPasta" && a.enabled, "com pasta, abre");

        // Ja' e' espelho: o gesto diario vira sincronizar.
        a = rules.primaryFor(stateWith({ "isMirror": true }));
        check(a.kind === "puxar" && a.enabled, "espelho aberto -> puxar");

        // Ocupado vence tudo, e nao oferece gesto nenhum.
        a = rules.primaryFor(stateWith({ "probing": true }));
        check(a.busy && !a.enabled && a.kind === "", "sondando: ocupado, sem gesto");
        a = rules.primaryFor(stateWith({ "syncing": true, "isMirror": true }));
        check(a.busy && !a.enabled, "sincronizando: ocupado");
        // Sincronizar vence sondar: se os dois rodassem, o de fora e' o rsync.
        a = rules.primaryFor(stateWith({ "syncing": true, "probing": true }));
        check(a.label.indexOf("Sincronizando") >= 0, "sincronizar vem antes de sondar");

        // TODA acao carrega motivo, sempre.
        for (const sample of [{}, { "savedTarget": false }, { "isMirror": true },
                            { "probing": true }, { "probeOk": false, "failure": "authentica" + "tion" }]) {
            const r = rules.primaryFor(stateWith(sample));
            check(r.hint !== undefined && r.hint !== "", "sem motivo em " + JSON.stringify(sample));
            check(r.label !== "", "sem rotulo em " + JSON.stringify(sample));
        }

        // A seccao onde o gesto vive, para o painel LEVAR ate' ela.
        check(rules.sectionFor("salvar") === "configurar", "salvar vive em Configurar");
        check(rules.sectionFor("configurar") === "configurar", "configurar leva a Configurar");
        check(rules.sectionFor("abrirPasta") === "workspace", "abrir pasta vive em Workspace");
        check(rules.sectionFor("sondar") === "", "sondar nao muda de seccao");

        Qt.exit(failures === 0 ? 0 : 1);
    }
}
