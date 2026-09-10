pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// DE ONDE veio o valor exato — a linha que faltava, e cuja ausencia fez a
// coluna mentir.
//
// POR QUE ESTE ARQUIVO EXISTE (2026-09-10). Medido contra o binario real em
// 2026-09-06 (`roadmaps/31` §19.0): com `-(k/m)*x - 2*(c/m)*v` no oscilador
// amortecido a IDE acusava erro de 2,5e-2 numa integracao correta ate' 3,2e-7 —
// setenta e oito mil vezes o que reportava. A coluna vinha da solucao do
// CONCEITO e nao olhava a formula digitada; o checador aprova a formula porque
// confere ligacao, nao fisica.
//
// **O conserto nao foi esconder o numero.** Sem o oraculo na maquina, a solucao
// do conceito continua sendo a melhor resposta disponivel — o que faltava era
// dizer que e' ela. A regra da casa e' que numero sem procedencia mente, e quem
// a estava quebrando era justamente a coluna que existe para dar procedencia.
//
// Ele mora num arquivo so' porque as duas telas — a escalar e a vetorial — dizem
// a MESMA coisa, e a mesma derivacao em dois arquivos diverge em silencio
// (`verificar-qml-duplicacao.sh`).
Column {
    id: root

    // "concept" ou "oracle", como o core os nomeia.
    property string source: ""
    // Quem resolveu, quando foi o oraculo: "SymPy 1.14.0".
    property string solvedBy: ""
    // A solucao fechada que ele devolveu.
    property string closedForm: ""
    // A frase que explica a procedencia, quando ela nao e' a do oraculo.
    property string note: ""

    readonly property bool doOraculo: root.source === "oracle"

    spacing: Theme.spacingXSmall

    // A PROCEDENCIA BOA: a IDE resolveu a equacao que voce escreveu.
    Text {
        visible: root.doOraculo
        width: root.width
        wrapMode: Text.WordWrap
        text: qsTr("Valor exato da SUA equação, resolvido por %1.").arg(root.solvedBy)
        color: Theme.successSoft
        font.family: Theme.uiFont
        font.pixelSize: Theme.fontSizeStatus
    }

    // A RESPOSTA VERDADEIRA ao lado do numero calculado (`arquitetura/34` §7.3,
    // item 2). Ela substitui "confie no raciocinio" por "confira o resultado",
    // que e' o que a engenharia chama de verificacao.
    Rectangle {
        visible: root.doOraculo && root.closedForm !== ""
        width: root.width
        height: formaFechada.implicitHeight + 2 * Theme.spacingSmall
        radius: Theme.radiusXSmall
        color: Theme.backgroundEditor
        border.width: 1
        border.color: Theme.borderSoft

        Text {
            id: formaFechada

            anchors.left: parent.left
            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            anchors.margins: Theme.spacingSmall
            wrapMode: Text.WrapAnywhere
            text: "y(t) = " + root.closedForm
            color: Theme.textSecondary
            font.family: Theme.monoFont
            font.pixelSize: Theme.fontSizeStatus
        }
    }

    // A RESSALVA: o numero e' de outra pergunta, e a tela diz qual e por que.
    Rectangle {
        visible: !root.doOraculo && root.note !== ""
        width: root.width
        height: ressalva.implicitHeight + 2 * Theme.spacingSmall
        radius: Theme.radiusXSmall
        color: Theme.background2
        border.width: 1
        border.color: Theme.warningSoft

        Text {
            id: ressalva

            anchors.left: parent.left
            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            anchors.margins: Theme.spacingSmall
            wrapMode: Text.WordWrap
            text: root.note
            color: Theme.textSecondary
            font.family: Theme.uiFont
            font.pixelSize: Theme.fontSizeStatus
        }
    }
}
