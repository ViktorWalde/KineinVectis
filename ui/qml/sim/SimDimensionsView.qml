pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// AS UNIDADES: o que a IDE conferiu na equacao que voce escreveu.
//
// **Por que ela existe** (decisao do autor, 2026-09-05; entregue em
// 2026-09-10). A decisao original desta etapa era rotulo SEM checagem, tomada
// sobre a medicao de que o `uom` checa em tempo de COMPILACAO e uma formula
// digitada nao tem tipo Rust nenhum. O `SymPy` — que entrou por outro motivo,
// como oraculo — checa em tempo de EXECUCAO, que e' quando a formula existe.
//
// **O limite vem NA TELA junto com o recurso, e ele nao e' pequeno:** checagem
// dimensional pega INCOERENCIA, nunca pega formula errada. Medido em
// 2026-09-10: `E = m*v^2` sem o meio PASSA, porque coerencia dimensional nao
// ve' constante adimensional. Isso nao enfraquece o aviso de que conceito certo
// e formula valida nao significam resultado certo — reforca.
//
// Um dono so' para as duas formas: a escalar tem um veredito, a vetorial tem
// `n`, e a mesma derivacao em dois arquivos diverge em silencio
// (`verificar-qml-duplicacao.sh`).
Column {
    id: root

    // Um veredito por equacao. Vazio = a IDE nao conferiu.
    property var checks: []
    // Por que nao conferiu, quando nao conferiu.
    property string note: ""

    // A frase de cada veredito. Ela casa por `verdict`, nunca por texto — a
    // mesma regra do `SimIssueList`.
    function frase(item) {
        switch (item.verdict) {
        case "coherent":
            return qsTr("as unidades fecham: os termos combinam entre si e com o "
                        + "lado esquerdo da equação.");
        case "incoherent":
            return qsTr("dois termos da soma têm unidades diferentes — não dá para "
                        + "somá-los (%1).").arg(item.detail);
        case "wrongSide":
            return qsTr("os termos combinam entre si, mas a equação não é da grandeza "
                        + "do lado esquerdo (%1).").arg(item.detail);
        case "dimensionalArgument":
            return qsTr("`%1` está recebendo algo com unidade — seno, cosseno, "
                        + "exponencial e logaritmo só aceitam número puro.")
                     .arg(item.detail);
        case "unknownUnit":
            return qsTr("a IDE não conhece uma das unidades declaradas (%1), então "
                        + "preferiu não afirmar nada.").arg(item.detail);
        default:
            return qsTr("a IDE não conseguiu ler esta equação para conferir as unidades.");
        }
    }

    function coerente(item) {
        return item.verdict === "coherent";
    }

    spacing: Theme.spacingXSmall

    Text {
        visible: root.checks.length > 0 || root.note !== ""
        text: qsTr("As unidades")
        color: Theme.textPrimary
        font.family: Theme.uiFont
        font.pixelSize: Theme.fontSizePanelTitle
        font.bold: true
    }

    Repeater {
        model: root.checks

        delegate: Row {
            id: linha

            required property var modelData

            width: root.width
            spacing: Theme.spacingXSmall

            Rectangle {
                width: 3
                height: texto.implicitHeight
                radius: 1
                color: root.coerente(linha.modelData) ? Theme.successSoft : Theme.warningSoft
            }

            Text {
                id: texto

                width: root.width - 3 - Theme.spacingXSmall
                wrapMode: Text.WordWrap
                // O rotulo so' aparece na forma vetorial, onde ha' `n` equacoes
                // e "qual delas" e' metade da informacao.
                text: (linha.modelData.equation === "" ? "" : linha.modelData.equation + ": ")
                      + root.frase(linha.modelData)
                color: Theme.textSecondary
                font.family: Theme.uiFont
                font.pixelSize: Theme.fontSizeStatus
            }
        }
    }

    // O LIMITE, dito junto com o recurso. Sem esta linha o "as unidades fecham"
    // vira promessa de que a fisica esta certa, que e' o que ele NAO e'.
    Text {
        visible: root.checks.length > 0
        width: root.width
        wrapMode: Text.WordWrap
        text: qsTr("Unidade que fecha não quer dizer física certa: um coeficiente "
                   + "errado passa por aqui — `E = m·v²` sem o meio tem a unidade "
                   + "certa e o valor errado.")
        color: Theme.textMuted
        font.family: Theme.uiFont
        font.pixelSize: Theme.fontSizeStatus - 1
    }

    // NAO conferiu — e diz por que, em vez de parar de checar em silencio.
    Text {
        visible: root.checks.length === 0 && root.note !== ""
        width: root.width
        wrapMode: Text.WordWrap
        text: qsTr("As unidades não foram verificadas nesta sessão: a IDE precisa do "
                   + "SymPy no Python desta máquina para conferi-las.")
        color: Theme.textMuted
        font.family: Theme.uiFont
        font.pixelSize: Theme.fontSizeStatus
    }
}
