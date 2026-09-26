import QtQuick

// A MAQUINA DE ESTADOS DA APRESENTACAO (§6 da especificacao, fatia V7/G1).
//
// Quatro eixos ORTOGONAIS, e a ortogonalidade e' o ponto: colapsa-los num
// "estado" so' foi o que produziu a tela de hoje, em que tres momentos
// diferentes — conectar, autenticar e usar — dividem a mesma coluna.
//
//   setup    absent | editing | saved
//   probe    unknown | probing | reachable | failed
//   auth     not_required | required | authenticated | failed
//   content  unknown | loading | ready | empty | stale
//
// As regras que a §6 escreve, e que este objeto existe para nao deixar
// esquecer:
//
//   - `reachable` NAO implica `authenticated`;
//   - trocar a URL invalida visualmente o resultado anterior;
//   - "atualizado" exige resposta DESTA sessao, com horario e idade;
//   - falha de atualizacao mantem o resultado anterior como DESATUALIZADO,
//     nunca como resultado novo;
//   - fechar e reabrir a janela nao transforma dado antigo em prova atual.
//
// Entra fato, sai estado. Sem UI, sem IPC — mesmo idioma de `RemoteHudRules` e
// `ProblemRules`.
QtObject {
    id: root

    // Depois disto, uma medida deixa de ser "agora" na frase curta.
    readonly property int frescorMs: 60000

    function setupFor(e) {
        if (e.hasInstance !== true) {
            return e.editing === true ? "editing" : "absent";
        }
        return e.editing === true ? "editing" : "saved";
    }

    function probeFor(e) {
        if (e.probing === true) {
            return "probing";
        }
        // NUNCA SONDADO NESTA SESSAO nao e' "falhou": e' desconhecido. Dizer
        // "falhou" aqui inventaria uma medicao que ninguem fez.
        if (e.probedAt === undefined || e.probedAt <= 0) {
            return "unknown";
        }
        return e.reachable === true ? "reachable" : "failed";
    }

    // `reachable` NAO implica `authenticated`, e o contrario tambem nao vale:
    // um alvo que exige token pode estar de pe' e recusando.
    function authFor(e) {
        if (e.authenticated === true) {
            return "authenticated";
        }
        if (e.tokenRequired === true) {
            return e.authFailed === true ? "failed" : "required";
        }
        // "Nao exigido" e' o que se sabe ate' a API pedir. E' deliberado que a
        // tela NAO mostre politica de token aqui: a §5.1 poe a autenticacao em
        // "somente se necessaria", e o defeito medido na tela de hoje e'
        // exatamente oferecer tres politicas antes de alguem precisar de uma.
        return "not_required";
    }

    // O eixo mais fácil de falsear: conteudo na tela nao prova conteudo ATUAL.
    function contentFor(e) {
        if (e.probing === true) {
            return "loading";
        }
        if (e.probedAt === undefined || e.probedAt <= 0) {
            return "unknown";
        }
        const total = (e.dataSourceCount || 0) + (e.dashboardCount || 0);
        // A URL DE AGORA NAO E' A QUE RESPONDEU: o que esta' na tela pertence a
        // outra instancia, e continuar mostrando como atual seria mentira.
        if (e.resultUrl !== undefined && e.url !== undefined && e.resultUrl !== e.url) {
            return "stale";
        }
        // A ULTIMA TENTATIVA FALHOU, mas ha' resultado anterior: ele vira
        // DESATUALIZADO, e nunca resultado novo.
        if (e.reachable !== true) {
            return total > 0 ? "stale" : "unknown";
        }
        return total > 0 ? "ready" : "empty";
    }

    // Os quatro de uma vez, que e' como a tela os consome.
    function stateFor(e) {
        return {
            "setup": root.setupFor(e),
            "probe": root.probeFor(e),
            "auth": root.authFor(e),
            "content": root.contentFor(e)
        };
    }

    // "ha' 3 min", "ha' 2 h" — a §6 exige horario/idade em toda afirmacao de
    // atualidade.
    function idade(ms) {
        if (ms === undefined || ms === null || ms < 0) {
            return "";
        }
        const seg = Math.floor(ms / 1000);
        if (seg < 60) {
            return qsTr("há %n s", "", seg);
        }
        const min = Math.floor(seg / 60);
        if (min < 60) {
            return qsTr("há %n min", "", min);
        }
        const h = Math.floor(min / 60);
        return h < 24 ? qsTr("há %n h", "", h) : qsTr("há %n dia(s)", "", Math.floor(h / 24));
    }

    // A frase de atualidade. "Medido agora" so' vale para medida DESTA sessao e
    // recente; o resto carrega a idade.
    function frescorFrase(e) {
        if (e.probedAt === undefined || e.probedAt <= 0) {
            return qsTr("não consultado nesta sessão");
        }
        const idadeMs = (e.agora || 0) - e.probedAt;
        return idadeMs < root.frescorMs
                ? qsTr("medido agora")
                : qsTr("medido %1").arg(root.idade(idadeMs));
    }
}
