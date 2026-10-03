pragma ComponentBehavior: Bound
import QtQuick

Rectangle {
    id: root

    // O trilho da Etapa 3 (E3-4, roadmaps/44 §4): Projeto · Git ·
    // Embarcados · Banco · Containers · Grafana · Ferramentas. Busca,
    // Build e Debug SAIRAM daqui (pedido do autor: "tem muito atalho
    // repetido; melhor deixar os do canto superior direito") — a busca no
    // projeto e' Ctrl+Shift+F e a aba de baixo; Build/Debug sao o widget
    // Executar do cabecalho, o menu e o painel de baixo.
    // As entradas sao DADO (V3, 2026-09-24): antes cada uma custava uma
    // propriedade `xActive`, um sinal `xRequested` e um bloco de botao aqui,
    // mais o binding e o handler do outro lado. Quem declara e' o
    // `ToolWindows`; este arquivo so' desenha.
    property var entries: []
    // A ordem que o usuario arrastou (0.3.9): os ids, na ordem dele.
    property var order: []
    readonly property ShellLayoutCodec codec: ShellLayoutCodec {}
    readonly property var orderedEntries: root.codec.orderItems(root.entries, "id", root.order)
    signal entryMoved(string id, int dropIndex, var visibleIds)
    // Os dois trilhos (0.3.9): o da esquerda e' o PRINCIPAL (tem o "⋯ Mais" e
    // o modo expandido); o da direita so' os icones. Um icone arrastado para
    // o outro trilho passa para la' (`entryTransferred`).
    property bool primary: true
    property ReorderController partnerReorder: null
    readonly property alias reorder: railReorder
    signal entryTransferred(string id, int dropIndex, var targetIds)

    signal activated(string id)
    // 0.3.7 F1: botao direito num icone (fixar, ocultar, restaurar) e o
    // "Mais" (as areas fora do trilho). As coordenadas sao deste item; o
    // host mapeia e abre o menu compartilhado.
    signal contextMenuRequested(string id, real menuX, real menuY)
    signal moreRequested(real menuX, real menuY)
    // O modo EXPANDIDO (F1, fechamento da Etapa 2): o rotulo ao lado do
    // icone, como a referencia; o chevron do pe' alterna e a escolha e'
    // persistida pelo ShellController.
    property bool expanded: false
    signal expandedToggled()

    // Fino e encostado na borda da janela (0.3.9): com o vao, a ilha comeca
    // a' mesma distancia da esquerda que do topo.
    width: expanded ? 168 : 34
    radius: Theme.radiusLarge
    // Sobre a moldura, sem ilha (0.3.9, paleta ilhas): o trilho e' borda da
    // janela, como na JetBrains.
    color: "transparent"

    component RailButton: Rectangle {
        id: railButton

        property string iconName: "file"
        property string tooltip: ""
        // O rotulo do modo expandido; sem ele, o tooltip ate' o primeiro parentese.
        property string label: ""
        property bool active: false
        // Vazio no "Mais": ele nao se arrasta.
        property string reorderKey: ""

        signal activated()
        signal contextMenuRequested(real menuX, real menuY)

        width: root.expanded ? root.width - 2 * Theme.spacingSmall : 28
        height: 28
        radius: Theme.radius
        opacity: enabled ? 1.0 : 0.72
        color: active ? Theme.surfaceSelected
                      : (railButtonArea.containsMouse ? Theme.surface2 : "transparent")

        KvIcon {
            id: railIcon

            anchors.verticalCenter: parent.verticalCenter
            anchors.left: parent.left
            anchors.leftMargin: 4
            name: railButton.iconName
            size: 20
            active: railButton.active
            disabled: !railButton.enabled
            iconColor: railButton.active ? Theme.accent
                                         : (railButtonArea.containsMouse
                                            ? Theme.textPrimary
                                            : Theme.textSecondary)
        }

        Text {
            anchors.verticalCenter: parent.verticalCenter
            anchors.left: railIcon.right
            anchors.leftMargin: Theme.spacingSmall
            anchors.right: parent.right
            visible: root.expanded
            text: railButton.label !== "" ? railButton.label : railButton.tooltip.split(" (")[0]
            color: railButton.active ? Theme.accent : Theme.textSecondary
            font.pixelSize: 12
            elide: Text.ElideRight
        }

        // Arrastar reordena o trilho (0.3.9); clique ativa, direito abre o menu.
        ReorderMouseArea {
            id: railButtonArea

            anchors.fill: parent
            reorder: railReorder
            reorderKey: railButton.reorderKey
            onContainsMouseChanged: {
                if (containsMouse && !root.expanded) {
                    TooltipController.showFor(railButton, railButton.tooltip,
                                              "right");
                } else {
                    TooltipController.hideFor(railButton);
                }
            }
            onTapped: function(mouse) {
                TooltipController.hideFor(railButton);
                // O menu de contexto vale tambem para a area indisponivel:
                // ocultar ou desafixar nao depende de abrir.
                if (mouse.button === Qt.RightButton) {
                    railButton.contextMenuRequested(mouse.x, mouse.y);
                } else if (railButton.enabled) {
                    railButton.activated();
                }
            }
        }
    }

    Column {
        id: railColumn

        anchors.top: parent.top
        anchors.topMargin: Theme.spacingSmall
        anchors.horizontalCenter: parent.horizontalCenter
        spacing: Theme.spacingSmall

        Repeater {
            model: root.orderedEntries

            delegate: RailButton {
                required property var modelData

                reorderKey: modelData.id
                opacity: (modelData.available ? 1.0 : 0.72) * railReorder.opacityFor(modelData.id)
                iconName: modelData.icon
                tooltip: modelData.tooltip
                label: modelData.label
                active: modelData.active
                enabled: modelData.available
                onActivated: root.activated(modelData.id)
                onContextMenuRequested: function(menuX, menuY) {
                    const pos = mapToItem(root, menuX, menuY);
                    root.contextMenuRequested(modelData.id, pos.x, pos.y);
                }
            }
        }

        // O "Mais": as areas que nao estao no trilho (contextuais sem uso,
        // desafixadas, ocultas), com o motivo e o atalho de cada uma.
        RailButton {
            id: moreButton

            visible: root.primary
            iconName: "more"
            tooltip: qsTr("Mais áreas")
            label: qsTr("Mais")
            onActivated: {
                const pos = mapToItem(root, width, 0);
                root.moreRequested(pos.x, pos.y);
            }
        }
    }

    // O chevron do pe': compacto <-> expandido.
    Rectangle {
        visible: root.primary
        anchors.bottom: parent.bottom
        anchors.bottomMargin: Theme.spacingSmall
        anchors.horizontalCenter: parent.horizontalCenter
        width: root.expanded ? root.width - 2 * Theme.spacingSmall : 28
        height: 24
        radius: Theme.radius
        color: chevronArea.containsMouse ? Theme.surface2 : "transparent"

        Text {
            anchors.centerIn: parent
            text: root.expanded ? qsTr("‹ recolher") : "›"
            color: Theme.textMuted
            font.pixelSize: 12
        }

        MouseArea {
            id: chevronArea

            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: root.expandedToggled()
        }
    }

    ReorderController {
        id: railReorder

        container: railColumn
        dropZone: root
        vertical: true
        partner: root.partnerReorder
        onMoved: function(key, dropIndex, visibleKeys) {
            root.entryMoved(key, dropIndex, visibleKeys);
        }
        onTransferred: function(key, dropIndex, targetKeys) {
            root.entryTransferred(key, dropIndex, targetKeys);
        }
    }
}
