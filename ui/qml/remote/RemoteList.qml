pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A coluna da esquerda: os alvos SSH salvos neste workspace, cada um com o
// destino (`usuario@host:porta`) — e' o que o autor confere antes de sondar.
Item {
    id: root

    property var targets: []
    property string selectedName: ""

    signal targetSelected(string name)
    signal newRequested()

    Text {
        id: titulo

        anchors.top: parent.top
        anchors.left: parent.left
        text: qsTr("Alvos")
        color: Theme.textMuted
        font.pixelSize: 10
    }

    ListView {
        id: lista

        anchors.top: titulo.bottom
        anchors.topMargin: 4
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: novo.top
        anchors.bottomMargin: Theme.spacingSmall
        clip: true
        model: root.targets
        spacing: 2

        delegate: Rectangle {
            id: linha

            required property int index
            required property var modelData

            width: ListView.view.width
            height: 32
            radius: Theme.radius
            color: linha.modelData.name === root.selectedName
                   ? Theme.surfaceSelected
                   : (area.containsMouse ? Theme.surface2 : "transparent")

            Column {
                anchors.verticalCenter: parent.verticalCenter
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.leftMargin: Theme.spacingSmall
                anchors.rightMargin: Theme.spacingSmall

                Text {
                    text: linha.modelData.name
                    color: Theme.textPrimary
                    font.pixelSize: 11
                    elide: Text.ElideRight
                    width: parent.width
                }

                Text {
                    text: (linha.modelData.user ? linha.modelData.user + "@" : "")
                          + linha.modelData.host
                          + (linha.modelData.port ? ":" + linha.modelData.port : "")
                    color: Theme.textMuted
                    font.family: Theme.monoFont
                    font.pixelSize: 9
                    elide: Text.ElideMiddle
                    width: parent.width
                }
            }

            MouseArea {
                id: area

                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: root.targetSelected(linha.modelData.name)
            }
        }
    }

    Text {
        anchors.centerIn: lista
        visible: root.targets.length === 0
        width: lista.width
        horizontalAlignment: Text.AlignHCenter
        wrapMode: Text.WordWrap
        text: qsTr("Nenhum alvo salvo.\nPreencha ao lado e salve.")
        color: Theme.textMuted
        font.pixelSize: 10
    }

    KvButton {
        id: novo

        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        compact: true
        text: qsTr("Novo alvo")
        onClicked: root.newRequested()
    }
}
