pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O PAINEL DE AREAS do trilho (0.3.7). Substitui a lista plana do menu
// generico, que o autor achou confusa em 2026-10-02 ("tudo numa cor so', em
// formato de lista, um abaixo do outro"). Duas secoes — o que esta' no trilho
// e o que esta' fora —, cada area com o estado em palavras e as acoes como
// botoes de icone com estado. O "⋯ Mais" e o botao direito num icone abrem
// ESTE painel (o botao direito destaca a area clicada): um modelo so'.
FocusScope {
    id: root

    property var toolWindows: null
    property var shellController: null
    property real anchorX: 0
    property real anchorY: 0
    property string focusId: ""

    readonly property var projection: toolWindows ? toolWindows.projection : null
    readonly property var railState: toolWindows ? toolWindows.railState
                                                 : ({ pinned: [], unpinned: [], hidden: [] })
    // As SECOES ficam como estavam ao abrir (visto na tela real em 2026-10-02:
    // fixar uma area a puxava para cima, a lista andava sob o ponteiro e o
    // proximo clique caia no alfinete de outra). Enquanto o painel esta'
    // aberto so' o ESTADO de cada linha muda; reabrir reorganiza.
    property var frozenRailIds: []
    readonly property var onRail: toolWindows
        ? toolWindows.entries.filter(function(e) { return root.frozenRailIds.indexOf(e.id) >= 0; })
        : []
    readonly property var offRail: toolWindows && projection
        ? projection.overflowEntries(toolWindows.entries, onRail) : []

    // Esta' no trilho AGORA (o estado mostrado), independente da secao.
    function railHas(id) {
        return toolWindows !== null
            && toolWindows.visibleEntries.some(function(e) { return e.id === id; });
    }

    visible: false

    function openAt(x, y, id) {
        frozenRailIds = toolWindows ? toolWindows.visibleEntries.map(function(e) { return e.id; }) : [];
        anchorX = x;
        anchorY = y;
        focusId = id;
        visible = true;
        forceActiveFocus();
    }

    function close() {
        visible = false;
        focusId = "";
    }

    function togglePin(entry) {
        if (projection.isPinned(entry, railState)) {
            shellController.unpinArea(entry.id);
        } else {
            shellController.pinArea(entry.id);
        }
    }

    function toggleHidden(entry) {
        if (projection.contains(railState.hidden, entry.id)) {
            shellController.resetArea(entry.id);
        } else {
            shellController.hideArea(entry.id);
        }
    }

    function open(entry) {
        close();
        toolWindows.activate(entry.id);
    }

    Keys.onEscapePressed: close()

    // Clique fora fecha.
    MouseArea {
        anchors.fill: parent
        onClicked: root.close()
    }

    Rectangle {
        id: panel

        x: Math.min(root.anchorX, root.width - width - Theme.spacingSmall)
        y: Math.max(Theme.spacingSmall,
                    Math.min(root.anchorY, root.height - height - Theme.spacingSmall))
        width: 380
        height: content.implicitHeight + 2 * Theme.spacingMedium
        radius: Theme.radiusLarge
        color: Theme.background2
        border.color: Theme.borderStrong
        border.width: 1

        // O clique dentro do painel nao fecha.
        MouseArea {
            anchors.fill: parent
        }

        Column {
            id: content

            anchors.fill: parent
            anchors.margins: Theme.spacingMedium
            spacing: Theme.spacingXSmall

            Text {
                text: qsTr("Áreas da IDE")
                color: Theme.textPrimary
                font.pixelSize: Theme.fontSizeLarge
                font.weight: Font.DemiBold
            }

            Text {
                width: parent.width
                text: qsTr("Fixe o que você usa sempre; as outras aparecem quando o projeto usar.")
                color: Theme.textMuted
                font.pixelSize: Theme.fontSizeCaption
                wrapMode: Text.WordWrap
            }

            Item { width: 1; height: Theme.spacingXSmall }

            Text {
                text: qsTr("NO TRILHO")
                color: Theme.textSecondary
                font.pixelSize: Theme.fontSizeMicro
                font.weight: Font.DemiBold
                font.letterSpacing: 0.8
            }

            Repeater {
                model: root.onRail

                delegate: RailAreaRow {
                    required property var modelData

                    width: content.width
                    entry: modelData
                    onRail: root.railHas(modelData.id)
                    status: root.projection.statusOf(modelData, root.railState,
                                                     root.railHas(modelData.id))
                    pinned: root.projection.isPinned(modelData, root.railState)
                    hidden: root.projection.contains(root.railState.hidden, modelData.id)
                    highlighted: root.focusId === modelData.id
                    onOpenRequested: root.open(modelData)
                    onPinToggled: root.togglePin(modelData)
                    onHideToggled: root.toggleHidden(modelData)
                }
            }

            Item { width: 1; height: Theme.spacingXSmall; visible: root.offRail.length > 0 }

            Text {
                visible: root.offRail.length > 0
                text: qsTr("FORA DO TRILHO")
                color: Theme.textSecondary
                font.pixelSize: Theme.fontSizeMicro
                font.weight: Font.DemiBold
                font.letterSpacing: 0.8
            }

            Repeater {
                model: root.offRail

                delegate: RailAreaRow {
                    required property var modelData

                    width: content.width
                    entry: modelData
                    onRail: root.railHas(modelData.id)
                    status: root.projection.statusOf(modelData, root.railState,
                                                     root.railHas(modelData.id))
                    pinned: root.projection.isPinned(modelData, root.railState)
                    hidden: root.projection.contains(root.railState.hidden, modelData.id)
                    highlighted: root.focusId === modelData.id
                    onOpenRequested: root.open(modelData)
                    onPinToggled: root.togglePin(modelData)
                    onHideToggled: root.toggleHidden(modelData)
                }
            }

            Rectangle {
                width: parent.width
                height: 1
                color: Theme.borderSoft
            }

            Item {
                width: parent.width
                height: 28

                KvButton {
                    anchors.left: parent.left
                    anchors.verticalCenter: parent.verticalCenter
                    compact: true
                    text: qsTr("Restaurar padrão")
                    onClicked: root.shellController.restoreRail()
                }

                Text {
                    anchors.right: parent.right
                    anchors.verticalCenter: parent.verticalCenter
                    text: qsTr("Esc fecha")
                    color: Theme.textMuted
                    font.pixelSize: Theme.fontSizeCaption
                }
            }
        }
    }
}
