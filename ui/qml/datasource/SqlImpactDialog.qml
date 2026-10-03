pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O PAINEL QUE PERGUNTA antes de uma escrita rodar (2026-10-03, pedido do
// autor: "exibindo para ele o comando e a consequencia de rodar aquele
// comando"). A logica e' do DataSourceImpactController; aqui so' o desenho:
//
//   ⚠ Esta instrução APAGA dados                 (ou: escreve no banco)
//   conexão loja · SQLite
//   ┌ o comando, inteiro, em mono ─────────────────────────────┐
//   └──────────────────────────────────────────────────────────┘
//   O QUE ACONTECE
//   ● Apaga TODAS as linhas de clientes: 3 linhas
//   Não há como desfazer depois de executar.
//   Para confirmar, digite clientes  [__________]
//                                   [Cancelar]  [Executar]
//
// Cancelar e' o padrao (Esc, clique fora); Executar so' liga quando a medida
// chegou e, na destrutiva, quando o nome digitado confere.
KvPanelFrame {
    id: root

    property var impact: null
    property string engineLabel: ""

    panelWidth: 600
    // A altura do CONTEUDO (o texto, a lista, a confirmacao) + a linha dos
    // botoes + a margem da moldura: sem vao vazio entre o campo e os botoes.
    panelHeight: body.implicitHeight + Theme.spacingMedium + buttons.height + 2 * Theme.spacingMedium

    readonly property bool destructive: root.impact !== null && root.impact.destructive

    onVisibleChanged: if (visible) focusTimer.restart()
    // A medida chega depois de o painel abrir: se ela diz DESTRUTIVA, o foco
    // vai para o campo onde se digita o nome.
    Connections {
        target: root.impact

        function onSeverityChanged() {
            if (root.visible) focusTimer.restart();
        }

        function onErrorTextChanged() {
            if (root.visible) focusTimer.restart();
        }
    }

    // O foco vai ao campo de confirmacao (destrutiva) ou ao Cancelar.
    Timer {
        id: focusTimer

        interval: 0
        onTriggered: if (root.destructive) confirmInput.forceActiveFocus(); else cancelButton.forceActiveFocus()
    }

    Column {
        id: body

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        spacing: Theme.spacingSmall

        Row {
            spacing: Theme.spacingSmall

            KvIcon {
                anchors.verticalCenter: parent.verticalCenter
                size: 20
                name: "warning"
                warning: true
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: !root.impact ? "" : (root.impact.measuring ? qsTr("Esta instrução escreve no banco — medindo o impacto…")
                      : (root.destructive ? qsTr("Esta instrução APAGA dados") : qsTr("Esta instrução escreve no banco")))
                color: root.destructive ? Theme.errorSoft : Theme.warningSoft
                font.pixelSize: Theme.fontSizeLarge
                font.weight: Font.DemiBold
            }
        }

        Text {
            text: root.impact ? qsTr("conexão %1").arg(root.impact.name) + (root.engineLabel !== "" ? " · " + root.engineLabel : "") : ""
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeSmall
        }

        // O comando inteiro, como vai rodar.
        Rectangle {
            width: parent.width
            height: Math.min(110, commandText.implicitHeight + 2 * Theme.spacingSmall)
            radius: Theme.radius
            color: Theme.background0
            border.width: 1
            border.color: root.destructive ? Theme.errorSoft : Theme.borderSoft
            clip: true

            Flickable {
                id: commandScroll

                anchors.fill: parent
                anchors.margins: Theme.spacingSmall
                contentWidth: width
                contentHeight: commandText.implicitHeight
                boundsBehavior: Flickable.StopAtBounds

                Text {
                    id: commandText

                    width: commandScroll.width
                    wrapMode: Text.WrapAnywhere
                    text: root.impact ? root.impact.sql : ""
                    color: Theme.textPrimary
                    font.family: Theme.monoFont
                    font.pixelSize: Theme.fontSizeSmall
                }
            }
        }

        Text {
            text: qsTr("O QUE ACONTECE")
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeMicro
            font.weight: Font.DemiBold
            font.letterSpacing: 0.8
        }

        Repeater {
            model: root.impact ? root.impact.statements : []

            delegate: Row {
                id: effect

                required property var modelData

                spacing: Theme.spacingSmall

                Rectangle {
                    anchors.verticalCenter: parent.verticalCenter
                    width: 8
                    height: 8
                    radius: width / 2
                    color: root.impact.isDestructive(effect.modelData) ? Theme.errorSoft : Theme.warningSoft
                }

                Text {
                    width: root.frameWidth - 6 * Theme.spacingMedium
                    text: root.impact.describe(effect.modelData)
                    color: Theme.textPrimary
                    font.pixelSize: Theme.fontSizeSmall
                    font.weight: root.impact.isDestructive(effect.modelData) ? Font.DemiBold : Font.Normal
                    wrapMode: Text.WordWrap
                }
            }
        }

        Text {
            width: parent.width
            visible: root.impact !== null && (root.impact.measuring || root.impact.errorText !== "")
            wrapMode: Text.WordWrap
            text: !root.impact ? "" : (root.impact.measuring ? qsTr("Contando as linhas atingidas (só leitura)…")
                                       : qsTr("Não foi possível medir: %1. Sem medida, a instrução é tratada como destrutiva.").arg(root.impact.errorText))
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeCaption
        }

        Text {
            width: parent.width
            visible: root.destructive && !root.impact.measuring
            wrapMode: Text.WordWrap
            text: qsTr("Não há como desfazer depois de executar. Se os dados importam, faça um backup antes.")
            color: Theme.errorSoft
            font.pixelSize: Theme.fontSizeCaption
        }

        // A confirmacao digitada da destrutiva.
        Row {
            visible: root.destructive && !root.impact.measuring
            spacing: Theme.spacingSmall

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: qsTr("Para confirmar, digite")
                color: Theme.textSecondary
                font.pixelSize: Theme.fontSizeSmall
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: root.impact ? root.impact.confirmName : ""
                color: Theme.textPrimary
                font.family: Theme.monoFont
                font.pixelSize: Theme.fontSizeSmall
                font.weight: Font.DemiBold
            }

            Rectangle {
                width: 200
                height: 26
                radius: Theme.radiusXSmall
                color: Theme.background0
                border.width: 1
                border.color: confirmInput.activeFocus ? Theme.accent : Theme.borderSoft

                TextInput {
                    id: confirmInput

                    anchors.fill: parent
                    anchors.leftMargin: Theme.spacingSmall
                    anchors.rightMargin: Theme.spacingSmall
                    verticalAlignment: TextInput.AlignVCenter
                    text: root.impact ? root.impact.typed : ""
                    color: Theme.textPrimary
                    font.family: Theme.monoFont
                    font.pixelSize: Theme.fontSizeSmall
                    clip: true
                    selectByMouse: true
                    onTextEdited: root.impact.typed = text
                    Keys.onReturnPressed: root.impact.confirm()
                    Keys.onEnterPressed: root.impact.confirm()
                }
            }
        }
    }

    Row {
        id: buttons

        anchors.right: parent.right
        anchors.bottom: parent.bottom
        spacing: Theme.spacingSmall

        KvButton {
            id: cancelButton

            compact: true
            text: qsTr("Cancelar")
            onClicked: root.dismissRequested()
        }

        // Vermelho so' para o que destroi; a escrita comum e' a acao primaria.
        KvButton {
            compact: true
            danger: root.destructive
            primary: !root.destructive
            text: root.destructive ? qsTr("Executar e apagar") : qsTr("Executar")
            enabled: root.impact !== null && root.impact.canRun
            onClicked: root.impact.confirm()
        }
    }
}
