import QtQuick
import KineinVectis

// O selo "neste projeto": o valor vem do .kinein/settings.json do projeto, e
// e' la' que a troca vai ser gravada (SettingsController.setWhereItLives).
Rectangle {
    width: badgeText.implicitWidth + 2 * Theme.spacingSmall
    height: 16
    radius: height / 2
    color: "transparent"
    border.width: 1
    border.color: Theme.accentDim

    Text {
        id: badgeText

        anchors.centerIn: parent
        text: qsTr("neste projeto")
        color: Theme.accent
        font.pixelSize: Theme.fontSizeMicro
    }
}
