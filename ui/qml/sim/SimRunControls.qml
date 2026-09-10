pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// COMO RESOLVER: metodo, passo, duracao, amostras e estado inicial.
//
// Todo campo comeca VAZIO e nenhum ganha padrao (arquitetura/34 §2.1). A razao
// esta escrita na tela ao lado do metodo, porque ela nao e' obvia: **o metodo
// muda o numero**, e por varias ordens de grandeza.
Column {
    id: root

    property string method: ""
    property string stepText: ""
    property string durationText: ""
    property string samplesText: ""
    property string y0Text: ""
    property string dy0Text: ""
    property bool needsInitialDerivative: false
    property var estimate: null
    // A forma VETORIAL nao tem `y(0)`: o estado inicial dela sao `n` numeros, e
    // eles ficam com quem guarda os componentes.
    property bool showScalarInitial: true
    // O simpletico so' existe onde o conceito DECLARA o pareamento
    // posicao/velocidade (`arquitetura/34` §13.3). Quando nao existe, o botao
    // fica desabilitado e o MOTIVO aparece — recusar dizendo por que e' melhor
    // que aceitar e integrar outra coisa em silencio.
    property string symplecticReason: ""

    signal methodChosen(string id)
    signal fieldEdited(string campo, string texto)

    spacing: Theme.spacingXSmall

    Text {
        text: qsTr("Como resolver")
        color: Theme.textPrimary
        font.family: Theme.uiFont
        font.pixelSize: Theme.fontSizePanelTitle
        font.bold: true
    }

    Text {
        width: root.width
        wrapMode: Text.WordWrap
        text: qsTr("O método muda o resultado, não só o tempo. Medido no oscilador "
                   + "amortecido com dt=0,1: Euler explícito erra por 3,11 onde a "
                   + "resposta é −0,276; RK4 erra por 0,000049.")
        color: Theme.textMuted
        font.family: Theme.uiFont
        font.pixelSize: Theme.fontSizeStatus
    }

    Row {
        spacing: Theme.spacingXSmall

        Repeater {
            model: [
                { id: "euler", label: qsTr("Euler explícito"), nota: qsTr("ordem 1") },
                { id: "eulerSymplectic", label: qsTr("Euler simplético"), nota: qsTr("ordem 1") },
                { id: "rk4", label: qsTr("Runge-Kutta 4"), nota: qsTr("ordem 4") }
            ]

            delegate: Rectangle {
                id: botao

                required property var modelData

                readonly property bool escolhido: root.method === botao.modelData.id
                // Recusado NAO e' o mesmo que nao escolhido: o botao continua
                // na tela, apagado, com o motivo logo abaixo.
                readonly property bool recusado:
                    botao.modelData.id === "eulerSymplectic" && root.symplecticReason !== ""

                width: rotulo.implicitWidth + 2 * Theme.spacingSmall
                height: 30
                radius: Theme.radiusXSmall
                color: botao.escolhido ? Theme.accent : Theme.surface1
                border.width: 1
                border.color: botao.escolhido ? Theme.accentActive : Theme.borderSoft
                opacity: botao.recusado ? 0.45 : 1

                Text {
                    id: rotulo

                    anchors.centerIn: parent
                    text: botao.modelData.label + "  (" + botao.modelData.nota + ")"
                    color: botao.escolhido ? Theme.background0
                                           : (botao.recusado ? Theme.textDisabled
                                                             : Theme.textSecondary)
                    font.family: Theme.uiFont
                    font.pixelSize: Theme.fontSizeStatus
                }

                MouseArea {
                    anchors.fill: parent
                    enabled: !botao.recusado
                    cursorShape: enabled ? Qt.PointingHandCursor : Qt.ArrowCursor
                    onClicked: root.methodChosen(botao.modelData.id)
                }
            }
        }
    }

    Text {
        visible: root.symplecticReason !== ""
        width: root.width
        wrapMode: Text.WordWrap
        text: root.symplecticReason
        color: Theme.warningSoft
        font.family: Theme.uiFont
        font.pixelSize: Theme.fontSizeStatus
    }

    Grid {
        columns: 4
        spacing: Theme.spacingSmall
        verticalItemAlignment: Grid.AlignVCenter

        SimNumberField {
            label: qsTr("passo (dt)")
            text: root.stepText
            onEdited: t => root.fieldEdited("step", t)
        }

        SimNumberField {
            label: qsTr("duração")
            text: root.durationText
            onEdited: t => root.fieldEdited("duration", t)
        }

        SimNumberField {
            label: qsTr("pontos na trilha")
            text: root.samplesText
            onEdited: t => root.fieldEdited("samples", t)
        }

        SimNumberField {
            visible: root.showScalarInitial
            label: qsTr("y(0)")
            text: root.y0Text
            onEdited: t => root.fieldEdited("y0", t)
        }

        SimNumberField {
            visible: root.showScalarInitial && root.needsInitialDerivative
            label: qsTr("y'(0)")
            text: root.dy0Text
            onEdited: t => root.fieldEdited("dy0", t)
        }
    }

    // A ESTIMATIVA, vinda do core. A IDE mostra; quem escolhe e' o autor.
    SimEstimateLine {
        width: root.width
        estimate: root.estimate
    }
}
