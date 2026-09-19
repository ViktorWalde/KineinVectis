import QtQuick
import KineinVectis

// O widget de GIT (Etapa 2, F1): a branch, o que esta' a frente/atras e
// quantas alteracoes — o VCS widget da referencia. O clique abre o painel
// Git. Fora de um repositorio, some.
Rectangle {
    id: root

    property string branchLabel: ""
    property int aheadCount: 0
    property int behindCount: 0
    property int changeCount: 0
    property bool panelActive: false

    signal panelRequested()
    // O clique no NOME do branch abre a troca de branch (HUD do Git, 2d).
    signal branchMenuRequested()

    visible: root.branchLabel !== ""
    height: 32
    width: conteudo.implicitWidth + 2 * Theme.spacingMedium
    radius: Theme.radius
    color: root.panelActive ? Theme.surfaceSelected
                            : (area.containsMouse ? Theme.surface2 : "transparent")
    border.color: area.containsMouse || root.panelActive ? Theme.borderSoft : "transparent"
    border.width: 1

    Row {
        id: conteudo

        anchors.centerIn: parent
        spacing: Theme.spacingSmall

        KvIcon {
            anchors.verticalCenter: parent.verticalCenter
            name: "git"
            size: 16
            iconColor: root.changeCount > 0 ? Theme.accent : Theme.textSecondary
        }

        Text {
            id: branchText

            anchors.verticalCenter: parent.verticalCenter
            text: root.branchLabel
            color: branchArea.containsMouse ? Theme.accent : Theme.textPrimary
            font.pixelSize: 12
            font.family: Theme.monoFont
            font.underline: branchArea.containsMouse

            MouseArea {
                id: branchArea

                anchors.fill: parent
                anchors.margins: -2
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: root.branchMenuRequested()
            }
        }

        Text {
            anchors.verticalCenter: parent.verticalCenter
            visible: root.aheadCount > 0
            text: "↑" + root.aheadCount
            color: Theme.textSecondary
            font.pixelSize: 11
        }

        Text {
            anchors.verticalCenter: parent.verticalCenter
            visible: root.behindCount > 0
            text: "↓" + root.behindCount
            color: Theme.textSecondary
            font.pixelSize: 11
        }

        // As alteracoes como um contador com fundo: e' o numero que pede
        // acao (commit), por isso e' o unico com destaque.
        Rectangle {
            anchors.verticalCenter: parent.verticalCenter
            visible: root.changeCount > 0
            width: alteracoes.implicitWidth + Theme.spacingSmall
            height: 16
            radius: 8
            color: Theme.accentDim

            Text {
                id: alteracoes

                anchors.centerIn: parent
                text: root.changeCount
                color: Theme.textPrimary
                font.pixelSize: 10
                font.weight: Font.DemiBold
            }
        }
    }

    MouseArea {
        id: area

        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: root.panelRequested()
    }
}
