import QtQuick

// UMA ACAO PRIMARIA POR ESTADO (fatia V7/G1).
//
// A tela de hoje mostra `Salvar · Sondar · Esquecer` com o mesmo peso, e a §3 da
// especificacao nomeia o atrito: "a acao inicial e' ambigua: salvar antes de
// sondar e' exigencia do DESENHO, nao intencao do usuario".
//
// Aqui o estado decide qual e' o proximo gesto, e ele e' UM. Os demais
// continuam alcancaveis — em `configurar…` —, mas param de disputar a atencao
// de quem so' quer ver um dashboard.
//
// A §4 tambem exige que ERRO SEMPRE OFEREÇA ACAO SEGUINTE: corrigir URL, tentar
// de novo ou fornecer token. Por isso nao existe estado sem `kind` aqui.
//
// Mesmo idioma do `RemoteActionRules`, e pelo mesmo motivo: a decisao e' regra,
// tem harness, e a tela so' desenha.
QtObject {
    id: root

    // { kind, label, hint, section }
    //
    // `section` diz ONDE o gesto vive, para o painel LEVAR a pessoa ate' la' em
    // vez de acender um botao que ela nao acha.
    function primaryFor(state, facts) {
        // SEM INSTANCIA, ou editando: o unico campo e' a URL, e o unico gesto
        // e' Conectar. A §4 e' explicita: "URL e' o unico campo inicial".
        if (state.setup !== "saved") {
            return {
                "kind": "connect",
                "label": qsTr("Conectar"),
                "hint": qsTr("testa o endereço antes de virar trabalho diário"),
                "section": "setup"
            };
        }
        if (state.probe === "probing") {
            return {
                "kind": "waiting",
                "label": qsTr("Consultando…"),
                "hint": qsTr("perguntando ao Grafana pela API dele"),
                "section": "overview"
            };
        }
        // TOKEN EXIGIDO ou RECUSADO: o proximo gesto e' a credencial, e so'
        // agora — antes disso a tela nao oferece politica de token nenhuma.
        if (state.auth === "required" || state.auth === "failed") {
            return {
                "kind": "provideToken",
                "label": state.auth === "failed" ? qsTr("Tentar outro token")
                                                 : qsTr("Fornecer token"),
                "hint": state.auth === "failed"
                        ? qsTr("o servidor recusou o token desta sessão")
                        : qsTr("o servidor pediu autenticação; fica só em memória"),
                "section": "auth"
            };
        }
        if (state.probe === "failed") {
            return {
                "kind": "fixUrl",
                "label": qsTr("Revisar endereço"),
                "hint": qsTr("o endereço não respondeu — confira ou tente de novo"),
                "section": "setup"
            };
        }
        // NUNCA CONSULTADO NESTA SESSAO: medir e' o proximo gesto, e o botao
        // diz isso em vez de "Sondar", que e' vocabulario de quem escreveu o
        // codigo.
        if (state.probe === "unknown") {
            return {
                "kind": "refresh",
                "label": qsTr("Atualizar"),
                "hint": qsTr("ainda não consultado nesta sessão"),
                "section": "overview"
            };
        }
        if (state.content === "stale") {
            return {
                "kind": "refresh",
                "label": qsTr("Atualizar"),
                "hint": qsTr("o que está na tela é de antes; atualizar mede de novo"),
                "section": "overview"
            };
        }
        // Tudo em ordem: atualizar continua sendo a unica acao primaria (§5.2).
        return {
            "kind": "refresh",
            "label": qsTr("Atualizar"),
            "hint": state.content === "empty"
                    ? qsTr("o Grafana respondeu sem fontes nem dashboards")
                    : qsTr("mede de novo pela API"),
            "section": "overview"
        };
    }

    // A CONFIGURACAO FICA RECOLHIDA depois de pronta (§5.2). Ela reaparece
    // quando o proximo gesto vive nela — e' o mesmo principio do `section`.
    function setupExpanded(state) {
        return state.setup !== "saved" || state.probe === "failed";
    }

    // A politica de token so' aparece quando o servidor pediu (§5.1: "somente
    // se necessaria"). Este e' o defeito medido da tela de hoje, virado regra.
    function authVisible(state) {
        return state.auth === "required" || state.auth === "failed";
    }
}
