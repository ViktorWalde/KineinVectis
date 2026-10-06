pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Fundo rebaixado comum ao editor, à prévia Markdown e ao painel de baixo.
// A bandeja do host usa surface1; o conteúdo fica em background0 com borda
// e sombra interna. Só desenho: margens, foco e input pertencem ao consumidor.
Rectangle {
    id: root

    radius: Theme.radiusLarge
    color: Theme.background0
    border.width: 1
    border.color: Theme.borderSoft

    // A sombra interna: escurece os primeiros pixels abaixo da borda de cima,
    // como se a bandeja projetasse sombra sobre o poco.
    Rectangle {
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: 1
        height: 10
        radius: root.radius - 1
        gradient: Gradient {
            GradientStop { position: 0.0; color: Qt.rgba(0, 0, 0, 0.32) }
            GradientStop { position: 1.0; color: "transparent" }
        }
    }
}
