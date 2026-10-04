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

    // O caminho selecionado (o painel da direita mostra o diff dele).
    property string selectedAbsPath: ""

    signal stageToggleRequested(int index)
    // Stage/unstage de TODOS os caminhos de uma pasta (a secao).
    signal folderStageRequested(var absPaths, bool stageAll)
    signal selectRequested(string absPath, string path)
    signal diffRequested(string absPath)
    signal discardRequested(int index)
    signal openRequested(string absPath)

    FlickableScrollBar {
        id: scrollBar

        view: root
    }

    clip: true
    model: root.changesModel

    // As mudancas agrupadas pela pasta de primeiro nivel (HUD do Git,
    // 2026-09-18): o cabecalho da secao e' a pasta.
    section.property: "folder"
    section.criteria: ViewSection.FullString
    // O cabecalho da secao vem das PARTES (arquivo sem pragma Bound): ver
    // GitChangesListParts.qml para o porque (Qt 6.4).
    readonly property GitChangesListParts parts: GitChangesListParts {}
    section.delegate: root.parts.section

    Text {
        anchors.centerIn: parent
        visible: root.changesModel.count === 0
        text: root.repo
              ? qsTr("Sem mudanças — árvore limpa.")
              : qsTr("Este projeto não é um repositório git.")
        color: Theme.textMuted
        font.pixelSize: Theme.fontSizeSmall
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
               : (rowHover.hovered ? Theme.surface2 : "transparent")

        HoverHandler {
            id: rowHover
        }

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
            font.pixelSize: Theme.fontSizeSmall
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

        // As acoes da linha aparecem com o mouse SOBRE A LINHA (HoverHandler: nao
        // pisca ao passar de um botao para outro, o defeito da arvore do Banco).
        KvIconButton {
            id: diffChip

            anchors.verticalCenter: parent.verticalCenter
            anchors.right: discardChip.left
            width: 20
            height: 20
            visible: rowHover.hovered
            compact: true
            iconSize: 13
            iconName: "eye"
            tooltip: qsTr("Ver o diff")
            focusOnClick: false
            onClicked: root.diffRequested(changeRow.absPath)
        }

        KvIconButton {
            id: discardChip

            anchors.verticalCenter: parent.verticalCenter
            anchors.right: parent.right
            anchors.rightMargin: Theme.spacingXSmall
            width: 20
            height: 20
            visible: diffChip.visible
            compact: true
            danger: true
            iconSize: 13
            iconName: "undo"
            tooltip: qsTr("Descartar a mudança…")
            focusOnClick: false
            onClicked: root.discardRequested(changeRow.index)
        }
    }
}

