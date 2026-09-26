import QtQuick

// QUANDO O TEXTO CHEGA AO RENDERER (fatia V5/M2, 2026-09-25).
//
// §7: "renderizacao nao roda no caminho sincrono do TextEdit de codigo". O
// renderer do Qt reconstroi o documento INTEIRO a cada `setMarkdown`, no thread
// da UI — ligar o buffer direto nele remontava tudo a cada caractere digitado,
// e a regua da especificacao e' a latencia da tecla.
//
// §6 e §10: "`documentId` evita que debounce de uma aba atualize outra" e
// "resposta atrasada nunca aparece na aba errada". E' o principio da identidade
// da V5 uma camada adiante: o texto que espera o prazo pode ser de outro
// documento quando ele vence.
//
// Mora num objeto proprio por dois motivos. O primeiro e' que isto e' REGRA, e
// nao desenho. O segundo e' pratico e vale registrar: o harness QML carrega os
// componentes de um espelho plano das FONTES, onde os tipos registrados em C++
// (`QML_ELEMENT`) nao existem — qualquer componente que toque num deles fica
// sem teste. Separando a regra do widget, ela volta a ser testavel.
QtObject {
    id: root

    // O buffer como esta' agora.
    property string content: ""
    // De qual documento ele e'.
    property int docId: 0

    // 150 ms fica dentro da faixa de 100-200 ms que a §7 propoe: curto o
    // bastante para a previa parecer viva, longo o bastante para uma rajada de
    // digitacao virar UM render.
    property int debounceMs: 150

    // O que ja' foi entregue ao renderer, e de QUAL documento.
    property string rendered: ""
    property int renderedDocId: 0

    // Enquanto o texto do documento novo nao chega, nao ha' o que mostrar.
    property bool awaitingFirstText: true

    readonly property Timer deadline: Timer {
        interval: root.debounceMs
        repeat: false
        onTriggered: {
            root.rendered = root.content;
            root.renderedDocId = root.docId;
        }
    }

    // O CASO DOS DOIS DOCUMENTOS COM O MESMO TEXTO (2026-09-25).
    //
    // Trocar de aba zera a previa e espera o texto do documento novo. Mas se os
    // dois tem conteudo IDENTICO, `onContentChanged` nunca dispara — a string
    // nao mudou — e a previa ficaria em branco para sempre.
    //
    // Este prazo curto fecha esse buraco com seguranca: a cadeia que troca de
    // documento e' SINCRONA (o controller muda o id e chama `setText` na mesma
    // funcao), entao passados 50 ms sem nenhuma mudanca de texto, o conteudo
    // que esta' aqui JA' E' o do documento novo.
    readonly property Timer settle: Timer {
        interval: 50
        repeat: false
        onTriggered: {
            if (root.awaitingFirstText) {
                root.renderNow();
            }
        }
    }

    // TROCAR DE DOCUMENTO ZERA A PREVIA, e isto e' deliberado. A alternativa —
    // deixar o documento anterior na tela ate' o novo chegar — mostraria o
    // texto de um arquivo sob o nome de outro, que e' exatamente o que a §10
    // proibe. Em branco e' honesto; errado nao e'.
    //
    // O prazo da digitacao e' PARADO aqui, e e' assim que "resposta atrasada
    // nunca aparece na aba errada" se cumpre: por cancelamento, e nao por
    // etiqueta. Uma versao anterior carregava tambem um `pendingDocId`
    // comparado no disparo — o teste de mutacao mostrou que aquele ramo era
    // inalcancavel, e codigo morto com cara de protecao e' pior que nada.
    onDocIdChanged: {
        root.deadline.stop();
        root.rendered = "";
        root.renderedDocId = root.docId;
        root.awaitingFirstText = true;
        root.settle.restart();
    }

    onContentChanged: {
        if (root.awaitingFirstText) {
            root.renderNow();
            return;
        }
        root.deadline.restart();
    }

    function renderNow() {
        root.deadline.stop();
        root.settle.stop();
        root.rendered = root.content;
        root.renderedDocId = root.docId;
        root.awaitingFirstText = false;
    }
}
