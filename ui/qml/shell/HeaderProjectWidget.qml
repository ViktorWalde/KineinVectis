import QtQuick
import KineinVectis

// O widget de PROJETO (Etapa 2, F1 do roadmaps/43): nome do workspace + o
// que ele e' (Cargo + CMake), num so' controle; o clique abre recentes,
// abrir e fechar. A referencia (New UI da JetBrains) poe exatamente isto
// no canto esquerdo da barra — e mais nada sobre o projeto. Burro: mostra
// o que recebe e pede por sinal.
Rectangle {
    id: root

    property bool workspaceOpen: false
    property string workspaceName: ""
    property string systemLabel: ""
    property bool coreConnected: false
    property bool menuOpen: false

    signal menuRequested(real menuX, real menuY)
    signal openWorkspaceRequested()

    height: 32
    width: conteudo.implicitWidth + 2 * Theme.spacingMedium
    radius: Theme.radius
    color: root.menuOpen ? Theme.surfaceSelected
                         : (area.containsMouse ? Theme.surface2 : "transparent")
    border.color: area.containsMouse || root.menuOpen ? Theme.borderSoft : "transparent"
    border.width: 1

    Row {
        id: conteudo

        anchors.centerIn: parent
        spacing: Theme.spacingSmall

        KvIcon {
            anchors.verticalCenter: parent.verticalCenter
            name: "project"
            size: 16
            iconColor: root.workspaceOpen ? Theme.accent : Theme.textSecondary
        }

        Text {
            anchors.verticalCenter: parent.verticalCenter
            text: root.workspaceOpen ? root.workspaceName : qsTr("Abrir workspace")
            color: Theme.textPrimary
            font.pixelSize: 12
            font.weight: Font.DemiBold
        }

        // O que o projeto e', em cinza: informacao, nao botao.
        Text {
            anchors.verticalCenter: parent.verticalCenter
            visible: root.workspaceOpen && root.systemLabel !== ""
            text: root.systemLabel
            color: Theme.textMuted
            font.pixelSize: 11
        }

        // O ponto do core: verde conectado, vermelho nao — o mesmo que a
        // barra de status diz por extenso.
        Rectangle {
            anchors.verticalCenter: parent.verticalCenter
            visible: root.workspaceOpen
            width: 6
            height: 6
            radius: 3
            color: root.coreConnected ? Theme.successSoft : Theme.errorSoft
        }

        KvIcon {
            anchors.verticalCenter: parent.verticalCenter
            visible: root.workspaceOpen
            name: "chevron-down"
            size: 12
            iconColor: Theme.textMuted
        }
    }

    MouseArea {
        id: area

        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: {
            if (!root.workspaceOpen) {
                root.openWorkspaceRequested();
                return;
            }
            root.menuRequested(0, root.height + 4);
        }
    }
}
