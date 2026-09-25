import QtQuick

// A LINHA QUE ESPERA UM TERMINAL NASCER (2026-09-25, V4).
//
// O painel Remoto tem "Shell no terminal": o core devolve a linha `ssh ...` e o
// `RuntimeController` a manda para o terminal da IDE. Com NENHUMA sessao aberta
// — o estado normal de quem acabou de abrir a IDE — ele pedia um terminal e
// devolvia SEM ENVIAR NADA. O autor via um terminal vazio, e o painel dizia
// "shell aberto no terminal".
//
// Mora num objeto proprio, e nao em mais trinta linhas do controller, porque o
// controller estava a uma linha do limite de 400 da catraca — e porque isto
// tem estado proprio (a linha guardada e o prazo dela) que ninguem mais usa.
QtObject {
    id: root

    // A linha guardada. Vazia quando nao ha' ninguem esperando.
    property string command: ""

    // Quinze segundos: se a sessao nao nasceu ate' la', a linha NAO fica
    // guardada. Um `ssh` antigo aparecendo num terminal que o autor abriu
    // depois, para outra coisa, seria pior que nao ter enviado. O harness
    // encurta o prazo — e' o mesmo caminho, so' que mais cedo.
    property int esperaMs: 15000

    // Ninguem descobre sozinho que a linha nao foi: quem pediu precisa ouvir.
    signal dropped(string command)

    readonly property Timer prazo: Timer {
        interval: root.esperaMs
        repeat: false
        onTriggered: root.desistir()
    }

    // Guarda a linha e comeca a contar.
    function aguardar(texto) {
        root.command = texto;
        root.prazo.restart();
    }

    // Entrega a linha a quem tem terminal, uma vez so'.
    function retirar() {
        const guardada = root.command;
        root.command = "";
        root.prazo.stop();
        return guardada;
    }

    function desistir() {
        if (root.command === "") {
            return;
        }
        root.dropped(root.retirar());
    }
}
