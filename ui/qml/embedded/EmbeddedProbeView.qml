pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A secao "Sonda" da aba Placa (E3-5; saiu do EmbeddedPanel quando ele
// virou abas): as sondas que o probe-rs reportou, o veredito quando nao
// ha' nenhuma, a dica do core e a saida CRUA — o que deixa o usuario ver
// se ha' uma sonda ali e o formato mudou. Le do EmbeddedController.
Column {
    id: root

    property var controller: null

    spacing: Theme.spacingSmall

    Text {
        text: qsTr("Sonda")
        color: Theme.textSecondary
        font.pixelSize: 11
        font.bold: true
    }

    Repeater {
        model: root.controller ? root.controller.probes : []

        Text {
            id: linhaSonda

            required property var modelData

            width: root.width
            text: "● " + root.controller.probeSummary(linhaSonda.modelData)
            color: Theme.textPrimary
            font.family: Theme.monoFont
            font.pixelSize: 11
            elide: Text.ElideMiddle
        }
    }

    // O veredito comum (F8): a busca em andamento, ou o que ela achou.
    // A ferramenta ausente e' outro estado: a dica do core ja' diz como
    // instalar, e a tela nao repete.
    KvVerdict {
        id: vereditoSonda

        width: parent.width
        visible: root.controller && !root.controller.probeFound
                 && (vereditoSonda.busy || vereditoSonda.showsBand)
        busy: root.controller !== null && root.controller.busy
        busyText: qsTr("procurando…")
        neutral: root.controller !== null && root.controller.toolAvailable
        message: root.controller
                 ? (root.controller.toolAvailable ? qsTr("nenhuma sonda reconhecida")
                                                  : qsTr("probe-rs não encontrado nesta máquina"))
                 : ""
    }

    Text {
        width: parent.width
        wrapMode: Text.WordWrap
        visible: root.controller && root.controller.hint !== "" && !root.controller.busy
        text: root.controller ? root.controller.hint : ""
        color: Theme.textMuted
        font.pixelSize: 10
    }

    Rectangle {
        id: caixaCrua

        width: parent.width
        height: Math.min(96, saidaCrua.contentHeight + 2 * Theme.spacingSmall)
        visible: root.controller && !root.controller.probeFound
                 && root.controller.rawOutput.trim() !== "" && !root.controller.busy
        radius: Theme.radius
        color: Theme.background0
        border.width: 1
        border.color: Theme.borderSoft
        clip: true

        Flickable {
            anchors.fill: parent
            anchors.margins: Theme.spacingSmall
            contentHeight: saidaCrua.contentHeight
            clip: true

            Text {
                id: saidaCrua

                width: parent.width
                wrapMode: Text.WrapAnywhere
                text: root.controller ? root.controller.rawOutput.trim() : ""
                color: Theme.textMuted
                font.family: Theme.monoFont
                font.pixelSize: 10
            }
        }
    }
}
