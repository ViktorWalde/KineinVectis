pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Escolhe somente nomes devolvidos pelo gerenciador; nenhuma connection string.
Column {
    id: root

    property var sources: []
    property string current: ""
    property bool loading: false
    property string message: ""
    signal selected(string dsn)
    signal refreshRequested()
    spacing: Theme.spacingSmall

    Row {
        width: parent.width
        Text {
            width: parent.width - refreshButton.width - Theme.spacingSmall
            anchors.verticalCenter: parent.verticalCenter
            text: qsTr("DSN registrado no unixODBC")
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeCaption
        }
        KvButton {
            id: refreshButton
            activeFocusOnTab: true
            compact: true
            iconName: "refresh"
            text: root.loading ? qsTr("Lendo…") : qsTr("Atualizar")
            enabled: !root.loading
            onClicked: root.refreshRequested()
        }
    }

    ListView {
        id: list
        width: parent.width
        height: Math.min(4, root.sources.length) * (Theme.spacingLarge + Theme.spacingSmall)
        model: root.sources
        clip: true
        boundsBehavior: Flickable.StopAtBounds
        currentIndex: root.sources.findIndex(source => source.dsn === root.current)
        keyNavigationEnabled: true
        Keys.onReturnPressed: if (currentIndex >= 0) root.selected(root.sources[currentIndex].dsn)
        Keys.onSpacePressed: if (currentIndex >= 0) root.selected(root.sources[currentIndex].dsn)

        FlickableScrollBar { view: list }

        delegate: KvButton {
            id: entry
            required property var modelData
            width: list.width
            height: Theme.spacingLarge + Theme.spacingSmall
            text: entry.modelData.dsn
            tooltip: entry.modelData.driver
            selected: root.current === entry.modelData.dsn
            activeFocusOnTab: true
            onClicked: root.selected(entry.modelData.dsn)
        }
    }

    Text {
        width: parent.width
        wrapMode: Text.WordWrap
        text: root.message !== "" ? root.message : (root.current !== "" ? qsTr("Selecionado: %1").arg(root.current)
             : qsTr("Selecione um DSN. Host, porta, TLS e driver são definidos na configuração do unixODBC."))
        color: Theme.textMuted
        font.pixelSize: Theme.fontSizeCaption
        textFormat: Text.PlainText
    }
}
