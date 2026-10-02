import QtQuick
import KineinVectis

// Um cartao de ACAO da tela inicial (0.3.8; retorno do autor de 2026-10-02: a
// tela inicial "esta' boa mas da' para melhorar"). Tres botoes pequenos numa
// linha viraram cartoes que dizem para que servem: o icone, o titulo e uma
// linha de explicacao. O principal ("Criar projeto") leva o icone na cor de
// destaque e a borda acende no hover.
Rectangle {
    id: root

    property string iconName: "file"
    property string title: ""
    property string description: ""
    property bool primary: false

    signal activated()

    implicitHeight: 92
    radius: Theme.radiusLarge
    color: tileArea.containsMouse ? Theme.surface2 : Theme.background1
    border.color: tileArea.containsMouse ? (primary ? Theme.accent : Theme.borderStrong)
                                         : (primary ? Theme.accentDim : Theme.borderSoft)
    border.width: 1

    KvIcon {
        id: tileIcon

        anchors.left: parent.left
        anchors.top: parent.top
        anchors.margins: Theme.spacingMedium
        name: root.iconName
        size: 22
        active: root.primary
        iconColor: root.primary ? Theme.accent : Theme.textSecondary
    }

    Text {
        anchors.left: tileIcon.right
        anchors.leftMargin: Theme.spacingSmall
        anchors.right: parent.right
        anchors.rightMargin: Theme.spacingMedium
        anchors.verticalCenter: tileIcon.verticalCenter
        text: root.title
        color: Theme.textPrimary
        font.pixelSize: Theme.fontSizeLarge
        font.weight: Font.DemiBold
        elide: Text.ElideRight
    }

    Text {
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.margins: Theme.spacingMedium
        text: root.description
        color: Theme.textMuted
        font.pixelSize: Theme.fontSizeCaption
        wrapMode: Text.WordWrap
        maximumLineCount: 2
        elide: Text.ElideRight
    }

    MouseArea {
        id: tileArea

        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: root.activated()
    }
}
