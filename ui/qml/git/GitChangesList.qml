pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A lista de MUDANCAS do git (working tree + index).
//
// Saiu do GitPanel.qml em 2026-09-03 (etapa 17 do roadmaps/34), que estava em
// 764/300 com cinco areas visuais no mesmo arquivo. A raiz aqui e' o proprio
// ListView de proposito: ele e' a fonte da verdade do scroll, e envolve-lo num
// Item quebraria o contrato do VerticalScrollBar (comentario B2 abaixo).
// O POSICIONAMENTO fica no GitPanel: a ancora depende dos irmaos.
ListView {
    id: root

    property var changesModel
    property bool repo: false
    // Sobe a cada status novo (GitController.revision): a secao reavalia.
    property int revision: 0

    GitRules { id: rules }
    // O caminho selecionado (o painel da direita mostra o diff dele).
    property string selectedAbsPath: ""

    signal stageToggleRequested(int index)
    // Stage/unstage de TODOS os caminhos de uma pasta (a secao).
    signal folderStageRequested(var absPaths, bool stageAll)
    signal selectRequested(string absPath, string path)
    signal diffRequested(string absPath)
    signal discardRequested(int index)
    signal openRequested(string absPath)



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
    model: root.changesModel

    // As mudancas agrupadas pela pasta de primeiro nivel (HUD do Git,
    // 2026-09-18): o cabecalho da secao e' a pasta.
    section.property: "folder"
    section.criteria: ViewSection.FullString
    section.delegate: Item {
        id: secao

        required property string section

        // Reavalia quando a lista muda (o ListModel nao notifica funcoes).
        readonly property var estado: root.revision >= 0 ? rules.folderState(root.changesModel, section) : null

        width: root.width
        height: 20

        // O checkbox da PASTA: cheio quando todas as mudancas dela estao
        // staged, meio quando algumas; o clique faz stage de todas (ou
        // unstage de todas, quando ja' estao).
        Rectangle {
            id: caixaPasta

            anchors.verticalCenter: parent.verticalCenter
            anchors.left: parent.left
            anchors.leftMargin: Theme.spacingSmall
            width: 14
            height: 14
            radius: Theme.radiusXSmall
            color: secao.estado && secao.estado.all ? Theme.accentDim : "transparent"
            border.color: secao.estado && secao.estado.staged > 0 ? Theme.accent : Theme.borderStrong
            border.width: 1

            KvIcon {
                anchors.centerIn: parent
                visible: secao.estado && secao.estado.all
                name: "check"
                size: 11
                active: true
            }

            Rectangle {
                anchors.centerIn: parent
                visible: secao.estado && !secao.estado.all && secao.estado.staged > 0
                width: 6
                height: 2
                color: Theme.accent
            }

            MouseArea {
                anchors.fill: parent
                cursorShape: Qt.PointingHandCursor
                onClicked: root.folderStageRequested(secao.estado.paths, !secao.estado.all)
            }
        }

        Text {
            anchors.verticalCenter: parent.verticalCenter
            anchors.left: caixaPasta.right
            anchors.leftMargin: Theme.spacingSmall
            anchors.right: parent.right
            text: secao.section
            color: Theme.textMuted
            font.pixelSize: 10
            font.weight: Font.DemiBold
            elide: Text.ElideMiddle
        }
    }

    Text {
        anchors.centerIn: parent
        visible: root.changesModel.count === 0
        text: root.repo
              ? qsTr("Sem mudanças — árvore limpa.")
              : qsTr("Este workspace não é um repositório git.")
        color: Theme.textMuted
        font.pixelSize: 11
    }

    delegate: Rectangle {
        id: changeRow

        required property int index
        required property string path
        required property string absPath
        required property string kind
        required property bool staged

        readonly property bool selected: root.selectedAbsPath === absPath

        width: root.width
        height: 24
        radius: Theme.radiusXSmall
        color: selected ? Theme.surfaceSelected
               : (changeRowArea.containsMouse ? Theme.surface2 : "transparent")

        Rectangle {
            id: stageBox

            anchors.verticalCenter: parent.verticalCenter
            anchors.left: parent.left
            anchors.leftMargin: Theme.spacingSmall
            width: 14
            height: 14
            radius: Theme.radiusXSmall
            color: changeRow.staged ? Theme.accentDim : "transparent"
            border.color: changeRow.staged ? Theme.accent : Theme.borderStrong
            border.width: 1

            KvIcon {
                anchors.centerIn: parent
                visible: changeRow.staged
                name: "check"
                size: 11
                active: true
            }

            MouseArea {
                anchors.fill: parent
                cursorShape: Qt.PointingHandCursor
                onClicked: root.stageToggleRequested(changeRow.index)
            }
        }

        Text {
            id: changePathText

            anchors.verticalCenter: parent.verticalCenter
            anchors.left: stageBox.right
            anchors.leftMargin: Theme.spacingSmall
            anchors.right: diffChip.left
            anchors.rightMargin: Theme.spacingSmall
            text: changeRow.path.substring(changeRow.path.lastIndexOf("/") + 1)
            color: StatusColors.gitKind(changeRow.kind)
            font.pixelSize: 11
            font.family: Theme.monoFont
            elide: Text.ElideMiddle
        }

        MouseArea {
            id: changeRowArea

            anchors.fill: parent
            hoverEnabled: true
            z: -1
            // Um clique mostra o diff a direita; dois abrem o arquivo.
            onClicked: root.selectRequested(changeRow.absPath, changeRow.path)
            onDoubleClicked: root.openRequested(changeRow.absPath)
        }

        Rectangle {
            id: diffChip

            anchors.verticalCenter: parent.verticalCenter
            anchors.right: discardChip.left
            anchors.rightMargin: Theme.spacingXSmall
            width: diffChipLabel.width + 2 * Theme.spacingSmall
            height: 18
            radius: Theme.radiusXSmall
            visible: changeRowArea.containsMouse || diffChipArea.containsMouse
                     || discardChipArea.containsMouse
            color: diffChipArea.containsMouse ? Theme.surfaceSelected : Theme.surface1
            border.color: Theme.borderSoft
            border.width: 1

            Text {
                id: diffChipLabel

                anchors.centerIn: parent
                text: qsTr("diff")
                color: Theme.textSecondary
                font.pixelSize: 9
            }

            MouseArea {
                id: diffChipArea

                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: root.diffRequested(changeRow.absPath)
            }
        }

        Rectangle {
            id: discardChip

            anchors.verticalCenter: parent.verticalCenter
            anchors.right: parent.right
            anchors.rightMargin: Theme.spacingSmall
            width: 18
            height: 18
            radius: Theme.radiusXSmall
            visible: diffChip.visible
            color: discardChipArea.containsMouse ? Theme.surfaceSelected : Theme.surface1
            border.color: Theme.borderSoft
            border.width: 1

            Text {
                anchors.centerIn: parent
                text: "↩"
                color: Theme.errorSoft
                font.pixelSize: 10
            }

            MouseArea {
                id: discardChipArea

                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: root.discardRequested(changeRow.index)
            }
        }
    }
}

