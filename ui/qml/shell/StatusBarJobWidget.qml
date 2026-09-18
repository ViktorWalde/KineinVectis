import QtQuick
import KineinVectis

// O job em curso NA BARRA DE STATUS (Etapa 2, F2): titulo, barra de
// progresso (indeterminada quando o job nao mede), a ultima linha e o
// cancelar. E' o "o que esta' acontecendo" que a referencia poe no mesmo
// lugar. Burro: recebe do ActiveJobController.
Row {
    id: root

    property string title: ""
    property real progress: -1
    property string message: ""
    property bool canCancel: false
    property int runningCount: 0

    signal cancelRequested()
    signal jobsRequested()

    spacing: Theme.spacingSmall
    visible: root.title !== ""

    Text {
        anchors.verticalCenter: parent.verticalCenter
        text: root.runningCount > 1 ? qsTr("%1 · %2 jobs").arg(root.title).arg(root.runningCount) : root.title
        color: Theme.accent
        font.pixelSize: Theme.fontSizeStatus

        MouseArea {
            anchors.fill: parent
            cursorShape: Qt.PointingHandCursor
            onClicked: root.jobsRequested()
        }
    }

    // A barra: determinada (0..1) ou o traco que anda, quando o job nao mede.
    Rectangle {
        id: trilho

        anchors.verticalCenter: parent.verticalCenter
        width: 96
        height: 4
        radius: 2
        color: Theme.surface2

        Rectangle {
            id: preenchido

            height: parent.height
            radius: 2
            color: Theme.accent
            width: root.progress >= 0 ? Math.max(4, trilho.width * Math.min(1, root.progress)) : 28
            x: root.progress >= 0 ? 0 : 0

            SequentialAnimation on x {
                running: root.progress < 0 && root.visible
                loops: Animation.Infinite
                NumberAnimation { from: 0; to: trilho.width - 28; duration: 900; easing.type: Easing.InOutSine }
                NumberAnimation { from: trilho.width - 28; to: 0; duration: 900; easing.type: Easing.InOutSine }
            }
        }
    }

    Text {
        anchors.verticalCenter: parent.verticalCenter
        visible: root.message !== ""
        width: Math.min(implicitWidth, 320)
        elide: Text.ElideRight
        text: root.message
        color: Theme.textMuted
        font.pixelSize: Theme.fontSizeStatus
        font.family: Theme.monoFont
    }

    KvIconButton {
        anchors.verticalCenter: parent.verticalCenter
        visible: root.canCancel
        compact: true
        iconName: "close"
        danger: true
        tooltip: qsTr("Cancelar %1").arg(root.title)
        onClicked: root.cancelRequested()
    }
}
