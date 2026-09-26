// O TECLADO DA ARVORE, LIGADO AO MODELO (P1).
//
// As regras puras tem o `tst_project_tree_keys`. Aqui se mede a FIACAO: que
// a tecla vira o mesmo pedido que o clique faria, que a busca por nome
// acumula o prefixo, e — o mais importante — que uma tecla com modificador
// NAO e' engolida pela busca. Engolir `Ctrl+C` aqui faria a copia parar de
// funcionar na arvore sem ninguem entender por que.
//
// MUTACAO QUE PROVA O GATE: tire o teste de modificador do `procurarPeloNome`
// e a assercao do `Ctrl+C` cai.
import QtQuick
import KineinVectis

Item {
    id: root

    property int falhas: 0
    property var movidos: []
    property var alternados: []
    property var ativados: []

    ListModel {
        id: modelo

        ListElement { path: "/p/src"; name: "src"; kind: "directory"; depth: 0; expanded: true }
        ListElement { path: "/p/src/main.rs"; name: "main.rs"; kind: "file"; depth: 1; expanded: false }
        ListElement { path: "/p/setup.py"; name: "setup.py"; kind: "file"; depth: 0; expanded: false }
    }

    ProjectTreeKeyboard {
        id: teclado

        model: modelo
        currentIndex: 0

        onMoveRequested: indice => root.movidos.push(indice)
        onToggleRequested: indice => root.alternados.push(indice)
        onActivateRequested: indice => root.ativados.push(indice)
    }

    function conferir(condicao, mensagem) {
        if (!condicao) {
            console.error("FALHOU: " + mensagem);
            root.falhas += 1;
        }
    }

    // Um evento de tecla como o QML o entrega.
    function tecla(codigo, texto, modificadores) {
        return {
            "key": codigo,
            "text": texto === undefined ? "" : texto,
            "modifiers": modificadores === undefined ? Qt.NoModifier : modificadores,
            "accepted": false
        };
    }

    Component.onCompleted: {
        // O RETRATO DAS LINHAS sai do modelo, com o que as regras esperam.
        const linhas = teclado.linhas();
        root.conferir(linhas.length === 3, "o retrato do modelo veio com " + linhas.length);
        root.conferir(linhas[0].name === "src" && linhas[0].expanded === true,
                      "o retrato perdeu o estado da pasta");

        // SETA PARA BAIXO vira pedido de mover.
        root.conferir(teclado.handleKey(root.tecla(Qt.Key_Down)), "a seta nao foi consumida");
        root.conferir(root.movidos.length === 1 && root.movidos[0] === 1,
                      "a seta nao pediu para mover para a linha 1");

        // ENTER numa pasta aberta alterna; num arquivo, ativa.
        teclado.currentIndex = 0;
        teclado.handleKey(root.tecla(Qt.Key_Return));
        root.conferir(root.alternados.length === 1 && root.alternados[0] === 0,
                      "Enter na pasta nao pediu para alternar");
        teclado.currentIndex = 1;
        teclado.handleKey(root.tecla(Qt.Key_Return));
        root.conferir(root.ativados.length === 1 && root.ativados[0] === 1,
                      "Enter no arquivo nao pediu para abrir");

        // A BUSCA ACUMULA O PREFIXO enquanto a digitacao e' rapida.
        teclado.currentIndex = 0;
        teclado.esquecerDigitacao();
        teclado.handleKey(root.tecla(Qt.Key_S, "s"));
        root.conferir(teclado.prefixo === "s", "o prefixo nao comecou: " + teclado.prefixo);
        teclado.handleKey(root.tecla(Qt.Key_E, "e"));
        root.conferir(teclado.prefixo === "se", "o prefixo nao acumulou: " + teclado.prefixo);
        root.conferir(root.movidos[root.movidos.length - 1] === 2,
                      "`se` nao levou ao `setup.py`");

        // SAIR DA ARVORE ESQUECE o que estava sendo digitado.
        teclado.esquecerDigitacao();
        root.conferir(teclado.prefixo === "", "a digitacao sobreviveu ao esquecimento");

        // TECLA COM MODIFICADOR NAO E' BUSCA. `Ctrl+C` pertence a outro dono,
        // e engoli-lo aqui mataria a copia sem aviso.
        const antes = root.movidos.length;
        const consumiu = teclado.handleKey(root.tecla(Qt.Key_C, "c", Qt.ControlModifier));
        root.conferir(!consumiu, "o teclado da arvore engoliu um Ctrl+C");
        root.conferir(teclado.prefixo === "", "o Ctrl+C entrou na busca por nome");
        root.conferir(root.movidos.length === antes, "o Ctrl+C moveu o cursor");

        // TECLA SEM TEXTO (um Shift sozinho) tambem nao e' busca.
        root.conferir(!teclado.handleKey(root.tecla(Qt.Key_Shift)),
                      "um Shift sozinho foi tratado como busca");

        // NAO ACHAR E' DIFERENTE DE NAO FAZER NADA: a tecla foi para a busca,
        // e deixa-la seguir dispararia um atalho no meio de uma palavra.
        teclado.esquecerDigitacao();
        root.conferir(teclado.handleKey(root.tecla(Qt.Key_Z, "z")),
                      "a busca sem resultado deixou a tecla escapar");

        // SEM MODELO nao explode.
        teclado.model = null;
        root.conferir(teclado.linhas().length === 0, "retrato de modelo nulo nao veio vazio");
        root.conferir(!teclado.handleKey(root.tecla(Qt.Key_Down)),
                      "a seta fez algo sem modelo");

        Qt.exit(root.falhas === 0 ? 0 : 1);
    }
}
