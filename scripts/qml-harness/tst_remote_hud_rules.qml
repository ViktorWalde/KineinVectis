import QtQuick
import "../../ui/qml/remote"

// O HUD do Remote na barra de status (fatia V4, 2026-09-25).
//
// Este harness existe para travar as tres frases que a V4 PROIBE:
// estado inicial "ok", sonda velha virando saude atual, e save local que deu
// certo com push que falhou aparecendo como "sincronizado".
Item {
    id: root

    property int failures: 0

    function check(ok, mensagem) {
        if (!ok) {
            failures++;
            console.error(mensagem);
        }
    }

    RemoteHudRules {
        id: rules
    }

    function facts(extra) {
        const base = {
            "isMirror": true, "targetName": "pi", "syncing": false, "syncDirection": "",
            "deploying": false, "syncFailed": false, "syncMessage": "",
            "probed": true, "probeOk": true, "probeAgeMs": 1000
        };
        for (const k in extra) {
            base[k] = extra[k];
        }
        return base;
    }

    Component.onCompleted: {
        // Sem espelho, o Remote NAO ocupa a barra (§5.2, por escrito).
        check(!rules.hudFor(facts({ "isMirror": false })).visible,
              "perfil so' selecionado nao ocupa a barra");

        // ESTADO INICIAL: nunca sondado nao e' "ok".
        let h = rules.hudFor(facts({ "probed": false }));
        check(h.visible && h.label.indexOf("não verificado nesta sessão") >= 0,
              "facts inicial: " + h.label);
        check(h.tone !== "ok", "nunca sondado nao pode ter tom de ok");

        // A FRASE PROIBIDA: local salvo, envio falhou -> NAO e' "sincronizado",
        // e o detalhe diz que o arquivo local esta' intacto.
        h = rules.hudFor(facts({ "syncFailed": true, "syncMessage": "connection closed" }));
        check(h.label.indexOf("sincronizado") < 0, "envio falhou nao pode dizer sincronizado");
        check(h.label.indexOf("envio falhou") >= 0, "diz que o envio falhou");
        check(h.tone === "atencao", "e chama atencao");
        check(h.detail.indexOf("local está salvo") >= 0, "tranquiliza sobre o arquivo local");
        check(h.detail.indexOf("connection closed") >= 0, "e repassa o que o rsync disse");

        // A falha de sincronia VENCE uma sonda boa: sondar nao prova que o
        // envio chegou.
        h = rules.hudFor(facts({ "syncFailed": true, "probeOk": true, "probeAgeMs": 10 }));
        check(h.label.indexOf("envio falhou") >= 0, "sonda boa nao apaga envio falho");

        // SONDA VELHA nao vira saude atual.
        h = rules.hudFor(facts({ "probeAgeMs": 5000 }));
        check(h.label.indexOf("verificado agora") >= 0 && h.tone === "ok",
              "medida fresca: " + h.label);
        h = rules.hudFor(facts({ "probeAgeMs": 3600000 }));
        check(h.label.indexOf("verificado") >= 0 && h.label.indexOf("há") >= 0,
              "a age entra na phrase: " + h.label);
        check(h.tone !== "ok", "medida velha perde o tom de ok");

        // TODA afirmacao de saude leva origem e idade no detalhe (§4).
        for (const sample of [{}, { "probeAgeMs": 3600000 }, { "probeOk": false }]) {
            const r = rules.hudFor(facts(sample));
            check(r.detail.indexOf("há") >= 0, "sem age no detalhe: " + r.detail);
        }
        check(rules.hudFor(facts({})).detail.indexOf("não há conexão aberta") >= 0,
              "diz que nao ha' conexao viva — sondar nao e' estar conectado");

        // Ocupado: o gesto em curso aparece, e manda para Jobs.
        h = rules.hudFor(facts({ "syncing": true, "syncDirection": "push" }));
        check(h.label.indexOf("enviando") >= 0 && h.tone === "ocupado", "push em curso");
        h = rules.hudFor(facts({ "syncing": true, "syncDirection": "pull" }));
        check(h.label.indexOf("puxando") >= 0, "pull em curso");
        check(h.detail.indexOf("Jobs") >= 0, "aponta para Jobs");
        h = rules.hudFor(facts({ "deploying": true }));
        check(h.label.indexOf("deploy") >= 0 && h.tone === "ocupado", "deploy em curso");

        // A idade em palavras.
        check(rules.age(5000).indexOf("s") >= 0, "segundos");
        check(rules.age(120000).indexOf("min") >= 0, "minutos");
        check(rules.age(7200000).indexOf("h") >= 0, "horas");
        check(rules.age(-1) === "", "age desconhecida nao inventa numero");
        check(rules.age(undefined) === "", "age ausente nao inventa numero");

        // "SINCRONIZADO" NAO EXISTE MAIS NO HUD (2026-09-25). A palavra fala de
        // ARQUIVOS; toda frase que o HUD sustenta vem de uma sonda, que mede
        // ALCANCE. A cena de inspecao mostrou as duas ok lado a lado — "agora:
        // sincronizado" e "ha' 40 min: verificado" — e o assunto trocava com a
        // idade. Nenhum estado pode trazer a palavra de volta.
        const everyState = [
            {}, { "probeAgeMs": 0 }, { "probeAgeMs": 5000 }, { "probeAgeMs": 3600000 },
            { "probed": false }, { "probeOk": false }, { "syncFailed": true },
            { "syncing": true, "syncDirection": "push" },
            { "syncing": true, "syncDirection": "pull" }, { "deploying": true }
        ];
        for (const sample of everyState) {
            const phrase = rules.hudFor(facts(sample));
            check(phrase.label.indexOf("sincroniz") < 0,
                  "o HUD voltou a falar de sincronia: " + phrase.label);
        }

        // O alvo sempre aparece: "SSH · <alvo> · ...".
        check(rules.hudFor(facts({})).label.indexOf("SSH · pi · ") === 0,
              "prefixo com o alvo");

        Qt.exit(failures === 0 ? 0 : 1);
    }
}
