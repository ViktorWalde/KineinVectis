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

    function check(ok, mensagem) {
        if (!ok) {
            failures++;
            console.error(mensagem);
        }
    }

    RemoteActionRules {
        id: regras
    }

    function estado(extra) {
        const base = {
            "temAlvoSalvo": true, "rascunhoNomeado": false, "sondando": false,
            "sondou": true, "sondaOk": true, "falha": "", "eEspelho": false,
            "sincronizando": false, "temPastaRemota": false
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
        let a = regras.primaryFor(estado({ "temAlvoSalvo": false }));
        check(a.kind === "configurar" && a.enabled, "sem alvo: leva a Configurar, habilitada");
        check(regras.sectionFor(a.kind) === "configurar", "e a seccao dela e' Configurar");
        check(a.hint.indexOf("ssh") >= 0, "diz como resolver: " + a.hint);
        a = regras.primaryFor(estado({ "temAlvoSalvo": false, "rascunhoNomeado": true }));
        check(a.kind === "salvar" && a.enabled, "com rascunho nomeado, salvar habilita");

        // Chave recusada tem UM gesto que resolve, e nao e' sondar de novo.
        a = regras.primaryFor(estado({ "sondaOk": false, "falha": "authentication" }));
        check(a.kind === "copiarChave", "chave recusada -> copiar chave, nao sondar");
        check(a.hint.indexOf("antes de rodar") >= 0, "avisa que a linha aparece antes");

        // As outras falhas NAO viram copiar chave: copiar chave nao resolve DNS.
        for (const f of ["host", "network", "other"]) {
            const r = regras.primaryFor(estado({ "sondaOk": false, "falha": f }));
            check(r.kind === "sondar", "falha `" + f + "` -> sondar, deu " + r.kind);
        }

        // Nunca sondado.
        a = regras.primaryFor(estado({ "sondou": false, "sondaOk": false }));
        check(a.kind === "sondar" && a.enabled, "alvo novo -> sondar");
        check(a.hint.indexOf("mede") >= 0, "diz o que a sonda faz");

        // Sondado e sem espelho: abrir a pasta, mas so' se ela foi informada.
        a = regras.primaryFor(estado({}));
        check(a.kind === "abrirPasta" && !a.enabled, "sem pasta remota, desabilitado");
        check(a.hint.indexOf("Workspace") >= 0, "diz em que seccao informar: " + a.hint);
        a = regras.primaryFor(estado({ "temPastaRemota": true }));
        check(a.kind === "abrirPasta" && a.enabled, "com pasta, abre");

        // Ja' e' espelho: o gesto diario vira sincronizar.
        a = regras.primaryFor(estado({ "eEspelho": true }));
        check(a.kind === "puxar" && a.enabled, "espelho aberto -> puxar");

        // Ocupado vence tudo, e nao oferece gesto nenhum.
        a = regras.primaryFor(estado({ "sondando": true }));
        check(a.busy && !a.enabled && a.kind === "", "sondando: ocupado, sem gesto");
        a = regras.primaryFor(estado({ "sincronizando": true, "eEspelho": true }));
        check(a.busy && !a.enabled, "sincronizando: ocupado");
        // Sincronizar vence sondar: se os dois rodassem, o de fora e' o rsync.
        a = regras.primaryFor(estado({ "sincronizando": true, "sondando": true }));
        check(a.label.indexOf("Sincronizando") >= 0, "sincronizar vem antes de sondar");

        // TODA acao carrega motivo, sempre.
        for (const caso of [{}, { "temAlvoSalvo": false }, { "eEspelho": true },
                            { "sondando": true }, { "sondaOk": false, "falha": "authentica" + "tion" }]) {
            const r = regras.primaryFor(estado(caso));
            check(r.hint !== undefined && r.hint !== "", "sem motivo em " + JSON.stringify(caso));
            check(r.label !== "", "sem rotulo em " + JSON.stringify(caso));
        }

        // A seccao onde o gesto vive, para o painel LEVAR ate' ela.
        check(regras.sectionFor("salvar") === "configurar", "salvar vive em Configurar");
        check(regras.sectionFor("configurar") === "configurar", "configurar leva a Configurar");
        check(regras.sectionFor("abrirPasta") === "workspace", "abrir pasta vive em Workspace");
        check(regras.sectionFor("sondar") === "", "sondar nao muda de seccao");

        Qt.exit(failures === 0 ? 0 : 1);
    }
}
