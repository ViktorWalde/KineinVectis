import QtQuick
import KineinVectis

// Um cartao de LINGUAGEM no "Criar projeto" (0.3.8; retorno do autor de
// 2026-10-02: a criacao tem de ser mais intuitiva). O icone do tipo de arquivo,
// o nome e, embaixo, o ecossistema — escolher e' clicar no cartao, e o
// escolhido fica com a borda e o fundo de selecao, nao so' com o texto em cor.
Rectangle {
    id: root

    property var language: ({})
    property bool selected: false

    signal chosen()

    implicitWidth: 150
    implicitHeight: 78
    radius: Theme.radiusLarge
    color: selected ? Theme.surfaceSelected : (tileArea.containsMouse ? Theme.surface2 : Theme.background2)
    border.color: selected ? Theme.accent : Theme.borderSoft
    border.width: selected ? 2 : 1

    // Hover e selecao trocam de cor em `motionFast`, nao num salto (0.3.9:
    // fluidez, decisao do autor de 2026-10-01).
    Behavior on color { ColorAnimation { duration: Theme.motionFast; easing.type: Theme.easingStandard } }
    Behavior on border.color { ColorAnimation { duration: Theme.motionFast; easing.type: Theme.easingStandard } }

    KvFileIcon {
        id: tileIcon

        anchors.left: parent.left
        anchors.top: parent.top
        anchors.margins: Theme.spacingMedium
        size: 22
        fileName: root.language.iconFile !== undefined ? root.language.iconFile : ""
        directory: root.language.iconFile === ""
    }

    Text {
        anchors.left: tileIcon.right
        anchors.leftMargin: Theme.spacingSmall
        anchors.right: parent.right
        anchors.rightMargin: Theme.spacingSmall
        anchors.verticalCenter: tileIcon.verticalCenter
        text: root.language.label !== undefined ? root.language.label : ""
        color: root.selected ? Theme.textPrimary : Theme.textSecondary
        font.pixelSize: Theme.fontSizeLarge
        font.weight: Font.DemiBold
        elide: Text.ElideRight
    }

    Text {
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.margins: Theme.spacingMedium
        text: root.language.ecosystems !== undefined
              ? root.language.ecosystems.map(function(e) { return e.label; }).join(" · ") : ""
        color: root.selected ? Theme.accent : Theme.textMuted
        font.pixelSize: Theme.fontSizeCaption
        elide: Text.ElideRight
    }

    MouseArea {
        id: tileArea

        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: root.chosen()
    }
}
