pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O destino permanece visível enquanto o autor escreve no console.
Rectangle {
    id: root
    property var profile: null
    readonly property string policy: DataSourceKinds.policyLabel(root.profile)
    visible: root.profile !== null
    height: visible ? 28 : 0
    color: Theme.background1

    Text {
        anchors.fill: parent
        anchors.leftMargin: Theme.spacingSmall
        anchors.rightMargin: Theme.spacingSmall
        verticalAlignment: Text.AlignVCenter
        textFormat: Text.PlainText
        text: root.profile ? qsTr("Console · %1 · %2%3").arg(root.profile.name)
            .arg(root.profile.database).arg(root.policy ? " · " + root.policy : "") : ""
        color: root.profile && root.profile.production === true ? Theme.errorSoft : Theme.textSecondary
        font.pixelSize: Theme.fontSizeSmall
        font.weight: Font.DemiBold
        elide: Text.ElideMiddle
    }
}
