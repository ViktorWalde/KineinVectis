pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O veredito da sonda, em UMA linha com a cor certa.
//
// DOIS BOOLEANOS, TRES ESTADOS. `reachable` e `authenticated` respondem
// perguntas diferentes, e juntar as duas num "ok" colapsaria "nao ha' Grafana
// nesse endereco" com "o Grafana esta' la' e recusou seu token" — duas frases
// que pedem acoes opostas.
Item {
    id: root

    property var controller: null

    readonly property bool alcancou: root.controller ? root.controller.reachable : false
    readonly property bool autenticou: root.controller ? root.controller.authenticated : false
    readonly property string erro: root.controller ? root.controller.errorText : ""
    readonly property string recado: root.controller ? root.controller.message : ""

    // Medido pelo conteudo, com piso: a frase do servidor pode ter duas
    // linhas, e cortar a segunda esconderia justamente o que fazer.
    // A MESMA FAIXA dos outros paineis de ambiente (KvVerdict): cinza
    // enquanto mede, verde quando autenticou, cinza-neutro quando alcancou
    // sem credencial, vermelha quando falhou. Eu tinha escrito aqui um
    // Text com a minha propria tabela de cores — a mesma decisao em dois
    // lugares, que e' como `severity` ficou azul num painel e vermelha no
    // outro.
    //
    // O que continua sendo DESTE arquivo e' a frase: juntar versao, banco e
    // recado sem anunciar pedaco que nao existe.
    implicitHeight: faixa.implicitHeight
    height: implicitHeight

    KvVerdict {
        id: faixa

        width: parent.width
        // SEM `busy` DE PROPOSITO: quem diz que esta' consultando e' o
        // subtitulo do cabecalho, que ainda nomeia o endereco. A faixa segue
        // mostrando a ULTIMA medida enquanto a nova nao chega — e o eixo
        // `content` ja' avisa quando o que esta' na tela e' de antes.
        ok: root.erro === "" && root.autenticou
        // ALCANCOU SEM CREDENCIAL NAO E' FALHA: e' o que se ve de fora, e o
        // que se ve de fora tem valor. Faixa neutra, nem verde nem vermelha.
        neutral: root.erro === "" && !root.autenticou && root.alcancou
        message: root.frase
    }

    readonly property string frase: {
        if (root.erro !== "") {
            return root.erro;
        }
        if (!root.controller || (!root.alcancou && root.recado === "")) {
            return "";
        }
        if (!root.alcancou) {
            return root.recado;
        }
        // NOME SEM VALOR E' RUIDO: o `database` e' opcional na resposta
        // do Grafana, e a frase fixa imprimia "banco " sozinho, anunciando
        // um dado que a tela nao tem. Cada pedaco so' entra com conteudo.
        // Mesma regra para a versao: alcancar sem saber a versao e'
        // possivel, e "Grafana " sozinho nao diz nada.
        let cabeca = root.controller.version !== ""
                   ? qsTr("Grafana %1").arg(root.controller.version)
                   : qsTr("Grafana respondeu");
        if (root.controller.database !== "") {
            cabeca += qsTr(" · banco %1").arg(root.controller.database);
        }
        return root.recado !== "" ? cabeca + " — " + root.recado : cabeca;
    }
}
