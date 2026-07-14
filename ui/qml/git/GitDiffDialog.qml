pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Visão de diff unificado do arquivo (fatia M3.2), overlay padrão dos
// diálogos: +/− coloridos por linha, Esc/fora fecha. Painel de diff
// definitivo (lado a lado, stage por hunk) chega com a M3.3.
Item {
    id: root

    property string title: ""
    property string diffText: ""
    property bool tracked: true
    property bool loading: false
    // M3.4: mensagem de vazio específica (diff de commit reusa o diálogo).
    property string emptyText: ""
    property real maxAvailableWidth: 900
    property real maxAvailableHeight: 600

    signal dismissRequested()

    onVisibleChanged: {
        if (visible) {
            forceActiveFocus();
        }
    }

    function lineColor(line) {
        if (line.startsWith("+++") || line.startsWith("---")) {
            return Theme.textMuted;
        }
        if (line.startsWith("+")) {
            return Theme.successSoft;
        }
        if (line.startsWith("-")) {
            return Theme.errorSoft;
        }
        if (line.startsWith("@@")) {
            return Theme.accent;
        }
        return Theme.textSecondary;
    }

    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        onClicked: root.dismissRequested()
    }

    Rectangle {
        anchors.centerIn: parent
        width: Math.min(920, root.maxAvailableWidth)
        height: Math.min(620, root.maxAvailableHeight)
        radius: Theme.radiusLarge
        color: Theme.background2
        border.color: Theme.borderStrong
        border.width: 1

        // Bloqueia o clique-fora de atravessar o corpo do diálogo.
        MouseArea {
            anchors.fill: parent
        }

        Text {
            id: diffTitle

            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: closeChip.left
            anchors.margins: Theme.spacingMedium
            text: root.title
            color: Theme.textPrimary
            font.pixelSize: 13
            font.bold: true
            elide: Text.ElideMiddle
        }

        KvIconButton {
            id: closeChip
            anchors.top: parent.top
            anchors.right: parent.right
            anchors.margins: Theme.spacingSmall
            iconName: "close"
            tooltip: qsTr("Fechar")
            onClicked: root.dismissRequested()
        }

        ListView {
            id: diffView

            anchors.top: diffTitle.bottom
            anchors.topMargin: Theme.spacingSmall
            anchors.bottom: parent.bottom
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.margins: Theme.spacingMedium
            clip: true
            model: root.diffText === "" ? [] : root.diffText.split("\n")

            Text {
                anchors.centerIn: parent
                visible: root.diffText === ""
                text: root.loading
                      ? qsTr("Carregando diff...")
                      : (root.emptyText !== ""
                         ? root.emptyText
                         : (root.tracked
                            ? qsTr("Sem mudanças em relação ao HEAD.")
                            : qsTr("Arquivo novo (ainda não versionado).")))
                color: Theme.textMuted
                font.pixelSize: 12
            }

            delegate: Text {
                id: diffLine

                required property string modelData

                width: diffView.width
                text: diffLine.modelData === "" ? " " : diffLine.modelData
                color: root.lineColor(diffLine.modelData)
                font.family: Theme.monoFont
                font.pixelSize: 12
                wrapMode: Text.WrapAnywhere
            }
        }
    }

    Keys.onEscapePressed: root.dismissRequested()
}
