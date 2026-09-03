pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O PLANO: o que seria escrito no CMake para o alvo usar a biblioteca.
//
// Nada aqui aplica nada. O plano nomeia as Configuration Actions que fariam o
// trabalho, e e o dominio configaction quem escreve — com o preview e o
// consentimento que ele ja tem. Mostrar o plano ANTES de aplicar e o mesmo
// contrato do resto da IDE: o usuario ve o que vai acontecer.
Item {
    id: root

    property var plan: null
    property string errorText: ""
    property bool hasTarget: false
    property bool hasSelection: false

    signal applyStepRequested(string actionId, var params)

    Rectangle {
        anchors.fill: parent
        radius: Theme.radius
        color: Theme.surface1
        border.width: 1
        border.color: Theme.borderSoft

        // Estado vazio que ENSINA em vez de so' ficar em branco.
        Text {
            anchors.centerIn: parent
            anchors.margins: Theme.spacingSmall
            width: parent.width - 2 * Theme.spacingSmall
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.WordWrap
            visible: root.errorText === "" && root.plan === null
            text: !root.hasSelection
                  ? qsTr("Escolha uma biblioteca para ver o que seria escrito.")
                  : (!root.hasTarget
                     ? qsTr("Informe o alvo do CMake que vai linkar a biblioteca.")
                     : qsTr("Calculando…"))
            color: Theme.textDisabled
            font.pixelSize: 10
        }

        Text {
            anchors.fill: parent
            anchors.margins: Theme.spacingSmall
            wrapMode: Text.WordWrap
            visible: root.errorText !== ""
            text: root.errorText
            color: Theme.errorSoft
            font.pixelSize: 10
        }

        Column {
            anchors.fill: parent
            anchors.margins: Theme.spacingSmall
            spacing: 2
            visible: root.errorText === "" && root.plan !== null

            Text {
                width: parent.width
                wrapMode: Text.WordWrap
                // Por que este caminho, e nao o outro. A escolha sai da
                // MEDICAO, e dizer isso evita a pergunta "por que ele baixou
                // se eu ja tenho instalado?".
                text: root.plan && root.plan.usesFindPackage
                      ? qsTr("Está instalada nesta máquina — a IDE vai usá-la, sem baixar nada.")
                      : qsTr("Não foi encontrada no sistema — seria baixada na versão fixada.")
                color: Theme.textMuted
                font.pixelSize: 9
            }

            Repeater {
                model: root.plan ? root.plan.steps : []

                delegate: Item {
                    id: passo

                    required property var modelData

                    width: parent.width
                    height: rotulo.implicitHeight + 4

                    Text {
                        id: rotulo

                        anchors.left: parent.left
                        anchors.right: aplicar.left
                        anchors.rightMargin: Theme.spacingSmall
                        anchors.verticalCenter: parent.verticalCenter
                        wrapMode: Text.WordWrap
                        text: "• " + passo.modelData.summary
                        color: Theme.textPrimary
                        font.pixelSize: 10
                    }

                    // O botao NAO escreve nada: ele entrega a acao ao dominio
                    // configaction, que abre o preview e pede consentimento.
                    // Aplicar sem mostrar o diff seria a IDE mexendo no arquivo
                    // do usuario por conta propria.
                    Rectangle {
                        id: aplicar

                        anchors.right: parent.right
                        anchors.verticalCenter: parent.verticalCenter
                        width: 68
                        height: 16
                        radius: Theme.radiusXSmall
                        color: area.containsMouse ? Theme.surface2 : "transparent"
                        border.width: 1
                        border.color: Theme.borderSoft

                        Text {
                            anchors.centerIn: parent
                            text: qsTr("revisar…")
                            color: Theme.textMuted
                            font.pixelSize: 9
                        }

                        MouseArea {
                            id: area

                            anchors.fill: parent
                            hoverEnabled: true
                            onClicked: root.applyStepRequested(passo.modelData.actionId,
                                                               passo.modelData.params)
                        }
                    }
                }
            }

            // "Nao achei" sem dizer ONDE manda o usuario adivinhar.
            Text {
                width: parent.width
                wrapMode: Text.WordWrap
                visible: root.plan && root.plan.searchedPaths
                         && root.plan.searchedPaths.length > 0
                text: qsTr("Procurei em: ") + (root.plan && root.plan.searchedPaths
                      ? root.plan.searchedPaths.join(", ") : "")
                color: Theme.textDisabled
                font.pixelSize: 9
            }
        }
    }
}
