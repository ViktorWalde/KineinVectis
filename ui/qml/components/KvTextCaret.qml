import QtQuick
import KineinVectis

// O CURSOR do KvTextField (2026-10-04): ambar, com piscar SUAVE (esmaece em
// vez de sumir de uma vez) e firme enquanto se digita — mover o cursor
// reinicia o piscar com ele aceso. Fica fora do campo pela catraca de 300
// linhas e porque e' uma coisa so': o cursor.
Item {
    id: root

    // A entrada que o desenha (o cursorDelegate dela).
    property TextInput input: null

    width: 2

    Rectangle {
        id: bar

        width: 2
        height: root.input ? Math.min(parent.height, root.input.font.pixelSize + 4) : parent.height
        anchors.verticalCenter: parent.verticalCenter
        radius: width / 2
        color: Theme.accent

        SequentialAnimation on opacity {
            id: blink

            running: root.input !== null && root.input.activeFocus && root.input.selectedText === ""
            loops: Animation.Infinite
            alwaysRunToEnd: false

            PauseAnimation { duration: Theme.motionCaretBlink }
            NumberAnimation { to: 0; duration: Theme.motionMedium }
            PauseAnimation { duration: Theme.motionCaretBlink - Theme.motionMedium }
            NumberAnimation { to: 1; duration: Theme.motionMedium }
        }
    }

    Connections {
        target: root.input

        function onCursorPositionChanged() {
            bar.opacity = 1;
            if (blink.running) blink.restart();
        }
    }
}
