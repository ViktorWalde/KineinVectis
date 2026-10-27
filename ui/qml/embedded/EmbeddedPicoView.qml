pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

Column {
    id: root
    property var controller: null
    property bool outputVisible: false
    spacing: Theme.spacingSmall
    Text {
        text: qsTr("RP2040 · picotool")
        color: Theme.textSecondary
        font.pixelSize: Theme.fontSizeSmall
        font.bold: true
    }
    Text {
        width: parent.width
        text: qsTr("Entre em BOOTSEL antes de consultar. A consulta acessa a interface USB e lê informações; não força reinício nem grava firmware. Suporte inicial: RP2040 2e8a:0003, picotool 2.2.0/2.3.1.")
        wrapMode: Text.WordWrap
        color: Theme.textMuted
        font.pixelSize: Theme.fontSizeCaption
    }
    KvButton {
        objectName: "embeddedPicoRefresh"
        text: qsTr("Consultar RP2040 BOOTSEL")
        compact: true
        enabled: root.controller !== null && !root.controller.busy
        onClicked: root.controller.refresh()
    }
    KvVerdict {
        width: parent.width
        visible: root.controller !== null && root.controller.verdict !== ""
        busy: root.controller !== null && root.controller.busy
        busyText: root.controller ? root.controller.verdict : ""
        neutral: root.controller !== null && root.controller.complete
        message: root.controller ? root.controller.verdict : ""
        codeFont: false
        animateChanges: true
    }
    Repeater {
        model: root.controller ? root.controller.devices : []
        Text {
            required property var modelData
            width: root.width
            text: root.controller.summary(modelData)
            textFormat: Text.PlainText
            wrapMode: Text.WrapAnywhere
            color: Theme.textPrimary
            font.pixelSize: Theme.fontSizeSmall
        }
    }
    Text {
        width: parent.width
        visible: text !== ""
        text: root.controller ? root.controller.errorText || root.controller.hint : ""
        textFormat: Text.PlainText
        wrapMode: Text.WordWrap
        color: Theme.textMuted
        font.pixelSize: Theme.fontSizeCaption
    }
    KvButton {
        text: root.outputVisible ? qsTr("Ocultar saída do picotool") : qsTr("Mostrar saída do picotool")
        compact: true
        visible: root.controller !== null && root.controller.rawOutput !== ""
        onClicked: root.outputVisible = !root.outputVisible
    }
    Text {
        width: parent.width
        visible: root.outputVisible
        text: root.controller ? root.controller.rawOutput : ""
        textFormat: Text.PlainText
        wrapMode: Text.WrapAnywhere
        color: Theme.textMuted
        font.family: Theme.monoFont
        font.pixelSize: Theme.fontSizeCaption
    }
}
