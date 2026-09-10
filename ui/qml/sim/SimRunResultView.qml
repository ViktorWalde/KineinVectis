pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O RESULTADO DA CORRIDA: o metodo usado, o erro contra a verdade, e a trilha.
//
// A coluna do ERRO e' o que separa resultado de animacao. Sem ela, uma curva de
// Euler com passo grande e' indistinguivel de fisica — medido, ela erra por
// 3,11 onde a resposta e' -0,276 (`roadmaps/31` §11.1).
//
// E quando NAO ha' solucao fechada a tela DIZ que nao ha', em vez de omitir a
// coluna e deixar parecer que o numero e' exato.
Column {
    id: root

    property var run: null
    property string errorText: ""
    // Por que o valor exato NAO veio da equacao digitada, quando nao veio.
    property string oracleNote: ""

    spacing: Theme.spacingXSmall

    Text {
        text: qsTr("A trajetória")
        color: Theme.textPrimary
        font.family: Theme.uiFont
        font.pixelSize: Theme.fontSizePanelTitle
        font.bold: true
    }

    Rectangle {
        visible: root.errorText !== ""
        width: root.width
        height: erro.implicitHeight + 2 * Theme.spacingSmall
        radius: Theme.radiusXSmall
        color: Theme.background2
        border.width: 1
        border.color: Theme.errorSoft

        Text {
            id: erro

            anchors.left: parent.left
            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            anchors.margins: Theme.spacingSmall
            wrapMode: Text.WordWrap
            text: root.errorText
            color: Theme.textSecondary
            font.family: Theme.uiFont
            font.pixelSize: Theme.fontSizeStatus
        }
    }

    // Numero sem procedencia mente: o metodo e o passo aparecem sempre.
    Text {
        visible: root.run !== null
        text: root.run === null ? "" : root.run.methodLabel
        color: Theme.textSecondary
        font.family: Theme.monoFont
        font.pixelSize: Theme.fontSizeStatus
    }

    // A AMOSTRAGEM VAI A TELA: uma tabela amostrada parece completa.
    Text {
        visible: root.run !== null
        width: root.width
        wrapMode: Text.WordWrap
        text: root.run === null ? "" :
              qsTr("%1 passos; mostrando 1 a cada %2.")
                .arg(root.run.stepsTaken).arg(root.run.sampleEvery)
        color: Theme.textMuted
        font.family: Theme.uiFont
        font.pixelSize: Theme.fontSizeStatus
    }

    // O ORACULO.
    Column {
        visible: root.run !== null && root.run.accuracy !== undefined
                 && root.run.accuracy !== null
        width: root.width
        spacing: 1

        Text {
            text: root.run === null || !root.run.accuracy ? "" :
                  qsTr("valor exato    %1").arg(SimFormat.number(root.run.accuracy.exact))
            color: Theme.textSecondary
            font.family: Theme.monoFont
            font.pixelSize: Theme.fontSizeStatus
        }

        Text {
            text: root.run === null || !root.run.accuracy ? "" :
                  qsTr("calculado      %1").arg(SimFormat.number(root.run.accuracy.numeric))
            color: Theme.textSecondary
            font.family: Theme.monoFont
            font.pixelSize: Theme.fontSizeStatus
        }

        Text {
            text: root.run === null || !root.run.accuracy ? "" :
                  qsTr("erro           %1").arg(SimFormat.number(root.run.accuracy.absoluteError))
            color: Theme.textPrimary
            font.family: Theme.monoFont
            font.pixelSize: Theme.fontSizeStatus
            font.bold: true
        }

        // O RELATIVO, porque so' o absoluto engana nos dois sentidos: `1,474`
        // sobre `83.178` e' uma integracao excelente, e o mesmo `1,474` ao lado
        // de `0,032` seria catastrofe (`roadmaps/31` §19.0).
        Text {
            visible: root.run !== null && root.run.accuracy
                     && root.run.accuracy.relativeError !== undefined
            text: root.run === null || !root.run.accuracy
                  || root.run.accuracy.relativeError === undefined ? "" :
                  qsTr("erro relativo  %1").arg(SimFormat.number(root.run.accuracy.relativeError))
            color: Theme.textSecondary
            font.family: Theme.monoFont
            font.pixelSize: Theme.fontSizeStatus
        }
    }

    // DE ONDE veio o valor exato. Sem esta linha, a coluna acima afirma o que a
    // IDE nao sabe — foi o defeito medido em 2026-09-06.
    SimAccuracyProvenance {
        visible: root.run !== null
        width: root.width
        source: root.run === null || !root.run.accuracy ? "" : root.run.accuracy.source
        solvedBy: root.run === null || !root.run.accuracy
                  || root.run.accuracy.solvedBy === undefined
                  ? "" : root.run.accuracy.solvedBy
        closedForm: root.run === null || !root.run.accuracy
                    || root.run.accuracy.closedForm === undefined
                    ? "" : root.run.accuracy.closedForm
        note: root.oracleNote
    }

    // Sem solucao fechada, a IDE DIZ que nao tem com que comparar.
    Text {
        visible: root.run !== null && (root.run.accuracy === undefined
                                       || root.run.accuracy === null)
        width: root.width
        wrapMode: Text.WordWrap
        text: qsTr("Este conceito não tem solução fechada conhecida, então a IDE não "
                   + "sabe dizer o quanto este número erra.")
        color: Theme.warningSoft
        font.family: Theme.uiFont
        font.pixelSize: Theme.fontSizeStatus
    }

    // O GRAFICO. Vem antes da tabela porque "aprender e visualizar" foi o
    // objetivo declarado do autor: a curva ensina o que a coluna de numeros
    // confirma.
    SimPlot2d {
        visible: root.run !== null
        width: root.width
        height: 200
        trail: root.run === null ? [] : root.run.trail
    }

    // A trilha em numeros, para conferir.
    Rectangle {
        visible: root.run !== null
        width: root.width
        height: Math.min(180, Math.max(40, lista.contentHeight + 2))
        radius: Theme.radiusXSmall
        color: Theme.backgroundEditor
        border.width: 1
        border.color: Theme.borderSoft

        ListView {
            id: lista

            anchors.fill: parent
            anchors.margins: Theme.spacingXSmall
            clip: true
            model: root.run === null ? [] : root.run.trail

            delegate: Text {
                required property var modelData

                text: "t=" + SimFormat.number(modelData.t) + "   y=" + SimFormat.number(modelData.y)
                      + (modelData.dy === undefined || modelData.dy === null
                         ? "" : "   y'=" + SimFormat.number(modelData.dy))
                color: Theme.textSecondary
                font.family: Theme.monoFont
                font.pixelSize: Theme.fontSizeStatus - 1
            }
        }
    }
}
