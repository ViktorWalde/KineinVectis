import QtQuick
import KineinVectis

// O POCO do painel de baixo (2026-10-04, pedido do autor: "acrescentar um
// fundo e melhorar a separacao visual do painel inferior ... um relevo").
//
// Antes, editor, painel e terminal tinham a MESMA cor (#191a1c): o painel
// so' se distinguia do editor por uma linha. Agora sao tres planos:
//
//   editor          background1   o que se edita
//   painel (bandeja) surface1     um degrau acima: abas e sessoes
//   poco (conteudo)  background0  um degrau ABAIXO, com a sombra interna no
//                                 topo: a saida fica "dentro" da bandeja
//
// Todas as abas (Terminal, Problemas, Jobs, Build...) desenham sobre ele, entao
// a separacao e' a mesma em qualquer uma.
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
