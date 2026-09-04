pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// EM QUAL BINARIO a biblioteca entra.
//
// POR QUE ESTE ARQUIVO EXISTE (2026-09-04). Relato de uso do autor: "o SQLite
// eu nao consegui ativar". A causa nao era o catalogo nem o plano — era este
// campo. Ate' hoje ele era uma caixa de texto VAZIA, e enquanto ficasse vazia
// o painel nao mostrava plano nenhum. A IDE cobrava do autor o nome de um alvo
// que esta' escrito no `CMakeLists.txt` do projeto dele.
//
// Agora o core responde os alvos que conhece — do file-api quando houve
// configure, do proprio `CMakeLists.txt` quando nao houve — e aqui eles viram
// botoes. A caixa de texto continua existindo como saida de emergencia, para o
// alvo que o scanner nao consegue ver (nome montado por variavel).
//
// A ORIGEM E' MOSTRADA de proposito: nome lido da fonte e' nome que o autor
// ESCREVEU, nao alvo que o `CMake` confirmou. Esconder essa diferenca faria a
// IDE parecer mais certa do que e'.
Item {
    id: root

    property var targets: []
    property string origin: ""
    property string target: ""

    signal targetEdited(string name)

    implicitHeight: coluna.implicitHeight

    Column {
        id: coluna

        anchors.left: parent.left
        anchors.right: parent.right
        spacing: Theme.spacingXSmall

        Text {
            width: parent.width
            text: root.targets.length > 0
                  ? qsTr("Onde linkar")
                  : qsTr("Onde linkar — nenhum alvo encontrado")
            color: Theme.textMuted
            font.pixelSize: 10
        }

        Flow {
            width: parent.width
            visible: root.targets.length > 0
            spacing: Theme.spacingXSmall

            Repeater {
                model: root.targets

                delegate: KvToggleChip {
                    id: chip

                    required property var modelData

                    // `KvToggleChip` nasceu quadrado (22x22) para rotulo de um
                    // caractere; nome de alvo e' palavra. `TextMetrics` mede
                    // sem desenhar — um Text escondido so' para medir seria um
                    // item a mais no grafo de cena por alvo.
                    width: Math.min(140, medida.width + 2 * Theme.spacingSmall)
                    labelText: chip.modelData.name
                    active: chip.modelData.name === root.target
                    onToggled: root.targetEdited(chip.modelData.name)

                    TextMetrics {
                        id: medida

                        font.family: Theme.monoFont
                        font.pixelSize: 11
                        text: chip.modelData.name
                    }
                }
            }
        }

        Text {
            width: parent.width
            visible: root.origin === "source"
            wrapMode: Text.WordWrap
            text: qsTr("Lidos do CMakeLists.txt — o projeto ainda não foi "
                       + "configurado, então são os nomes que você escreveu.")
            color: Theme.textMuted
            font.pixelSize: 9
        }

        Rectangle {
            id: caixa

            width: parent.width
            height: 22
            radius: Theme.radius
            color: Theme.surface2
            border.width: 1
            border.color: entrada.activeFocus ? Theme.accent : Theme.borderSoft

            TextInput {
                id: entrada

                anchors.fill: parent
                anchors.margins: 4
                verticalAlignment: TextInput.AlignVCenter
                color: Theme.textPrimary
                font.pixelSize: 11
                clip: true
                selectByMouse: true
                text: root.target

                onEditingFinished: root.targetEdited(text.trim())
            }

            Text {
                anchors.fill: parent
                anchors.margins: 4
                verticalAlignment: Text.AlignVCenter
                visible: entrada.text === ""
                text: root.targets.length > 0
                      ? qsTr("ou digite outro alvo")
                      : qsTr("digite o alvo do CMake (ex.: app)")
                color: Theme.textDisabled
                font.pixelSize: 11
            }
        }
    }
}
