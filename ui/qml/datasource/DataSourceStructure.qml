pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O que existe DENTRO do banco: esquemas, tabelas e colunas.
//
// POR QUE ESTE ARQUIVO EXISTE (2026-09-04). O painel sabia dizer "conectei" e
// nao sabia dizer "e tem isto aqui dentro" — o que faz dele um testador de
// conexao, nao um cliente de banco. Esta e' a primeira pergunta de quem abre
// um cliente.
//
// A ARVORE E' RASA DE PROPOSITO: esquema > tabela > colunas, com a tabela
// abrindo no clique. Um banco de producao tem milhares de colunas, e desenhar
// todas de uma vez trava a tela antes de mostrar a primeira linha.
//
// VIEW E TABELA sao distinguidas porque a diferenca muda o que se pode fazer:
// dar UPDATE numa view costuma falhar, e descobrir isso no erro do servidor e'
// pior que ver na lista.
Item {
    id: root

    property var schemas: []
    property bool loading: false

    // Chaves "esquema.tabela" abertas. Objeto e nao lista para o teste de
    // pertencimento ser direto na hora de desenhar cada linha.
    property var expanded: ({})

    implicitHeight: coluna.implicitHeight

    function toggle(chave) {
        const atual = Object.assign({}, root.expanded);
        if (atual[chave] === true) {
            delete atual[chave];
        } else {
            atual[chave] = true;
        }
        root.expanded = atual;
    }

    Column {
        id: coluna

        anchors.left: parent.left
        anchors.right: parent.right
        spacing: 2

        Text {
            width: parent.width
            visible: root.loading
            text: qsTr("Lendo a estrutura...")
            color: Theme.textMuted
            font.pixelSize: 10
        }

        Text {
            width: parent.width
            visible: !root.loading && root.schemas.length === 0
            wrapMode: Text.WordWrap
            text: qsTr("Clique em \"Ler estrutura\" para ver esquemas, tabelas e colunas.")
            color: Theme.textMuted
            font.pixelSize: 10
        }

        Repeater {
            model: root.schemas

            delegate: Column {
                id: blocoEsquema

                required property var modelData

                width: coluna.width
                spacing: 2

                Text {
                    text: "▾ " + blocoEsquema.modelData.name
                    color: Theme.textSecondary
                    font.pixelSize: 11
                    font.weight: Font.DemiBold
                }

                Text {
                    visible: blocoEsquema.modelData.tables.length === 0
                    x: Theme.spacingMedium
                    text: qsTr("(vazio)")
                    color: Theme.textMuted
                    font.pixelSize: 10
                }

                Repeater {
                    model: blocoEsquema.modelData.tables

                    delegate: Column {
                        id: blocoTabela

                        required property var modelData

                        readonly property string chave:
                            blocoEsquema.modelData.name + "." + blocoTabela.modelData.name
                        readonly property bool aberta: root.expanded[blocoTabela.chave] === true

                        width: blocoEsquema.width
                        spacing: 1

                        Row {
                            x: Theme.spacingMedium
                            spacing: Theme.spacingXSmall

                            Text {
                                text: blocoTabela.aberta ? "▾" : "▸"
                                color: Theme.textMuted
                                font.pixelSize: 10
                            }

                            Text {
                                text: blocoTabela.modelData.name
                                color: Theme.textPrimary
                                font.pixelSize: 10
                            }

                            Text {
                                text: blocoTabela.modelData.kind === "view"
                                      ? qsTr("view") : qsTr("tabela")
                                color: blocoTabela.modelData.kind === "view"
                                       ? Theme.infoSoft : Theme.textMuted
                                font.pixelSize: 9
                            }

                            Text {
                                text: "· " + blocoTabela.modelData.columns.length
                                      + qsTr(" colunas")
                                color: Theme.textMuted
                                font.pixelSize: 9
                            }
                        }

                        MouseArea {
                            width: blocoTabela.width
                            height: 14
                            y: -14
                            cursorShape: Qt.PointingHandCursor
                            onClicked: root.toggle(blocoTabela.chave)
                        }

                        Repeater {
                            model: blocoTabela.aberta ? blocoTabela.modelData.columns : []

                            delegate: Text {
                                required property var modelData

                                x: 2 * Theme.spacingMedium
                                text: modelData.name + "  " + modelData.dataType
                                      + (modelData.nullable ? "" : "  NOT NULL")
                                color: Theme.textMuted
                                font.family: Theme.monoFont
                                font.pixelSize: 10
                            }
                        }
                    }
                }
            }
        }
    }
}
