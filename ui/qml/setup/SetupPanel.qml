pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import KineinVectis

// Catalogue presentation; detection and filtering belong to SetupController.
FocusScope {
    id: root

    property string distroName: ""
    property var tools: []
    property int totalCount: 0
    property bool terminalAvailable: false
    property bool busy: false
    property string query: ""
    property string availability: "all"
    property string expandedId: ""
    property string errorText: ""

    signal queryEdited(string text)
    signal availabilityRequested(string value)
    signal refreshRequested()
    signal toggleRequested(string id)
    signal commandRequested(string command)
    signal closeRequested()

    function focusArea() { search.forceActiveFocus(); }
    Keys.onEscapePressed: root.closeRequested()

    KvPanelHeader {
        id: header
        width: parent.width
        title: qsTr("Instalar ferramentas")
        subtitle: root.distroName !== "" ? qsTr("Detectado: %1").arg(root.distroName) : qsTr("Ambiente local")
        onCloseRequested: root.closeRequested()
    }

    KvTextField {
        id: search
        objectName: "setupSearch"
        anchors.top: header.bottom
        anchors.topMargin: Theme.spacingSmall
        width: parent.width
        placeholder: qsTr("Buscar ferramenta…")
        codeFont: false
        activeFocusOnTab: true
        text: root.query
        onEdited: text => root.queryEdited(text)
        onAccepted: list.forceActiveFocus()
    }

    Flow {
        id: filters
        anchors.top: search.bottom
        anchors.topMargin: Theme.spacingSmall
        width: parent.width
        spacing: Theme.spacingXSmall

        Repeater {
            model: [ { value: "all", label: qsTr("Todas") },
                     { value: "missing", label: qsTr("Ausentes") },
                     { value: "installed", label: qsTr("Instaladas") } ]
            delegate: KvButton {
                required property var modelData
                compact: true
                activeFocusOnTab: true
                text: modelData.label
                selected: root.availability === modelData.value
                onClicked: root.availabilityRequested(modelData.value)
            }
        }

        KvIconButton {
            objectName: "setupRefresh"
            compact: true
            activeFocusOnTab: true
            iconName: "refresh"
            tooltip: qsTr("Atualizar detecção")
            enabled: !root.busy
            onClicked: root.refreshRequested()
        }
    }

    Text {
        id: status
        anchors.top: filters.bottom
        anchors.topMargin: Theme.spacingSmall
        width: parent.width
        wrapMode: Text.WordWrap
        textFormat: Text.PlainText
        text: root.busy ? qsTr("Detectando ferramentas…")
              : root.errorText !== "" ? root.errorText
              : root.tools.length === 0 ? qsTr("Nenhuma ferramenta corresponde aos filtros.")
              : qsTr("%1 de %2 ferramentas").arg(root.tools.length).arg(root.totalCount)
        color: root.errorText !== "" ? Theme.errorSoft : Theme.textMuted
        font.pixelSize: Theme.fontSizeCaption
    }

    ListView {
        id: list
        objectName: "setupTools"
        anchors.top: status.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.bottom: footerText.top
        anchors.bottomMargin: Theme.spacingSmall
        width: parent.width
        clip: true
        spacing: Theme.spacingMedium
        boundsBehavior: Flickable.StopAtBounds
        model: root.tools
        reuseItems: true
        activeFocusOnTab: true
        keyNavigationEnabled: true
        visible: !root.busy && root.errorText === ""
        Keys.onReturnPressed: {
            if (currentIndex >= 0 && currentIndex < root.tools.length && root.tools[currentIndex].guide)
                root.toggleRequested(root.tools[currentIndex].id);
        }
        ScrollBar.vertical: ScrollBar { }

        delegate: SetupToolRow {
            required property var modelData
            width: list.width - Theme.spacingMedium
            tool: modelData
            terminalAvailable: root.terminalAvailable
            selected: ListView.isCurrentItem && list.activeFocus
            expanded: modelData.id === root.expandedId
            onToggleRequested: root.toggleRequested(modelData.id)
            onCommandRequested: command => root.commandRequested(command)
        }
    }

    Text {
        id: footerText
        anchors.bottom: parent.bottom
        width: parent.width
        wrapMode: Text.WordWrap
        textFormat: Text.PlainText
        text: root.terminalAvailable
              ? qsTr("Revise o guia e a fonte antes de enviar ao terminal. Atualizar a detecção não instala ferramentas.")
              : qsTr("Abra um projeto para enviar ao terminal da IDE. Você pode consultar e copiar os comandos sem projeto.")
        color: Theme.textMuted
        font.pixelSize: Theme.fontSizeCaption
    }
}
