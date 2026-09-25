import QtQuick

// A REGRA PURA de "uma acao primaria por estado" (fatia R1/V2, 2026-09-24).
//
// A §4 da `especificacoes/remote-ssh-ui-hud.md` pede "uma acao primaria por
// estado"; a §3 diagnostica o contrario no painel atual: "todas as acoes
// aparecem com peso parecido". A foto de 2026-09-24 mostrou isso piorando — as
// duas entradas novas da R0.5 empurraram Enviar/Rodar/depurar para fora da tela.
//
// Aqui nao ha' UI: dado o ESTADO, sai qual e' o proximo gesto. Regra pura, no
// mesmo idioma de `ProblemRules`/`GitRules`, e por isso testavel em harness.
//
// A ORDEM importa, e e' a do fluxo real: nao adianta oferecer "Sondar" a quem
// ainda nao tem alvo, nem "Abrir pasta" a quem a sonda acabou de recusar.
QtObject {
    id: root

    // { label, kind, enabled, busy, hint }
    //
    // `kind` e' o que o host chama; `hint` e' a frase curta que explica por que
    // ESTE e' o proximo passo — sem ela a acao primaria vira adivinhacao.
    function primaryFor(e) {
        if (e.sincronizando) {
            return { label: qsTr("Sincronizando…"), kind: "", enabled: false, busy: true,
                     hint: qsTr("o rsync está movendo os arquivos") };
        }
        if (e.sondando) {
            return { label: qsTr("Sondando…"), kind: "", enabled: false, busy: true,
                     hint: qsTr("o ssh está medindo o alvo, até 5 s") };
        }
        // Sem alvo salvo, o unico gesto que faz sentido e' criar um.
        //
        // Sem nem rascunho, a acao LEVA a Configurar em vez de ficar cinza:
        // com "Visao geral" como padrao (decisao do autor, 2026-09-24), um
        // botao desabilitado seria um beco — a pessoa veria o que falta sem ter
        // um gesto para resolver.
        if (!e.temAlvoSalvo) {
            if (e.rascunhoNomeado === true) {
                return { label: qsTr("Salvar alvo"), kind: "salvar", enabled: true, busy: false,
                         hint: qsTr("grava o perfil neste projeto") };
            }
            return { label: qsTr("Configurar alvo"), kind: "configurar", enabled: true,
                     busy: false,
                     hint: qsTr("escolha um alias do seu ~/.ssh/config ou cole a linha ssh") };
        }
        // A chave recusada tem UM gesto que a resolve, e nao e' sondar de novo.
        if (e.sondou && !e.sondaOk && e.falha === "authentication") {
            return { label: qsTr("Copiar minha chave"), kind: "copiarChave", enabled: true,
                     busy: false,
                     hint: qsTr("o alvo recusou a chave; a linha aparece antes de rodar") };
        }
        if (!e.sondou || !e.sondaOk) {
            return { label: qsTr("Sondar"), kind: "sondar", enabled: true, busy: false,
                     hint: e.sondou
                         ? qsTr("tente de novo depois de resolver o que o erro diz")
                         : qsTr("mede arquitetura, kernel e ferramentas do alvo") };
        }
        // Sondado e' um espelho aberto: o gesto diario vira sincronizar.
        if (e.eEspelho) {
            return { label: qsTr("Puxar do alvo"), kind: "puxar", enabled: true, busy: false,
                     hint: qsTr("traz o que mudou no alvo para o espelho local") };
        }
        return { label: qsTr("Abrir pasta no alvo"), kind: "abrirPasta",
                 enabled: e.temPastaRemota === true, busy: false,
                 hint: e.temPastaRemota
                     ? qsTr("abre a pasta do alvo como espelho local")
                     : qsTr("informe a pasta do alvo em Workspace") };
    }

    // Qual seccao o proximo gesto vive, para o painel LEVAR a pessoa ate' ela
    // em vez de habilitar um botao que ela nao acha.
    function sectionFor(kind) {
        switch (kind) {
        case "configurar": return "configurar";
        case "salvar": return "configurar";
        case "abrirPasta": return "workspace";
        case "puxar": return "workspace";
        default: return "";
        }
    }
}
