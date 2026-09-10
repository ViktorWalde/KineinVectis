pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A TABELA DE LIGACAO: para cada variavel da formula, o autor diz o que ela e'.
//
// Esta tela existe por uma decisao explicita (arquitetura/34 §5.0): a IDE nao
// casa variavel com grandeza por NOME. Casar `x` da formula com a grandeza
// chamada `x` e' deducao, e ela erra calada — quem chamar de `x` outra coisa
// tem a fisica errada com a tabela parecendo certa.
//
// E' tambem o lugar onde a UNIDADE aparece, o que faz esta tela pagar dois
// pedidos com uma superficie so'.
Column {
    id: root

    property var variables: []
    property var quantities: []
    property var bindings: ({})

    signal bindRequested(string variable, string quantityId)

    spacing: Theme.spacingXSmall

    Text {
        text: qsTr("O que é cada variável")
        color: Theme.textPrimary
        font.family: Theme.uiFont
        font.pixelSize: Theme.fontSizePanelTitle
        font.bold: true
    }

    Text {
        width: root.width
        wrapMode: Text.WordWrap
        text: qsTr("A IDE não adivinha: diga o que cada variável representa. "
                   + "A ordem em que você escreveu a fórmula não decide nada.")
        color: Theme.textMuted
        font.family: Theme.uiFont
        font.pixelSize: Theme.fontSizeStatus
    }

    Item {
        width: 1
        height: Theme.spacingXSmall
    }

    Repeater {
        model: root.variables

        delegate: Row {
            id: linha

            required property string modelData

            spacing: Theme.spacingSmall

            Rectangle {
                width: 76
                height: 26
                radius: Theme.radiusXSmall
                color: Theme.surface2
                border.width: 1
                border.color: root.bindings[linha.modelData] === undefined
                              ? Theme.warningSoft : Theme.borderSoft

                Text {
                    anchors.centerIn: parent
                    text: linha.modelData
                    color: Theme.textPrimary
                    font.family: Theme.monoFont
                    font.pixelSize: Theme.fontSizeStatus
                }
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: "="
                color: Theme.textMuted
                font.family: Theme.uiFont
                font.pixelSize: Theme.fontSizeStatus
            }

            // A escolha. Uma lista de botoes em vez de um ComboBox porque o
            // numero de grandezas de um conceito e' pequeno e mostrar todas de
            // uma vez ensina o conceito — que e' o objetivo declarado do autor.
            Flow {
                width: linha.parent ? Math.max(120, root.width - 130) : 200
                spacing: Theme.spacingXSmall

                Repeater {
                    model: root.quantities

                    delegate: Rectangle {
                        id: opcao

                        required property var modelData

                        readonly property bool escolhida:
                            root.bindings[linha.modelData] === opcao.modelData.id
                        // Uma grandeza ja' usada por OUTRA variavel: a tela
                        // mostra e desabilita, em vez de deixar o autor
                        // descobrir pelo erro do core.
                        readonly property bool tomada: {
                            for (const outra in root.bindings) {
                                if (outra !== linha.modelData
                                        && root.bindings[outra] === opcao.modelData.id) {
                                    return true;
                                }
                            }
                            return false;
                        }

                        width: rotulo.implicitWidth + 2 * Theme.spacingSmall
                        height: 24
                        radius: Theme.radiusXSmall
                        color: opcao.escolhida ? Theme.accent
                                               : (opcao.tomada ? Theme.background2 : Theme.surface1)
                        border.width: 1
                        border.color: opcao.escolhida ? Theme.accentActive : Theme.borderSoft
                        opacity: opcao.tomada && !opcao.escolhida ? 0.45 : 1

                        Text {
                            id: rotulo

                            anchors.centerIn: parent
                            text: opcao.modelData.unit === ""
                                  ? opcao.modelData.label
                                  : opcao.modelData.label + " (" + opcao.modelData.unit + ")"
                            color: opcao.escolhida ? Theme.background0 : Theme.textSecondary
                            font.family: Theme.uiFont
                            font.pixelSize: Theme.fontSizeStatus
                        }

                        MouseArea {
                            anchors.fill: parent
                            enabled: !opcao.tomada || opcao.escolhida
                            cursorShape: enabled ? Qt.PointingHandCursor : Qt.ArrowCursor
                            // Clicar na escolhida DESLIGA: desfazer uma ligacao
                            // tem de ser possivel sem trocar de conceito.
                            onClicked: root.bindRequested(
                                linha.modelData,
                                opcao.escolhida ? "" : opcao.modelData.id)
                        }
                    }
                }
            }
        }
    }
}
