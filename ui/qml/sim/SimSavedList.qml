pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// As simulacoes salvas em `.kinein/simulacoes/`: gravar, abrir e esquecer.
//
// O que se grava e' o que o autor MONTOU. O resultado nao entra no arquivo
// (`arquitetura/34` §9), e a linha abaixo do campo diz isso — porque quem salva
// espera reencontrar o que viu, e a IDE precisa ser clara sobre o que guarda.
Column {
    id: root

    property var saved: []
    property string saveName: ""
    property bool canSave: false
    // Por que salvar esta' indisponivel, quando esta'. Vazio quando nao ha'
    // impedimento — botao morto sem explicacao e' o que esta tela nao faz.
    property string note: ""

    signal nameEdited(string texto)
    signal saveRequested()
    signal loadRequested(var salva)
    signal forgetRequested(string nome)

    spacing: Theme.spacingXSmall

    Text {
        text: qsTr("Salvar esta simulação")
        color: Theme.textPrimary
        font.family: Theme.uiFont
        font.pixelSize: Theme.fontSizePanelTitle
        font.bold: true
    }

    Text {
        width: root.width
        wrapMode: Text.WordWrap
        text: qsTr("Guarda o conceito, a fórmula, as ligações, os valores e a escolha "
                   + "numérica — em .kinein/simulacoes/, um arquivo por simulação. "
                   + "O resultado não é gravado: ele se refaz rodando de novo.")
        color: Theme.textMuted
        font.family: Theme.uiFont
        font.pixelSize: Theme.fontSizeStatus
    }

    Text {
        visible: root.note !== ""
        width: root.width
        wrapMode: Text.WordWrap
        text: root.note
        color: Theme.warningSoft
        font.family: Theme.uiFont
        font.pixelSize: Theme.fontSizeStatus
    }

    Row {
        spacing: Theme.spacingSmall

        Rectangle {
            width: 220
            height: 26
            radius: Theme.radiusXSmall
            color: Theme.backgroundEditor
            border.width: 1
            border.color: Theme.borderSoft

            TextInput {
                id: campo

                anchors.fill: parent
                anchors.margins: Theme.spacingXSmall
                verticalAlignment: TextInput.AlignVCenter
                clip: true
                text: root.saveName
                color: Theme.textPrimary
                font.family: Theme.uiFont
                font.pixelSize: Theme.fontSizeStatus
                selectByMouse: true

                onTextEdited: root.nameEdited(campo.text)

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    visible: campo.text === ""
                    text: qsTr("nome desta simulação")
                    color: Theme.textDisabled
                    font.family: Theme.uiFont
                    font.pixelSize: Theme.fontSizeStatus
                }
            }
        }

        Rectangle {
            width: 80
            height: 26
            radius: Theme.radiusXSmall
            color: root.canSave ? Theme.surface2 : Theme.background2
            border.width: 1
            border.color: root.canSave ? Theme.borderStrong : Theme.borderSoft

            Text {
                anchors.centerIn: parent
                text: qsTr("Salvar")
                color: root.canSave ? Theme.textPrimary : Theme.textDisabled
                font.family: Theme.uiFont
                font.pixelSize: Theme.fontSizeStatus
            }

            MouseArea {
                anchors.fill: parent
                enabled: root.canSave
                cursorShape: enabled ? Qt.PointingHandCursor : Qt.ArrowCursor
                onClicked: root.saveRequested()
            }
        }
    }

    Repeater {
        model: root.saved

        delegate: Row {
            id: linha

            required property var modelData

            spacing: Theme.spacingSmall

            Text {
                width: 220
                anchors.verticalCenter: parent.verticalCenter
                elide: Text.ElideRight
                text: linha.modelData.name
                color: Theme.textSecondary
                font.family: Theme.uiFont
                font.pixelSize: Theme.fontSizeStatus
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: qsTr("abrir")
                color: Theme.accent
                font.family: Theme.uiFont
                font.pixelSize: Theme.fontSizeStatus

                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.loadRequested(linha.modelData)
                }
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: qsTr("esquecer")
                color: Theme.textMuted
                font.family: Theme.uiFont
                font.pixelSize: Theme.fontSizeStatus

                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.forgetRequested(linha.modelData.name)
                }
            }
        }
    }
}
