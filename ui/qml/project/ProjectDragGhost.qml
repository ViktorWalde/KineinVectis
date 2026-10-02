import QtQuick
import KineinVectis

// O que vai na mao durante o arrasto na arvore (0.3.9, pedido do autor:
// "ter uma visualizacao melhor de que ele ta arrastando uma pasta ou arquivo
// do projeto para outro lugar"): o icone e o nome do item, e "+N" quando sao
// varios. A linha captura isto com grabToImage e entrega ao arrasto do
// sistema (Drag.imageSource) — o cursor leva a pilula ate' o destino.
Rectangle {
    id: root

    property string name: ""
    property bool directory: false
    property int count: 1

    width: content.implicitWidth + 2 * Theme.spacingMedium
    height: 28
    radius: height / 2
    color: Theme.surface2
    border.color: Theme.accent
    border.width: 1

    Row {
        id: content

        anchors.centerIn: parent
        spacing: Theme.spacingSmall

        KvFileIcon {
            anchors.verticalCenter: parent.verticalCenter
            size: 16
            fileName: root.name
            directory: root.directory
        }

        Text {
            anchors.verticalCenter: parent.verticalCenter
            text: root.name
            color: Theme.textPrimary
            font.pixelSize: Theme.fontSizeBody
            font.bold: true
        }

        Rectangle {
            anchors.verticalCenter: parent.verticalCenter
            visible: root.count > 1
            width: moreText.implicitWidth + Theme.spacingSmall
            height: 16
            radius: height / 2
            color: Theme.accent

            Text {
                id: moreText

                anchors.centerIn: parent
                text: "+" + (root.count - 1)
                color: Theme.background0
                font.pixelSize: Theme.fontSizeCaption
                font.bold: true
            }
        }
    }
}
