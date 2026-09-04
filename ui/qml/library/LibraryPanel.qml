pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O catalogo de bibliotecas na tela: escolher, ENTENDER e aplicar.
//
// Componente burro: recebe por property, pede por signal. O "entender" nao e
// enfeite — e a razao do painel existir. Quem nao sabe qual biblioteca usar nao
// e ajudado por um inventario, entao cada linha mostra o que a biblioteca FAZ,
// a licenca, a versao PINADA e a data do release. Idade visivel e' informacao:
// uma lib parada ha' tres anos pode ser madura ou abandonada, e quem decide e'
// quem le, nao a IDE.
Item {
    id: root

    property var libraries: []
    property string selectedId: ""
    property string target: ""
    property var targets: []
    property string targetsOrigin: ""
    property var plan: null
    property string errorText: ""

    signal librarySelected(string id)
    signal targetEdited(string name)
    signal closeRequested()
    signal applyStepRequested(string actionId, var params)

    Text {
        id: titulo

        anchors.top: parent.top
        anchors.left: parent.left
        text: qsTr("Bibliotecas C/C++")
        color: Theme.textPrimary
        font.pixelSize: 12
        font.weight: Font.DemiBold
    }

    Text {
        id: subtitulo

        anchors.top: titulo.bottom
        anchors.topMargin: 2
        anchors.left: parent.left
        anchors.right: parent.right
        wrapMode: Text.WordWrap
        text: qsTr("Auditadas: licença verificada na fonte e versão fixada. "
                   + "A IDE não usa gerenciador de pacotes — escreve CMake.")
        color: Theme.textMuted
        font.pixelSize: 10
    }

    LibraryTargetPicker {
        id: alvoBox

        anchors.top: subtitulo.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.left: parent.left
        anchors.right: parent.right

        targets: root.targets
        origin: root.targetsOrigin
        target: root.target

        onTargetEdited: name => root.targetEdited(name)
    }

    ListView {
        id: lista

        anchors.top: alvoBox.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: rodape.top
        anchors.bottomMargin: Theme.spacingSmall
        clip: true
        spacing: 2
        model: root.libraries

        delegate: Rectangle {
            id: linha

            required property var modelData

            width: lista.width
            height: 52
            radius: Theme.radius
            color: linha.modelData.id === root.selectedId ? Theme.surface2 : "transparent"

            MouseArea {
                anchors.fill: parent
                onClicked: root.librarySelected(linha.modelData.id)
            }

            Text {
                id: nome

                anchors.top: parent.top
                anchors.topMargin: 4
                anchors.left: parent.left
                anchors.leftMargin: Theme.spacingSmall
                text: linha.modelData.name
                color: Theme.textPrimary
                font.pixelSize: 11
                font.weight: Font.DemiBold
            }

            // O sinal FORTE, quando existe. Nao ha certificacao oficial de
            // biblioteca C++; o que existe e' o comite ter adotado a lib no
            // padrao. Quando isso aconteceu, e' o que mais importa na linha.
            Text {
                anchors.verticalCenter: nome.verticalCenter
                anchors.left: nome.right
                anchors.leftMargin: Theme.spacingSmall
                visible: linha.modelData.standardLineage !== undefined
                         && linha.modelData.standardLineage !== ""
                text: "★ " + (linha.modelData.standardLineage || "")
                color: Theme.accent
                font.pixelSize: 9
            }

            Text {
                anchors.top: parent.top
                anchors.topMargin: 4
                anchors.right: parent.right
                anchors.rightMargin: Theme.spacingSmall
                text: linha.modelData.status === "detected"
                      ? qsTr("instalada") : qsTr("seria baixada")
                color: linha.modelData.status === "detected"
                       ? Theme.successSoft : Theme.textMuted
                font.pixelSize: 9
            }

            Text {
                id: resumo

                anchors.top: nome.bottom
                anchors.topMargin: 1
                anchors.left: parent.left
                anchors.leftMargin: Theme.spacingSmall
                anchors.right: parent.right
                anchors.rightMargin: Theme.spacingSmall
                elide: Text.ElideRight
                text: linha.modelData.summary
                color: Theme.textMuted
                font.pixelSize: 10
            }

            Text {
                anchors.top: resumo.bottom
                anchors.topMargin: 1
                anchors.left: parent.left
                anchors.leftMargin: Theme.spacingSmall
                text: linha.modelData.license + " · " + linha.modelData.pinnedVersion
                      + " · " + linha.modelData.releasedAt
                color: Theme.textDisabled
                font.pixelSize: 9
            }
        }
    }

    LibraryPlanView {
        id: rodape

        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        height: 96

        plan: root.plan
        errorText: root.errorText
        hasTarget: root.target !== ""
        hasSelection: root.selectedId !== ""

        onApplyStepRequested: function(actionId, params) {
            root.applyStepRequested(actionId, params);
        }
    }
}
