pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A LISTA da janela de Containers (2026-10-03): as secoes do ContainerRows,
// uma ContainerRow por linha (secao, container ou imagem). O clique num
// container o escolhe (o detalhe embaixo da janela).
// Sem motor, ou sem nada listado, o meio da lista diz o que falta — o passo
// oficial vem do core (`status.hint`); a IDE nunca roda instalacao.
ListView {
    id: root

    property var controller: null
    property var rowsModel: null

    clip: true
    boundsBehavior: Flickable.StopAtBounds
    model: root.rowsModel ? root.rowsModel.rows : []

    readonly property bool engineMissing: root.controller !== null && !root.controller.statusBusy
                                          && (!root.controller.engineFound || !root.controller.reachable)
    readonly property bool empty: root.rowsModel !== null && !root.rowsModel.rows.some(r => r.kind !== "section")

    FlickableScrollBar {
        id: scrollBar

        view: root
    }

    // O meio da lista quando nao ha' o que mostrar: o porque e o proximo passo.
    Column {
        anchors.centerIn: parent
        width: parent.width - 2 * Theme.spacingMedium
        spacing: Theme.spacingSmall
        visible: root.engineMissing || (root.empty && root.controller !== null && !root.controller.listBusy)

        Text {
            width: parent.width
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.WordWrap
            text: root.engineMissing
                  ? (root.controller.engineFound ? qsTr("O motor %1 não responde").arg(root.controller.engineLabel)
                                                 : qsTr("Nenhum motor de container nesta máquina"))
                  : (root.controller && root.controller.filter.trim() !== "" ? qsTr("Nada com esse filtro")
                                                                           : qsTr("Nenhum container nem imagem"))
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeSmall
        }

        Text {
            width: parent.width
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.WordWrap
            visible: text !== ""
            text: root.controller === null ? ""
                  : (root.controller.errorText !== "" ? root.controller.errorText
                     : (root.engineMissing && root.controller.status.hint !== undefined ? root.controller.status.hint : ""))
            color: root.controller && root.controller.errorText !== "" ? Theme.errorSoft : Theme.warningSoft
            font.pixelSize: Theme.fontSizeCaption
        }
    }

    delegate: ContainerRow {
        width: root.width - scrollBar.width
        controller: root.controller
        rowsModel: root.rowsModel
    }
}
