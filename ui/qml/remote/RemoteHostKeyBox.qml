pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// QUEM O SERVIDOR DIZ SER, na primeira conexao (0.153.0, 2026-10-04). A
// impressao digital da chave mais forte (a que o ssh vai usar) e como
// conferi-la; confiar e' a acao principal do cartao, logo abaixo.
//
//   PRIMEIRA CONEXÃO
//   [127.0.0.1]:2222 se apresenta como
//   ED25519  SHA256:WWfk8uLZVhkQmDlfy04yW2eCd+BOnypfRYABE2Giz7I
//   confira no servidor: ssh-keygen -lf /etc/ssh/ssh_host_ed25519_key.pub
Rectangle {
    id: root

    property var trust: null

    readonly property var strongest: root.trust !== null && root.trust.keys.length > 0 ? root.trust.keys[0] : null

    height: column.implicitHeight + 2 * Theme.spacingSmall
    radius: Theme.radius
    color: Theme.background1
    border.width: 1
    border.color: Theme.warningSoft

    Column {
        id: column

        anchors.fill: parent
        anchors.margins: Theme.spacingSmall
        spacing: 2

        Text {
            text: qsTr("PRIMEIRA CONEXÃO")
            color: Theme.warningSoft
            font.pixelSize: Theme.fontSizeMicro
            font.weight: Font.DemiBold
            font.letterSpacing: 0.6
        }

        Text {
            width: parent.width
            visible: root.trust !== null && root.trust.loading
            text: qsTr("lendo a impressão digital do servidor…")
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeCaption
        }

        Text {
            width: parent.width
            visible: root.strongest !== null
            text: qsTr("%1 se apresenta como").arg(root.trust ? root.trust.host : "")
            color: Theme.textSecondary
            font.pixelSize: Theme.fontSizeCaption
            elide: Text.ElideRight
        }

        Text {
            width: parent.width
            visible: root.strongest !== null
            wrapMode: Text.WrapAnywhere
            text: root.strongest ? root.strongest.kind + "  " + root.strongest.fingerprint : ""
            color: Theme.textPrimary
            font.family: Theme.monoFont
            font.pixelSize: Theme.fontSizeCaption
        }

        Text {
            width: parent.width
            visible: root.strongest !== null
            wrapMode: Text.WordWrap
            text: qsTr("Confira no próprio servidor com `ssh-keygen -lf /etc/ssh/ssh_host_%1_key.pub`, ou pergunte a quem o administra.")
                  .arg(root.strongest ? root.strongest.kind.toLowerCase() : "ed25519")
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeMicro
        }

        Text {
            width: parent.width
            visible: root.trust !== null && root.trust.errorText !== ""
            wrapMode: Text.WordWrap
            text: root.trust ? root.trust.errorText : ""
            color: Theme.errorSoft
            font.pixelSize: Theme.fontSizeCaption
        }
    }
}
