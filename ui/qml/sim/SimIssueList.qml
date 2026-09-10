pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Os problemas que a checagem achou, traduzidos para frase.
//
// A traducao acontece aqui, e nao no core, porque a frase e' de INTERFACE — e
// ela casa por `kind`, nunca por texto. Casar por texto foi o que o
// `requestFailed` sem `code` obrigou em 2026-09-04, e e' exatamente o que o
// comentario do `SecretRequired` existe para evitar.
Column {
    id: root

    property var issues: []
    property string caveat: ""

    spacing: Theme.spacingXSmall

    function frase(problema) {
        switch (problema.kind) {
        case "parseFailed":
            return problema.message;
        case "missingQuantity":
            return qsTr("Este conceito precisa de %1, e nenhuma variável da sua "
                        + "fórmula foi ligada a ela.").arg(problema.label);
        case "unboundVariable":
            return qsTr("Você não disse o que `%1` é.").arg(problema.variable);
        case "unknownQuantity":
            return qsTr("`%1` foi ligada a algo que este conceito não declara.")
                     .arg(problema.variable);
        case "duplicateQuantity":
            return qsTr("Duas variáveis diferentes dizem ser a mesma grandeza.");
        case "variableNotInFormula":
            return qsTr("`%1` está ligada, mas não aparece na fórmula.")
                     .arg(problema.variable);
        default:
            return qsTr("A fórmula não passou na checagem.");
        }
    }

    Repeater {
        model: root.issues

        delegate: Row {
            required property var modelData

            spacing: Theme.spacingXSmall

            Rectangle {
                width: 3
                height: texto.implicitHeight
                color: Theme.warningSoft
                radius: 1
            }

            Text {
                id: texto

                width: root.width - 3 - Theme.spacingXSmall
                wrapMode: Text.WordWrap
                text: root.frase(parent.modelData)
                color: Theme.textSecondary
                font.family: Theme.uiFont
                font.pixelSize: Theme.fontSizeStatus
            }
        }
    }

    // O AVISO que o autor pediu na tela, e nao em nota de rodape
    // (arquitetura/34 §5.1). Ele aparece quando a checagem PASSA, que e'
    // justamente o momento em que dá para achar que está tudo certo.
    Rectangle {
        visible: root.caveat !== ""
        width: root.width
        height: avisoTexto.implicitHeight + 2 * Theme.spacingSmall
        radius: Theme.radiusXSmall
        color: Theme.background2
        border.width: 1
        border.color: Theme.warningSoft

        Text {
            id: avisoTexto

            anchors.left: parent.left
            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            anchors.margins: Theme.spacingSmall
            wrapMode: Text.WordWrap
            text: root.caveat
            color: Theme.textSecondary
            font.family: Theme.uiFont
            font.pixelSize: Theme.fontSizeStatus
        }
    }
}
