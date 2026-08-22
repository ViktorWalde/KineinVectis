pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Painel "Contexto de compilação" (L3 fatia 1, §10.2 do roadmap do motor
// semântico). Responde, para o arquivo aberto: como ele é compilado de
// verdade, e com que confiança nós sabemos disso.
//
// A linha de ORIGEM vem primeiro e colorida de propósito. Ela é a única parte
// que não existe em nenhuma outra IDE deste porte: dizer que o contexto de um
// header foi emprestado, e de quem, em vez de apresentar palpite como fato.
Item {
    id: panel

    property var context: null

    Text {
        id: semArquivo

        anchors.centerIn: parent
        visible: panel.context === null || !panel.context.hasContext
        text: qsTr("Abra um arquivo do projeto para ver como ele é compilado.")
        color: Theme.textMuted
        font.pixelSize: 11
    }

    Column {
        id: corpo

        anchors.fill: parent
        visible: panel.context !== null && panel.context.hasContext
        spacing: Theme.spacingXSmall

        Row {
            width: corpo.width
            spacing: Theme.spacingSmall

            Text {
                text: panel.context !== null ? panel.context.path : ""
                color: Theme.textPrimary
                font.family: Theme.monoFont
                font.pixelSize: 11
                font.bold: true
                elide: Text.ElideLeft
                width: Math.min(implicitWidth, corpo.width * 0.5)
            }

            Rectangle {
                anchors.verticalCenter: parent.verticalCenter
                width: 7
                height: 7
                radius: 4
                color: panel.context !== null ? panel.context.originColor(Theme) : Theme.textMuted
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: panel.context !== null ? panel.context.originLabel : ""
                color: panel.context !== null ? panel.context.originColor(Theme) : Theme.textMuted
                font.pixelSize: 11
                elide: Text.ElideRight
                width: corpo.width - 200
            }
        }

        // A nota só aparece quando NÃO há contexto, e ela diz o gesto que
        // falta — configurar o build, por exemplo — em vez de só constatar.
        Text {
            width: corpo.width
            visible: panel.context !== null && panel.context.note !== ""
            text: panel.context !== null ? panel.context.note : ""
            color: Theme.textSecondary
            font.pixelSize: 11
            wrapMode: Text.WordWrap
        }

        Grid {
            columns: 2
            columnSpacing: Theme.spacingMedium
            rowSpacing: 2
            visible: panel.context !== null && panel.context.compiler !== ""

            Text {
                text: qsTr("Compilador")
                color: Theme.textMuted
                font.pixelSize: 11
            }
            Text {
                text: panel.context !== null ? panel.context.compiler : ""
                color: Theme.textSecondary
                font.family: Theme.monoFont
                font.pixelSize: 11
            }

            Text {
                text: qsTr("Padrão")
                color: Theme.textMuted
                font.pixelSize: 11
            }
            Text {
                text: panel.context !== null && panel.context.standard !== ""
                      ? panel.context.standard : qsTr("não declarado no comando")
                color: Theme.textSecondary
                font.family: Theme.monoFont
                font.pixelSize: 11
            }

            Text {
                text: qsTr("Defines")
                color: Theme.textMuted
                font.pixelSize: 11
            }
            Text {
                text: panel.context !== null ? String(panel.context.defines.length) : "0"
                color: Theme.textSecondary
                font.family: Theme.monoFont
                font.pixelSize: 11
            }

            Text {
                text: qsTr("Includes")
                color: Theme.textMuted
                font.pixelSize: 11
            }
            Text {
                text: panel.context !== null ? String(panel.context.includes.length) : "0"
                color: Theme.textSecondary
                font.family: Theme.monoFont
                font.pixelSize: 11
            }

            Text {
                text: qsTr("Banco de comandos")
                color: Theme.textMuted
                font.pixelSize: 11
            }
            Text {
                text: panel.context !== null ? panel.context.database : ""
                color: Theme.textSecondary
                font.family: Theme.monoFont
                font.pixelSize: 11
                elide: Text.ElideLeft
                width: Math.min(implicitWidth, corpo.width * 0.6)
            }
        }

        // O comando inteiro, selecionável: o §10.2 pede que o usuário possa
        // copiá-lo e reexecutá-lo fora da IDE. Sem isso a transparência para
        // na metade — ele veria o resumo e teria que confiar nele.
        TextEdit {
            width: corpo.width
            height: Math.min(contentHeight, 64)
            visible: panel.context !== null && panel.context.command !== ""
            text: panel.context !== null ? panel.context.command : ""
            color: Theme.textMuted
            font.family: Theme.monoFont
            font.pixelSize: 10
            wrapMode: TextEdit.WrapAnywhere
            readOnly: true
            selectByMouse: true
        }
    }
}
