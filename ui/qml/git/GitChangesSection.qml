import QtQuick
import KineinVectis

// O cabecalho de uma secao (pasta) da GitChangesList: checkbox da pasta e o
// nome. Criado pela GitChangesListParts (ver la' o porque); a lista dona e'
// `ListView.view`, tipada como `var` de proposito — tipar como GitChangesList
// fecha um ciclo de tipos (lista -> partes -> secao -> lista) que trava o
// carregador do Qt 6.4.
Item {
    id: sectionRoot

    required property string section

    readonly property var view: ListView.view
    // Reavalia quando a lista muda (o ListModel nao notifica funcoes).
    readonly property var folderState: view && view.revision >= 0
                                  ? rules.folderState(view.changesModel, section) : null

    width: view ? view.width : 0
    height: 20

    GitRules { id: rules }

    // O checkbox da PASTA: cheio quando todas as mudancas dela estao
    // staged, meio quando algumas; o clique faz stage de todas (ou
    // unstage de todas, quando ja' estao).
    Rectangle {
        id: folderBox

        anchors.verticalCenter: parent.verticalCenter
        anchors.left: parent.left
        anchors.leftMargin: Theme.spacingSmall
        width: 14
        height: 14
        radius: Theme.radiusXSmall
        color: sectionRoot.folderState && sectionRoot.folderState.all ? Theme.accentDim : "transparent"
        border.color: sectionRoot.folderState && sectionRoot.folderState.staged > 0 ? Theme.accent : Theme.borderStrong
        border.width: 1

        KvIcon {
            anchors.centerIn: parent
            visible: sectionRoot.folderState && sectionRoot.folderState.all
            name: "check"
            size: 11
            active: true
        }

        Rectangle {
            anchors.centerIn: parent
            visible: sectionRoot.folderState && !sectionRoot.folderState.all && sectionRoot.folderState.staged > 0
            width: 6
            height: 2
            color: Theme.accent
        }

        MouseArea {
            anchors.fill: parent
            cursorShape: Qt.PointingHandCursor
            onClicked: sectionRoot.view.folderStageRequested(sectionRoot.folderState.paths, !sectionRoot.folderState.all)
        }
    }

    Text {
        anchors.verticalCenter: parent.verticalCenter
        anchors.left: folderBox.right
        anchors.leftMargin: Theme.spacingSmall
        anchors.right: parent.right
        text: sectionRoot.section
        color: Theme.textMuted
        font.pixelSize: Theme.fontSizeCaption
        font.weight: Font.DemiBold
        elide: Text.ElideMiddle
    }
}
