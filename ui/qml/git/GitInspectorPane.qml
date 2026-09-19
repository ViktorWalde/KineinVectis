pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O painel da direita da HUD do Git: uma mudanca (o diff dela) ou um
// commit (sha, autor, refs, arquivos, patch). Le do GitInspectorController.
Item {
    id: root

    property var inspector: null

    GitRules { id: rules }

    Text {
        anchors.centerIn: parent
        visible: !root.inspector || !root.inspector.active
        width: parent.width - 2 * Theme.spacingLarge
        horizontalAlignment: Text.AlignHCenter
        wrapMode: Text.WordWrap
        text: qsTr("Clique numa mudança para ver o diff, ou num commit para ver o que ele mudou.")
        color: Theme.textMuted
        font.pixelSize: 11
    }

    Column {
        id: cabecalho

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        visible: root.inspector && root.inspector.active
        spacing: Theme.spacingXSmall

        Row {
            width: parent.width
            spacing: Theme.spacingSmall

            Text {
                anchors.verticalCenter: parent.verticalCenter
                visible: root.inspector && root.inspector.isCommit
                text: root.inspector ? root.inspector.shortSha : ""
                color: Theme.accent
                font.family: Theme.monoFont
                font.pixelSize: 11
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                width: parent.width - x
                text: root.inspector ? root.inspector.summary : ""
                color: Theme.textPrimary
                font.pixelSize: 12
                font.weight: Font.DemiBold
                elide: Text.ElideMiddle
            }
        }

        Text {
            width: parent.width
            visible: root.inspector && root.inspector.isCommit
            text: root.inspector ? root.inspector.author + " · " + root.inspector.age : ""
            color: Theme.textMuted
            font.pixelSize: 10
        }

        Flow {
            width: parent.width
            spacing: Theme.spacingXSmall
            visible: root.inspector && root.inspector.isCommit && root.inspector.refs.length > 0

            Repeater {
                model: root.inspector ? root.inspector.refs : []

                delegate: Rectangle {
                    id: chip

                    required property var modelData

                    readonly property var ref: rules.refChip(modelData)

                    width: chipLabel.implicitWidth + 2 * Theme.spacingSmall
                    height: 16
                    radius: 8
                    color: ref.head ? Theme.accentDim : (ref.tag ? Theme.purpleOrbital : Theme.surface2)
                    opacity: 0.9

                    Text {
                        id: chipLabel

                        anchors.centerIn: parent
                        text: chip.ref.name
                        color: chip.ref.head || chip.ref.tag ? Theme.background0 : Theme.textSecondary
                        font.pixelSize: 9
                        font.bold: chip.ref.head
                    }
                }
            }
        }

        // Os arquivos do commit, com +/-: a grade comum (F8).
        KvDataGrid {
            width: parent.width
            visible: root.inspector && root.inspector.isCommit && root.inspector.files.length > 0
            columns: [
                { key: "path", label: qsTr("arquivo") },
                { key: "added", label: "+" },
                { key: "removed", label: "−" }
            ]
            rows: root.inspector ? root.inspector.files : []
            maxHeight: 96
        }
    }

    GitPatchView {
        anchors.top: cabecalho.visible ? cabecalho.bottom : parent.top
        anchors.topMargin: cabecalho.visible ? Theme.spacingSmall : 0
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        visible: root.inspector && root.inspector.active
        patch: root.inspector ? root.inspector.patch : ""
        loading: root.inspector ? root.inspector.loading : false
        emptyText: root.inspector && !root.inspector.tracked
                   ? qsTr("Arquivo novo (ainda não versionado).")
                   : qsTr("Sem mudanças em relação ao HEAD.")
    }
}
