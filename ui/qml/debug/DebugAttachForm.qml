pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Endpoint entry only. The existing debug controller/core own the session.
Row {
    id: root

    property bool available: false
    property string host: "127.0.0.1"
    property string port: "5678"
    signal attachRequested(string host, real port)

    spacing: Theme.spacingSmall
    height: 42

    DataSourceField {
        width: 180
        label: qsTr("Host do Python")
        value: root.host
        enabled: root.available
        onEdited: function(text) { root.host = text; }
    }
    DataSourceField {
        width: 80
        label: qsTr("Porta")
        value: root.port
        numeric: true
        enabled: root.available
        onEdited: function(text) { root.port = text; }
        onAccepted: {
            if (root.available) root.attachRequested(root.host, Number(root.port));
        }
    }
    KvButton {
        anchors.verticalCenter: parent.verticalCenter
        text: qsTr("Conectar ao Python")
        compact: true
        enabled: root.available
        onClicked: root.attachRequested(root.host, Number(root.port))
    }
}
