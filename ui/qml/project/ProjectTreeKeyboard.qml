pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O TECLADO DA ARVORE, ligado ao modelo (P1).
//
// Fica separado do `ProjectExplorer` por dois motivos. O painel ja' estava
// perto do limite da catraca de arquitetura — e crescer em cima e' o que ela
// recusa. E este componente tem ESTADO: o prefixo que a pessoa esta'
// digitando e a hora da ultima tecla. Estado com nome proprio e' estado que
// se enxerga.
//
// Ele nao toca no modelo: traduz tecla em INTENCAO (as regras decidem) e
// emite o pedido. Quem muda a arvore continua sendo o controller, pelos
// mesmos sinais que o mouse usa — e' isso que faz "mouse e teclado alcancam
// os mesmos itens" ser verdade, e nao uma promessa com dois caminhos.
Item {
    id: root

    // O `ListModel` achatado do `ProjectTreeController`.
    property var model: null
    // Onde o cursor esta'. -1 e' "em lugar nenhum ainda".
    property int currentIndex: -1

    signal moveRequested(int index)
    signal toggleRequested(int index)
    signal activateRequested(int index)

    visible: false

    ProjectTreeKeyRules {
        id: regras
    }

    // O que a pessoa esta' digitando agora, e quando ela digitou a ultima
    // tecla. Separados do resto porque so' a busca por nome os usa.
    property string prefixo: ""
    property double ultimaTeclaMs: 0

    // O retrato das linhas visiveis, como as regras o esperam.
    //
    // Refeito a cada tecla de proposito: a arvore muda por baixo (uma pasta
    // termina de listar, um arquivo aparece de fora), e guardar uma copia
    // seria navegar por uma arvore que nao existe mais.
    function linhas() {
        const saida = [];
        if (root.model === null) {
            return saida;
        }
        for (let indice = 0; indice < root.model.count; ++indice) {
            const linha = root.model.get(indice);
            saida.push({
                "name": linha.name,
                "kind": linha.kind,
                "depth": linha.depth,
                "expanded": linha.expanded
            });
        }
        return saida;
    }

    function aplicar(intencao) {
        switch (intencao.kind) {
        case "move":
            root.moveRequested(intencao.index);
            return true;
        case "expand":
        case "collapse":
            // UM SINAL SO' PARA OS DOIS: quem sabe o que fazer com uma pasta
            // e' o controller, que ja' recebe o estado atual dela do mouse.
            root.toggleRequested(intencao.index);
            return true;
        case "activate":
            root.activateRequested(intencao.index);
            return true;
        default:
            return false;
        }
    }

    // Devolve `true` quando a tecla foi consumida — quem chama usa isso para
    // nao deixar a tecla seguir para outro dono.
    function handleKey(evento) {
        const linhas = root.linhas();
        const indice = root.currentIndex;
        switch (evento.key) {
        case Qt.Key_Down:
            return root.aplicar(regras.aoDescer(linhas, indice));
        case Qt.Key_Up:
            return root.aplicar(regras.aoSubir(linhas, indice));
        case Qt.Key_Right:
            return root.aplicar(regras.aoAvancar(linhas, indice));
        case Qt.Key_Left:
            return root.aplicar(regras.aoRecuar(linhas, indice));
        case Qt.Key_Home:
            return root.aplicar(regras.aoIrParaOTopo(linhas));
        case Qt.Key_End:
            return root.aplicar(regras.aoIrParaOFim(linhas));
        case Qt.Key_Return:
        case Qt.Key_Enter:
            return root.aplicar(regras.aoAtivar(linhas, indice));
        default:
            return root.procurarPeloNome(evento, linhas, indice);
        }
    }

    // BUSCA PELO NOME DIGITADO. Só entra aqui o que e' texto de verdade: uma
    // tecla com modificador (Ctrl+C, por exemplo) pertence a outro dono, e
    // engoli-la aqui faria a copia parar de funcionar sem ninguem entender.
    function procurarPeloNome(evento, linhas, indice) {
        const texto = String(evento.text);
        if (texto === "" || texto.charCodeAt(0) < 32
            || (evento.modifiers & (Qt.ControlModifier | Qt.AltModifier | Qt.MetaModifier))) {
            return false;
        }
        const agora = Date.now();
        root.prefixo = regras.prefixoAcumulado(root.prefixo, root.ultimaTeclaMs, agora, texto);
        root.ultimaTeclaMs = agora;
        const intencao = regras.aoDigitar(linhas, indice, root.prefixo);
        if (intencao.kind === "none") {
            // NAO ACHOU NAO E' "NAO FIZ NADA": a tecla foi para a busca, e
            // deixa-la seguir faria um atalho disparar no meio de uma palavra.
            return true;
        }
        return root.aplicar(intencao);
    }

    // Sair da arvore apaga o que estava sendo digitado: voltar a ela minutos
    // depois e continuar uma palavra pela metade seria pior que recomecar.
    function esquecerDigitacao() {
        root.prefixo = "";
        root.ultimaTeclaMs = 0;
    }
}
