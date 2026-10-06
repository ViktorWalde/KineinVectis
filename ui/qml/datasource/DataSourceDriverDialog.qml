pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Aviso antes do primeiro carregamento de codigo nativo de terceiros.
KvPanelFrame {
    id: root

    property var controller: null
    panelWidth: 560
    panelHeight: body.implicitHeight + buttons.height + 3 * Theme.spacingMedium
    onVisibleChanged: if (visible) focusTimer.restart()

    Timer {
        id: focusTimer
        interval: 0
        onTriggered: cancelButton.forceActiveFocus()
    }

    Column {
        id: body
        width: parent.width
        spacing: Theme.spacingMedium

        KvPanelHeader {
            width: parent.width
            title: qsTr("Carregar driver ODBC?")
            subtitle: qsTr("Código nativo instalado nesta máquina")
            onCloseRequested: root.dismissRequested()
        }

        Text {
            width: parent.width
            wrapMode: Text.WrapAnywhere
            text: root.controller && root.controller.pending
                ? qsTr("Conexão: %1\nDSN: %2\nDriver: %3").arg(root.controller.pending.name)
                    .arg(root.controller.pending.dsn).arg(root.controller.pending.driver) : ""
            color: Theme.textPrimary
            font.pixelSize: Theme.fontSizeBody
            textFormat: Text.PlainText
        }

        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            text: qsTr("O driver será executado dentro do processo do core e poderá acessar os dados e as credenciais usados na conexão. Carregue somente um driver em que você confia. A autorização vale para este perfil durante esta sessão.")
            color: Theme.textSecondary
            font.pixelSize: Theme.fontSizeSmall
        }

        Text {
            width: parent.width
            visible: root.controller !== null && root.controller.message !== ""
            wrapMode: Text.WordWrap
            text: root.controller ? root.controller.message : ""
            color: Theme.errorSoft
            font.pixelSize: Theme.fontSizeCaption
            textFormat: Text.PlainText
        }
    }

    Row {
        id: buttons
        anchors.bottom: parent.bottom
        anchors.right: parent.right
        spacing: Theme.spacingSmall

        KvButton {
            id: cancelButton
            activeFocusOnTab: true
            text: qsTr("Cancelar")
            onClicked: root.dismissRequested()
        }

        KvButton {
            activeFocusOnTab: true
            primary: true
            text: root.controller && root.controller.authorizing ? qsTr("Autorizando…") : qsTr("Carregar driver")
            enabled: root.controller !== null && root.controller.canAuthorize
            onClicked: root.controller.confirm()
        }
    }
}
