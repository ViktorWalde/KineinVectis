pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Environment inspector of the active file, mounted on first use.
KvPanelFrame {
    id: root

    property var controller: null
    readonly property var context: root.controller.fileContext

    panelWidth: 780
    panelHeight: 600
    onVisibleChanged: visible ? focusKeeper.take() : focusKeeper.giveBack()

    KvFocusKeeper {
        id: focusKeeper
        target: root
    }

    Column {
        id: header
        width: parent.width
        spacing: Theme.spacingSmall

        Row {
            width: parent.width
            spacing: Theme.spacingSmall

            Text {
                width: parent.width - closeButton.width - Theme.spacingSmall
                text: qsTr("Contexto de compilação do arquivo")
                color: Theme.textPrimary
                font.pixelSize: Theme.fontSizeLarge
                font.bold: true
                wrapMode: Text.WordWrap
            }

            KvButton {
                id: closeButton
                text: qsTr("Fechar")
                compact: true
                onClicked: root.controller.closeInspector()
            }
        }

        TextEdit {
            width: parent.width
            text: root.controller.contextPath || qsTr("Abra um arquivo para inspecionar seu contexto.")
            readOnly: true
            selectByMouse: true
            wrapMode: TextEdit.WrapAnywhere
            textFormat: TextEdit.PlainText
            color: Theme.textSecondary
            font.family: Theme.monoFont
            font.pixelSize: Theme.fontSizeSmall
        }

        Text {
            width: parent.width
            visible: root.controller.contextLoading
            text: qsTr("Carregando o contexto…")
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeBody
        }

        Text {
            width: parent.width
            visible: !!root.context.hint
            text: root.context.hint || ""
            color: Theme.warningSoft
            font.pixelSize: Theme.fontSizeBody
            wrapMode: Text.WordWrap
            textFormat: Text.PlainText
        }

        Text {
            width: parent.width
            visible: !!(root.context.python || {}).warning
            text: (root.context.python || {}).warning || ""
            color: Theme.warningSoft
            font.pixelSize: Theme.fontSizeSmall
            wrapMode: Text.WordWrap
            textFormat: Text.PlainText
        }

        Flow {
            width: parent.width
            spacing: Theme.spacingSmall

            KvButton {
                text: qsTr("Abrir CDB na unidade")
                enabled: !!root.context.cdbLocation && !root.controller.contextLoading
                onClicked: root.controller.openContextCdb()
            }

            KvButton {
                text: qsTr("Abrir kit")
                enabled: !!(root.context.effective || {}).kit && !root.controller.contextLoading
                onClicked: root.controller.openContextKit()
            }
        }
    }

    ListView {
        id: factsList
        anchors.top: header.bottom
        anchors.topMargin: Theme.spacingMedium
        anchors.bottom: parent.bottom
        width: parent.width - 14
        clip: true
        spacing: Theme.spacingMedium
        boundsBehavior: Flickable.StopAtBounds
        model: root.controller.contextFacts.rows
        onModelChanged: positionViewAtBeginning()

        delegate: Column {
            id: factRow
            required property var modelData
            width: ListView.view.width
            spacing: Theme.spacingXSmall

            Text {
                width: parent.width
                text: factRow.modelData.label + (factRow.modelData.origin ? " · " + factRow.modelData.origin : "")
                color: factRow.modelData.warning ? Theme.warningSoft : Theme.textMuted
                font.pixelSize: Theme.fontSizeSmall
                wrapMode: Text.WordWrap
                textFormat: Text.PlainText
            }

            TextEdit {
                width: parent.width
                text: factRow.modelData.value
                readOnly: true
                selectByMouse: true
                wrapMode: TextEdit.WrapAnywhere
                textFormat: TextEdit.PlainText
                color: Theme.textPrimary
                font.pixelSize: Theme.fontSizeBody
                font.family: Theme.monoFont
            }
        }
    }

    VerticalScrollBar {
        anchors.right: parent.right
        anchors.top: factsList.top
        anchors.bottom: factsList.bottom
        contentSize: factsList.contentHeight
        viewportSize: factsList.height
        position: factsList.contentY - factsList.originY
        onMoveRequested: position => factsList.contentY = factsList.originY + position
    }
}
