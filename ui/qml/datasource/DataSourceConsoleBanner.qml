pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O destino permanece visível enquanto o autor escreve no console.
Rectangle {
    id: root
    property var profile: null
    signal previewRequested()
    // O historico da conexao (passo 13b): o ponto e' onde o menu abre.
    signal historyRequested(real x, real y)
    readonly property string policy: DataSourceKinds.policyLabel(root.profile)
    visible: root.profile !== null
    height: visible ? 28 : 0
    color: Theme.background1

    Text {
        anchors.fill: parent
        anchors.leftMargin: Theme.spacingSmall
        anchors.rightMargin: buttons.width + 2 * Theme.spacingSmall
        verticalAlignment: Text.AlignVCenter
        textFormat: Text.PlainText
        text: root.profile ? qsTr("Console · %1 · %2%3").arg(root.profile.name)
            .arg(root.profile.database).arg(root.policy ? " · " + root.policy : "") : ""
        color: root.profile && root.profile.production === true ? Theme.errorSoft : Theme.textSecondary
        font.pixelSize: Theme.fontSizeSmall
        font.weight: Font.DemiBold
        elide: Text.ElideMiddle
    }

    Row {
        id: buttons
        anchors.right: parent.right
        anchors.rightMargin: Theme.spacingSmall
        anchors.verticalCenter: parent.verticalCenter
        spacing: Theme.spacingXSmall

        KvButton {
            id: historyButton
            compact: true
            text: qsTr("Histórico")
            tooltip: qsTr("As últimas instruções executadas nesta conexão")
            onClicked: {
                const point = historyButton.mapToItem(root, 0, historyButton.height);
                root.historyRequested(point.x, point.y);
            }
        }

        KvButton {
            compact: true
            text: qsTr("Executar com prévia…")
            visible: root.profile !== null && DataSourceKinds.isPostgres(root.profile.engine) && root.profile.readOnly !== true
            onClicked: root.previewRequested()
        }
    }
}
