import QtQuick
import KineinVectis

// Botao de LIGA/DESLIGA com rotulo curto ("Aa", "W", ".*") — ou, com
// `switchStyle`, um INTERRUPTOR de preferencia (rotulo + trilho com bolinha).
//
// POR QUE ESTE ARQUIVO EXISTE (2026-09-03). Era um `component ToggleButton`
// declarado DENTRO do EditorFindBar. Desenho de botao nao e' responsabilidade
// da barra de busca, e enquanto ficou la' nenhuma outra tela pode reusa-lo — a
// definicao inline e' visivel so' de dentro do arquivo que a declara.
//
// Ativo e' mostrado por TRES sinais ao mesmo tempo (fundo, borda e peso do
// texto) de proposito: so' cor nao sobrevive a daltonismo nem a tema claro.
//
// O TAMANHO E' DELE, e isto foi corrigido em 2026-09-04. Ele nasceu quadrado
// (22x22) para rotulo de UM caractere ("Aa", ".*"), e quando passou a receber
// palavra — nome de alvo, "Rust · Cargo", "PRIVATE" — cada chamador
// recalculava a largura com um `TextMetrics` proprio. Agora o chip mede o
// proprio rotulo; quem quiser outro tamanho ainda pode fixar `width`.
//
// INTERATIVO (2026-10-03, pedido do autor: os interruptores "Animacao" e
// "parados" estavam "ultrapassados" ao lado do resto da IDE): todo chip acende
// ao pairar e responde ao pressionar, com transicao curta; e o liga/desliga
// de PREFERENCIA usa `switchStyle` — o trilho diz "ligado/desligado" sem ler
// o texto:
//
//   chip       [ Aa ]                 opcao curta (busca, escolha)
//   switch     ( ✦ Animação  ◯━━ )    preferencia; ligado: trilho ambar,
//                                     bolinha a direita
Rectangle {
    id: root

    property string labelText: ""
    property bool active: false
    property string tooltip: ""
    // Opcao de uma escolha (a linguagem do "Criar Projeto", 2026-10-01): a
    // inativa tambem tem borda, para parecer clicavel ao lado da ativa; e o
    // rotulo e' texto de interface, nao codigo. O padrao e' o de sempre.
    property bool outlined: false
    property bool codeFont: true
    // O interruptor de preferencia (veja acima) e, opcional, o icone dele.
    property bool switchStyle: false
    property string iconName: ""

    signal toggled()

    readonly property bool hovered: area.containsMouse

    implicitWidth: root.switchStyle
                   ? content.implicitWidth + 2 * Theme.spacingSmall + Theme.spacingXSmall
                   : Math.max(22, label.implicitWidth + 2 * Theme.spacingMedium)
    implicitHeight: root.switchStyle ? 26 : 22
    width: implicitWidth
    height: implicitHeight
    radius: root.switchStyle ? height / 2 : Theme.radiusXSmall
    color: area.pressed ? Theme.surfaceSelected
           : (root.active && !root.switchStyle ? Theme.surfaceSelected
              : (root.hovered ? Theme.surface2 : (root.switchStyle ? Theme.background0 : "transparent")))
    border.width: 1
    border.color: root.switchStyle ? (root.hovered ? Theme.borderStrong : Theme.borderSoft)
                  : (root.active ? Theme.accent : (root.outlined || root.hovered ? Theme.borderSoft : "transparent"))
    scale: area.pressed ? 0.97 : 1

    Behavior on color {
        ColorAnimation { duration: Theme.motionFast }
    }

    Behavior on scale {
        NumberAnimation { duration: Theme.motionFast }
    }

    Row {
        id: content

        anchors.centerIn: parent
        spacing: Theme.spacingSmall

        KvIcon {
            anchors.verticalCenter: parent.verticalCenter
            visible: root.switchStyle && root.iconName !== ""
            name: root.iconName
            size: 14
            active: root.active
        }

        Text {
            id: label

            anchors.verticalCenter: parent.verticalCenter
            text: root.labelText
            color: root.active ? (root.switchStyle ? Theme.textPrimary : Theme.accent)
                               : (root.hovered ? Theme.textSecondary : Theme.textMuted)
            font.family: root.codeFont && !root.switchStyle ? Theme.monoFont : Theme.uiFont
            font.pixelSize: Theme.fontSizeSmall
            font.bold: root.active && !root.switchStyle
        }

        // O trilho do interruptor: ambar e bolinha a direita quando ligado.
        Rectangle {
            anchors.verticalCenter: parent.verticalCenter
            visible: root.switchStyle
            width: 26
            height: 14
            radius: height / 2
            color: root.active ? Theme.accent : Theme.surface2
            border.width: 1
            border.color: root.active ? Theme.accent : Theme.borderStrong

            Behavior on color {
                ColorAnimation { duration: Theme.motionFast }
            }

            Rectangle {
                width: 10
                height: 10
                radius: width / 2
                anchors.verticalCenter: parent.verticalCenter
                x: root.active ? parent.width - width - 2 : 2
                color: root.active ? Theme.background0 : Theme.textMuted

                Behavior on x {
                    NumberAnimation { duration: Theme.motionFast; easing.type: Theme.easingStandard }
                }
            }
        }
    }

    MouseArea {
        id: area

        anchors.fill: parent
        cursorShape: Qt.PointingHandCursor
        hoverEnabled: true
        onClicked: root.toggled()
        onContainsMouseChanged: {
            if (root.tooltip === "") return;
            if (containsMouse) TooltipController.showFor(root, root.tooltip, "bottom");
            else TooltipController.hideFor(root);
        }
    }
}
