pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

Column {
    id: root
    property var controller: null
    property var selection: null
    property bool outputVisible: false
    spacing: Theme.spacingSmall

    Text {
        text: qsTr("USB DFU")
        color: Theme.textSecondary
        font.pixelSize: Theme.fontSizeSmall
        font.bold: true
    }
    Text {
        width: parent.width
        text: qsTr("Consulte os modos e interfaces USB anunciados pela placa. A consulta não grava nem reinicia.")
        wrapMode: Text.WordWrap
        color: Theme.textMuted
        font.pixelSize: Theme.fontSizeCaption
    }
    KvButton {
        objectName: "embeddedDfuRefresh"
        text: qsTr("Verificar USB DFU")
        compact: true
        enabled: root.controller !== null && !root.controller.busy
        onClicked: root.controller.refresh()
    }
    KvVerdict {
        objectName: "embeddedDfuVerdict"
        width: parent.width
        busy: root.controller !== null && root.controller.busy
        busyText: qsTr("Consultando USB DFU…")
        neutral: root.controller !== null && root.controller.complete
        message: !root.controller ? "" : root.controller.errorText !== "" ? root.controller.errorText
            : root.controller.complete ? (root.controller.hint !== "" ? root.controller.hint
                : root.controller.verdict) : ""
        codeFont: false
        animateChanges: true
    }
    Repeater {
        model: root.controller ? root.controller.interfaces : []
        Column {
            id: row
            required property var modelData
            width: root.width
            spacing: Theme.spacingSmall
            KvToggleChip {
                objectName: "dfuAlternative_" + row.modelData.path + "_" + row.modelData.alternate
                visible: root.selection !== null && root.selection.selectable(row.modelData)
                active: root.selection !== null && root.selection.key(root.selection.destination) === root.selection.key(row.modelData)
                labelText: active ? qsTr("Escolhida") : qsTr("Escolher alternativa")
                onToggled: root.selection.select(row.modelData)
            }
            Text {
                width: parent.width
                text: root.controller.summary(row.modelData)
                textFormat: Text.PlainText
                wrapMode: Text.Wrap
                color: Theme.textPrimary
                font.pixelSize: Theme.fontSizeSmall
            }
            Text {
                width: parent.width
                text: row.modelData.name + "\n" + qsTr("Caminho USB: %1 · serial: %2")
                    .arg(row.modelData.path || qsTr("não informado"))
                    .arg(row.modelData.serial || qsTr("não informado"))
                textFormat: Text.PlainText
                wrapMode: Text.WrapAnywhere
                color: Theme.textSecondary
                font.pixelSize: Theme.fontSizeCaption
            }
        }
    }
    Text {
        width: parent.width
        visible: root.controller !== null && root.controller.tool !== ""
        text: root.controller ? root.controller.tool : ""
        textFormat: Text.PlainText
        wrapMode: Text.WrapAnywhere
        color: Theme.textMuted
        font.pixelSize: Theme.fontSizeCaption
    }
    KvButton {
        text: root.outputVisible ? qsTr("Ocultar saída da consulta") : qsTr("Mostrar saída da consulta")
        compact: true
        visible: root.controller !== null && root.controller.rawOutput !== ""
        onClicked: root.outputVisible = !root.outputVisible
    }
    Text {
        width: parent.width
        visible: root.outputVisible && root.controller !== null && root.controller.rawOutput !== ""
        text: root.controller ? root.controller.rawOutput.trim() : ""
        textFormat: Text.PlainText
        wrapMode: Text.WrapAnywhere
        color: Theme.textMuted
        font.family: Theme.monoFont
        font.pixelSize: Theme.fontSizeCaption
    }
}
