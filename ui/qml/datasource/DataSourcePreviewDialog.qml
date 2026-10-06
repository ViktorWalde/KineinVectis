pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A amostra vem do RETURNING do motor; nenhuma decisão/parsing de SQL na View.
KvPanelFrame {
    id: root
    property var controller: null
    readonly property var sample: root.controller ? root.controller.active : null
    panelWidth: 780
    panelHeight: 570
    contentHeight: body.implicitHeight + buttons.height + Theme.spacingMedium
    onVisibleChanged: if (visible) Qt.callLater(() => rollbackButton.forceActiveFocus())

    Column {
        id: body
        width: parent.width
        spacing: Theme.spacingSmall

        Text {
            width: parent.width
            text: qsTr("Prévia PostgreSQL · alterações ainda pendentes")
            color: Theme.warningSoft
            font.pixelSize: Theme.fontSizeLarge
            font.weight: Font.DemiBold
            wrapMode: Text.WordWrap
        }
        Text {
            width: parent.width
            text: root.sample ? qsTr("%1 · %2%3").arg(root.sample.name)
                .arg(root.sample.expectedContext.profile.database)
                .arg(DataSourceKinds.policyLabel(root.sample.expectedContext.profile)
                    ? " · " + DataSourceKinds.policyLabel(root.sample.expectedContext.profile) : "") : ""
            textFormat: Text.PlainText
            color: root.sample && root.sample.expectedContext.profile.production ? Theme.errorSoft : Theme.textSecondary
            font.pixelSize: Theme.fontSizeSmall
            wrapMode: Text.WordWrap
        }
        Text {
            width: parent.width
            text: root.sample ? qsTr("%1 linhas afetadas · amostra de %2%3 · prazo: %4 s").arg(root.sample.affected)
                .arg(root.sample.rows.length).arg(root.sample.truncated ? qsTr(" (limitada)") : "")
                .arg(root.controller.remaining) : ""
            color: Theme.textPrimary
            font.pixelSize: Theme.fontSizeSmall
            wrapMode: Text.WordWrap
        }
        Text {
            width: parent.width
            text: qsTr("Esta escrita já foi executada na transação. Desfazer não recupera sequências nem efeitos externos de funções/triggers. Fechar ou deixar o prazo expirar desfaz as alterações transacionais pendentes.")
            wrapMode: Text.WordWrap
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeCaption
        }
        Rectangle {
            width: parent.width
            height: 76
            color: Theme.background0
            radius: Theme.radius
            clip: true
            Flickable {
                id: sqlScroll
                anchors.fill: parent
                anchors.margins: Theme.spacingSmall
                contentWidth: width
                contentHeight: sqlText.implicitHeight
                boundsBehavior: Flickable.StopAtBounds
                Text {
                    id: sqlText
                    width: sqlScroll.width
                    text: root.sample ? qsTr("Solicitado:\n%1\n\nExecutado:\n%2").arg(root.sample.sql).arg(root.sample.executedSql) : ""
                    textFormat: Text.PlainText
                    wrapMode: Text.WrapAnywhere
                    color: Theme.textPrimary
                    font.family: Theme.monoFont
                    font.pixelSize: Theme.fontSizeSmall
                }
            }
        }
        KvDataGrid {
            width: parent.width
            maxHeight: Math.max(80, root.frameHeight - 310)
            columns: root.sample ? root.sample.columns.map(name => ({ key: name, label: name })) : []
            rows: root.sample ? root.sample.rows : []
            emptyText: qsTr("Nenhuma linha alterada.")
        }
        Text {
            width: parent.width
            visible: root.controller !== null && root.controller.message !== ""
            text: root.controller ? root.controller.message : ""
            textFormat: Text.PlainText
            wrapMode: Text.WordWrap
            color: Theme.warningSoft
            font.pixelSize: Theme.fontSizeCaption
        }
    }
    Row {
        id: buttons
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        spacing: Theme.spacingSmall
        KvButton {
            id: rollbackButton
            text: qsTr("Desfazer (ROLLBACK)")
            activeFocusOnTab: true
            enabled: root.controller !== null && !root.controller.deciding
            onClicked: root.controller.decide("rollback")
        }
        KvButton {
            text: qsTr("Confirmar (COMMIT)")
            activeFocusOnTab: true
            danger: true
            enabled: root.controller !== null && root.controller.canCommit
            onClicked: root.controller.decide("commit")
        }
    }
}
