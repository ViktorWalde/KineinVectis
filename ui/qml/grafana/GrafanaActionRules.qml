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

    // A SEGUNDA LINHA DO CABECALHO (§5.2): "autenticado em X · medido agora".
    //
    // `tempo` e' o `GrafanaStateRules`, passado pelo controller: quem SABE a
    // idade e o host e' quem deriva os fatos; quem decide o que dizer com eles
    // e' este arquivo. A separacao nao e' cerimonia — a catraca de duplicacao
    // acusou `auth === "required"` nos dois lugares, e ela esta' certa: um
    // arquivo PRODUZ o estado, o outro o LE, e misturar os dois papeis e' como
    // `severity` ficou azul num painel e vermelha no outro.
    //
    // Uma frase so', com dono unico, porque a alternativa e' o que o painel
    // fazia: cada pedaco de informacao aparecendo num canto diferente, e o
    // autor somando de cabeca. Quando o proximo gesto ainda e' configurar, ela
    // cede o lugar para a dica da acao — dizer "nao consultado" ao lado de um
    // botao `Conectar` seria repetir a mesma coisa com outras palavras.
    function statusPhrase(state, e, actionHint, tempo) {
        if (state.setup !== "saved") {
            return actionHint !== undefined ? actionHint : "";
        }
        if (state.probe === "probing") {
            return qsTr("consultando %1…").arg(tempo.hostOf(e.url));
        }
        if (state.probe === "failed") {
            return actionHint !== undefined ? actionHint : "";
        }
        const onde = tempo.hostOf(e.resultUrl !== "" ? e.resultUrl : e.url);
        const quando = tempo.frescorFrase(e);
        // PEDIDO DE TOKEN NAO ENTRA AQUI: quem explica o pedido e' o proprio
        // campo, logo abaixo, e a frase repetida em dois lugares foi o que a
        // cena de inspecao mostrou. O cabecalho volta a falar do que ele sabe:
        // onde, e ha' quanto tempo.
        if (state.probe === "unknown") {
            return qsTr("%1 · %2").arg(onde).arg(quando);
        }
        // SEM AUTENTICACAO NAO E' FALHA, e a frase nao pode soar como uma:
        // e' o que se ve de fora, e o que se ve de fora tem valor.
        return state.auth === "authenticated"
             ? qsTr("autenticado em %1 · %2").arg(onde).arg(quando)
             : qsTr("sem autenticação em %1 · %2").arg(onde).arg(quando);
    }

    // O QUE DIZER NA AREA DO CONTEUDO quando nao ha' conteudo (§5.2: "estados
    // vazio, carregando, erro e sem autenticacao possuem texto e acao
    // proprios"). A ACAO e' sempre a primaria — a §5.2 tambem diz que ela e' a
    // unica —, entao aqui so' entra o TEXTO, e ele existe para que area vazia
    // nunca seja area muda.
    // CADA ESTADO FALA UMA VEZ SO'. Carregando ja' tem dono — a frase do
    // cabecalho diz "consultando X…", com o endereco —, o pedido de token e'
    // explicado pelo proprio campo, e o endereco que nao respondeu, pelo
    // veredito. Uma segunda frase aqui seria a terceira copia da mesma coisa,
    // que foi o que a cena de inspecao pegou duas vezes nesta fatia.
    function contentPhrase(state, e) {
        // O VELHO SE ANUNCIA ANTES DE TUDO, inclusive antes do erro: e' o caso
        // em que a tela mostra conteudo E deu errado, e sem esta frase o
        // conteudo passaria por atual.
        if (state.content === "stale") {
            return qsTr("o que está aqui é da medição anterior, e pode não valer mais.");
        }
        if (state.auth === "required" || state.auth === "failed"
            || state.probe === "failed" || state.setup !== "saved") {
            return "";
        }
        if (state.content === "unknown") {
            return qsTr("ainda não consultado nesta sessão.");
        }
        if (state.content === "empty") {
            // SEM TOKEN A API NAO LISTA: dizer "nao ha' dashboards" seria
            // afirmar sobre o que a IDE nao pode ver.
            return state.auth === "authenticated"
                 ? qsTr("este Grafana não tem fontes de dados nem dashboards.")
                 : qsTr("sem token, a API não lista fontes nem dashboards — a versão e a saúde já são visíveis.");
        }
        return "";
    }

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
