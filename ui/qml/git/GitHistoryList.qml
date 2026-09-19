pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O HISTORICO de commits.
//
// Saiu do GitPanel.qml em 2026-09-03 (etapa 17). Mesma razao e mesma forma da
// GitChangesList: a raiz e' o ListView, e o posicionamento fica no GitPanel.
ListView {
    id: root

    property var historyModel
    property bool historyLoading: false
    property bool repo: false
    // O grafo (HUD do Git, 2026-09-18): quantas raias, e o commit selecionado.
    property int laneCount: 1
    property string selectedSha: ""
    readonly property int laneWidth: 12
    readonly property int graphWidth: Math.max(1, laneCount) * laneWidth + Theme.spacingXSmall

    GitRules { id: rules }

    signal commitActivated(string sha, string shortSha, string summary)


    // B2 (DocsPublic/roadmaps/24): barra de rolagem. `parent: root` é OBRIGATÓRIO — um filho
    // declarado dentro de um ListView vira filho do contentItem e ROLARIA
    // junto com a lista. O ListView segue sendo a fonte da verdade.
    VerticalScrollBar {
        id: scrollBar

        parent: root
        anchors.right: root.right
        anchors.top: root.top
        anchors.bottom: root.bottom

        contentSize: root.contentHeight
        viewportSize: root.height
        position: root.contentY

        onMoveRequested: function(position) {
            root.contentY = position;
        }
    }
    clip: true
    model: root.historyModel

    Text {
        anchors.centerIn: parent
        visible: root.historyModel.count === 0
        text: root.historyLoading
              ? qsTr("Carregando histórico...")
              : (root.repo
                 ? qsTr("Sem commits ainda.")
                 : qsTr("Este workspace não é um repositório git."))
        color: Theme.textMuted
        font.pixelSize: 11
    }

    delegate: Rectangle {
        id: commitRowItem

        required property int index
        required property string sha
        required property string shortSha
        required property string author
        required property string age
        required property string summary
        required property string refsText

        readonly property var refs: refsText === "" ? [] : refsText.split("\u001f")
        required property int lane
        required property bool merge
        required property string edgesText

        readonly property var edges: edgesText === "" ? []
            : edgesText.split(",").map(e => ({ from: Number(e.split(">")[0]), to: Number(e.split(">")[1]) }))

        readonly property bool selected: root.selectedSha === sha

        width: root.width
        height: 24
        radius: Theme.radiusXSmall
        color: selected ? Theme.surfaceSelected
               : (commitRowArea.containsMouse ? Theme.surface2 : "transparent")

        // O grafo: as arestas para a linha de baixo (retas quando a raia e' a
        // mesma, curvas quando muda — fatia 2c) e o ponto do commit.
        Item {
            id: grafo

            anchors.left: parent.left
            anchors.leftMargin: Theme.spacingXSmall
            anchors.top: parent.top
            anchors.bottom: parent.bottom
            width: root.graphWidth

            Canvas {
                id: tela

                anchors.fill: parent
                onPaint: {
                    const ctx = getContext("2d");
                    ctx.reset();
                    ctx.lineWidth = 1.5;
                    const meio = height / 2;
                    const cx = lane => lane * root.laneWidth + root.laneWidth / 2;
                    for (let i = 0; i < commitRowItem.edges.length; i++) {
                        const e = commitRowItem.edges[i];
                        const x0 = cx(e.from), x1 = cx(e.to);
                        // A raia que so' passa e' cinza; a que sai do commit e' ambar.
                        ctx.strokeStyle = e.from === commitRowItem.lane ? Theme.accentDim : Theme.borderStrong;
                        ctx.beginPath();
                        if (e.from === commitRowItem.lane) {
                            ctx.moveTo(x0, meio);
                        } else {
                            ctx.moveTo(x0, 0);
                            ctx.lineTo(x0, meio);
                        }
                        if (x0 === x1) {
                            ctx.lineTo(x1, height);
                        } else {
                            ctx.bezierCurveTo(x0, height, x1, meio, x1, height);
                        }
                        ctx.stroke();
                    }
                    // A metade de cima da raia do proprio commit (vem de cima).
                    if (commitRowItem.index > 0) {
                        ctx.strokeStyle = Theme.accentDim;
                        ctx.beginPath();
                        ctx.moveTo(cx(commitRowItem.lane), 0);
                        ctx.lineTo(cx(commitRowItem.lane), meio);
                        ctx.stroke();
                    }
                }
                Component.onCompleted: requestPaint()
                onWidthChanged: requestPaint()
            }

            Rectangle {
                x: commitRowItem.lane * root.laneWidth + root.laneWidth / 2 - 4
                anchors.verticalCenter: parent.verticalCenter
                width: 8
                height: 8
                radius: 4
                color: commitRowItem.merge ? Theme.background1 : Theme.accent
                border.width: commitRowItem.merge ? 2 : 0
                border.color: Theme.accent
            }
        }

        Text {
            id: commitShaText

            anchors.verticalCenter: parent.verticalCenter
            anchors.left: grafo.right
            anchors.leftMargin: Theme.spacingSmall
            text: commitRowItem.shortSha
            color: Theme.accent
            font.pixelSize: 11
            font.family: Theme.monoFont
        }

        Row {
            id: refsRow

            anchors.verticalCenter: parent.verticalCenter
            anchors.left: commitShaText.right
            anchors.leftMargin: Theme.spacingSmall
            spacing: Theme.spacingXSmall

            Repeater {
                model: commitRowItem.refs

                delegate: Rectangle {
                    id: refChip

                    required property var modelData

                    readonly property var ref: rules.refChip(modelData)

                    width: refLabel.implicitWidth + 2 * Theme.spacingXSmall + 2
                    height: 14
                    radius: 7
                    color: ref.head ? Theme.accentDim : (ref.tag ? Theme.purpleOrbital : Theme.surface2)

                    Text {
                        id: refLabel

                        anchors.centerIn: parent
                        text: refChip.ref.name
                        color: refChip.ref.head || refChip.ref.tag ? Theme.background0 : Theme.textSecondary
                        font.pixelSize: 9
                        font.bold: refChip.ref.head
                    }
                }
            }
        }

        Text {
            anchors.verticalCenter: parent.verticalCenter
            anchors.left: refsRow.right
            anchors.leftMargin: commitRowItem.refs.length > 0 ? Theme.spacingSmall : 0
            anchors.right: commitMetaText.left
            anchors.rightMargin: Theme.spacingSmall
            text: commitRowItem.summary
            color: Theme.textPrimary
            font.pixelSize: 11
            elide: Text.ElideRight
        }

        Text {
            id: commitMetaText

            anchors.verticalCenter: parent.verticalCenter
            anchors.right: parent.right
            anchors.rightMargin: Theme.spacingSmall
            text: commitRowItem.author + ", " + commitRowItem.age
            color: Theme.textMuted
            font.pixelSize: 10
        }

        MouseArea {
            id: commitRowArea

            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: root.commitActivated(commitRowItem.sha,
                                             commitRowItem.shortSha,
                                             commitRowItem.summary)
        }
    }
}

