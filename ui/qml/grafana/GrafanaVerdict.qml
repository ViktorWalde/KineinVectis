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
    implicitHeight: Math.max(18, texto.implicitHeight)
    height: implicitHeight

    Text {
        id: texto

        anchors.left: parent.left
        anchors.right: parent.right
        anchors.verticalCenter: parent.verticalCenter
        wrapMode: Text.WordWrap
        font.pixelSize: 10
        color: root.erro !== "" ? Theme.errorSoft
                                : (root.autenticou ? Theme.successSoft
                                                   : (root.alcancou ? Theme.warningSoft
                                                                    : Theme.textMuted))
        text: {
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
}
