import QtQuick
import KineinVectis

// A MOLDURA comum dos paineis de ambiente (Etapa 2 F8): centralizada,
// dispensa por clique fora (nao por clique dentro), e o conteudo ROLA
// quando nao cabe — o Embarcados vazava da moldura a 800 px (foto 12c).
//
// Dois modos: `contentHeight` 0 (padrao) e' o painel que preenche a
// moldura e rola por dentro (banco, remoto, containers); `contentHeight`
// > 0 e' a altura que o painel pede, e a moldura rola o que passar.
Item {
    id: root

    default property alias content: conteudo.data
    property real panelWidth: 720
    property real panelHeight: 560
    property real maxAvailableWidth: 720
    property real maxAvailableHeight: 520
    property real contentHeight: 0

    signal dismissRequested()

    // ESC FECHA — nos cinco paineis de ambiente de uma vez. A moldura sempre
    // soube dispensar por clique fora, e nunca ouviu o teclado: quem abriu o
    // painel por `Ctrl+Alt+O` tinha de ir ao mouse para sair dele. Os dialogos
    // do projeto ja' faziam isto (`SettingsDialog`, `GitDiscardDialog`); os
    // paineis, nao.
    //
    // `focus: true` e' o que faz a tecla chegar aqui: sem ele o item existe,
    // aparece e nao ouve nada.
    focus: visible
    Keys.onEscapePressed: evento => {
        root.dismissFromKeyboard();
        evento.accepted = true;
    }

    // A DECISAO TEM NOME para poder ser medida: o harness do projeto roda com
    // o `qml` puro e nao entrega tecla a ninguem (isso e' do `qmltestrunner`,
    // que o projeto nao usa). O que fica sem prova automatica e' a LINHA de
    // cima — o `Keys.onEscapePressed` chegar aqui —, e ela foi conferida a
    // mao no binario. O que este nome protege e' o resto: que Esc dispensa, e
    // que dispensar e' o mesmo caminho do clique fora.
    function dismissFromKeyboard() {
        root.dismissRequested();
    }

    readonly property real frameWidth: Math.min(panelWidth, maxAvailableWidth)
    readonly property real frameHeight: Math.min(panelHeight, maxAvailableHeight)

    MouseArea {
        anchors.fill: parent
        onClicked: root.dismissRequested()
    }

    Rectangle {
        anchors.centerIn: parent
        width: root.frameWidth
        height: root.frameHeight
        radius: Theme.radius
        color: Theme.background1
        border.width: 1
        border.color: Theme.borderStrong

        MouseArea {
            anchors.fill: parent
        }

        Flickable {
            id: rolagem

            anchors.fill: parent
            anchors.margins: Theme.spacingMedium
            clip: true
            contentWidth: width
            contentHeight: conteudo.height
            interactive: root.contentHeight > height
            boundsBehavior: Flickable.StopAtBounds
            flickableDirection: Flickable.VerticalFlick

            Item {
                id: conteudo

                width: rolagem.width
                height: Math.max(rolagem.height, root.contentHeight)
            }
        }
    }
}
