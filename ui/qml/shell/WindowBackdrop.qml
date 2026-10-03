import QtQuick
import KineinVectis

// O FUNDO da janela: a moldura inteira numa cor so' (0.3.9, pedido do autor:
// "juntar numa ilha as bordas todas e deixar a parte central"). Topo, trilho
// e barra de status ficam transparentes sobre ele — uma moldura continua, sem
// faixas. O veu ambar e' um brilho no canto de cima a' esquerda que esmaece
// para a direita e para baixo, cobrindo o topo e o comeco do trilho.
Rectangle {
    id: root

    color: Theme.frame

    Item {
        width: root.width * 0.4
        height: 280

        Rectangle {
            anchors.fill: parent
            gradient: Gradient {
                orientation: Gradient.Horizontal
                GradientStop { position: 0.0; color: Theme.frameAccentTint }
                GradientStop { position: 1.0; color: Theme.frame }
            }
        }

        Rectangle {
            anchors.fill: parent
            gradient: Gradient {
                GradientStop { position: 0.0; color: "transparent" }
                GradientStop { position: 1.0; color: Theme.frame }
            }
        }
    }
}
