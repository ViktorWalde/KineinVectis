import QtQuick

// A REGRA PURA do HUD do Remote na barra de status (fatia V4, 2026-09-25).
//
// A §5.2 da `especificacoes/remote-ssh-ui-hud.md` pede o item compacto; a §4
// diz o que ele NAO pode fazer: "nao chamar de 'conectado' sem sessao/conexao
// viva que prove isso" e "toda afirmacao de saude mostra origem e idade da
// medicao". A §3 lista como defeito 5: "'sondado uma vez' pode parecer
// 'conectado agora'".
//
// Daqui saem so' frases que o estado sustenta. As tres que a V4 proibe, por
// escrito:
//
//   - estado inicial NAO e' "ok": e' "nao verificado nesta sessao";
//   - sonda velha nao vira saude atual — a idade vai junto, sempre;
//   - save local que deu certo com push que falhou NAO e' "sincronizado";
//   - sonda verde NAO e' "sincronizado": ela mede alcance, nao arquivos.
//
// Sem UI aqui: entra fato, sai frase. Mesmo idioma de `ProblemRules`/`GitRules`.
QtObject {
    id: root

    // Quanto tempo uma medida continua sendo "agora" na frase curta.
    readonly property int freshnessMs: 60000

    // "ha' 3 min", "ha' 2 h". Curto de proposito: e' barra de status.
    function age(ms) {
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
        if (h < 24) {
            return qsTr("há %n h", "", h);
        }
        return qsTr("há %n dia(s)", "", Math.floor(h / 24));
    }

    // { visible, label, tone, detail }
    //
    // `tone`: "neutro" | "ocupado" | "atencao" | "ok".
    function hudFor(e) {
        // Sem workspace espelhado o Remote NAO ocupa a barra: um perfil apenas
        // selecionado nao e' contexto de trabalho (§5.2, por escrito).
        if (!e.isMirror) {
            return { "visible": false, "label": "", "tone": "neutro", "detail": "" };
        }
        const target = e.targetName || qsTr("alvo");
        const prefix = "SSH · " + target + " · ";

        if (e.syncing) {
            const verb = e.syncDirection === "push" ? qsTr("enviando") : qsTr("puxando");
            return { "visible": true, "label": prefix + verb + "…", "tone": "ocupado",
                     "detail": qsTr("rsync em curso; acompanhe em Jobs") };
        }
        if (e.deploying) {
            return { "visible": true, "label": prefix + qsTr("enviando (deploy)…"),
                     "tone": "ocupado", "detail": qsTr("rsync em curso; acompanhe em Jobs") };
        }
        // FALHA DE SINCRONIA VENCE tudo o que viria depois. Este e' o caso que
        // a V4 nomeia: o arquivo local foi salvo, o envio nao foi, e a barra
        // NAO pode dizer "sincronizado".
        if (e.syncFailed) {
            return { "visible": true, "label": prefix + qsTr("envio falhou"), "tone": "atencao",
                     "detail": qsTr("o arquivo local está salvo e intacto; o que falhou foi "
                                    + "levá-lo ao alvo. %1").arg(e.syncMessage || "") };
        }
        if (e.probed && !e.probeOk) {
            return { "visible": true, "label": prefix + qsTr("última sonda falhou"),
                     "tone": "atencao",
                     "detail": qsTr("medido %1 pelo ssh; o alvo pode ter voltado desde então")
                               .arg(root.age(e.probeAgeMs)) };
        }
        // NUNCA sondado nesta sessao: o estado inicial exigido pela V4. Dizer
        // "ok" aqui seria afirmar saude que ninguem mediu.
        if (!e.probed) {
            return { "visible": true, "label": prefix + qsTr("não verificado nesta sessão"),
                     "tone": "neutro",
                     "detail": qsTr("nenhuma sonda desde que a IDE abriu; Sondar mede o alvo") };
        }
        // Sondou e deu certo. A IDADE vai na frase quando a medida envelhece:
        // "sondado uma vez" nao pode parecer "conectado agora".
        // "sincronizado" seria outra mentira, e estava aqui ate' a foto da cena
        // mostrar as duas frases lado a lado: sincronia e' sobre ARQUIVOS, a
        // sonda mede ALCANCE. O verbo e' o mesmo nos dois casos — so' muda a
        // idade — para a frase nao trocar de assunto quando a medida esfria.
        const stale = e.probeAgeMs !== undefined && e.probeAgeMs >= root.freshnessMs;
        const suffix = stale ? qsTr("verificado %1").arg(root.age(e.probeAgeMs))
                             : qsTr("verificado agora");
        return { "visible": true, "label": prefix + suffix, "tone": stale ? "neutro" : "ok",
                 "detail": qsTr("última sonda %1, pelo ssh do sistema; não há conexão aberta")
                           .arg(root.age(e.probeAgeMs)) };
    }
}
