import QtQuick
import KineinProva

Window {
    id: root

    width: 1280
    height: 800
    visible: true
    color: "#1e1f22"
    title: "Kinein Vectis — prova do Neovim"

    NvimView {
        id: editor

        objectName: "editor"
        anchors.fill: parent
        focus: true
        socketPath: supervisor.socketPath
        Accessible.role: Accessible.EditableText
        Accessible.name: "Editor"
        Accessible.focusable: true
    }

    // A lista de completion do Neovim (ext_popupmenu), com o visual da Kinein.
    Rectangle {
        id: popup

        visible: editor.popupVisible && editor.popupItems.length > 0
        x: Math.min(editor.popupPos.x, root.width - width)
        y: editor.popupPos.y
        width: 420
        height: Math.min(list.contentHeight, 10 * editor.cellHeight) + 2
        color: "#2b2d30"
        border.color: "#43454a"
        radius: 4

        ListView {
            id: list

            anchors.fill: parent
            anchors.margins: 1
            clip: true
            model: editor.popupItems
            currentIndex: editor.popupSelected
            delegate: Rectangle {
                required property var modelData
                required property int index

                width: ListView.view.width
                height: editor.cellHeight
                color: index === editor.popupSelected ? "#2e436e" : "transparent"

                Text {
                    x: 8
                    anchors.verticalCenter: parent.verticalCenter
                    text: modelData.word
                    color: "#dfe1e5"
                    font.family: "monospace"
                }
                Text {
                    anchors.right: parent.right
                    anchors.rightMargin: 8
                    anchors.verticalCenter: parent.verticalCenter
                    text: modelData.kind
                    color: "#868a91"
                    font.family: "monospace"
                }
            }
        }
    }

    Rectangle {
        visible: supervisor.error !== ""
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        height: 48
        color: "#5c2020"

        Text {
            anchors.fill: parent
            anchors.margins: 8
            text: supervisor.error
            color: "#ffffff"
            elide: Text.ElideRight
        }
    }
}
