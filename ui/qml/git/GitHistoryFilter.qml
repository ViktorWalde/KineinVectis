pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O filtro do Historico (HUD do Git, fatia 2b): o texto (resumo, autor,
// sha — local) e o branch/tag (pede ao core `git.log { ref }`). Burro:
// estado por property, intencao por signal.
Item {
    id: root

    property string filterText: ""
    property string logRef: ""
    property var branchesModel: null
    property bool loading: false

    signal filterTextEdited(string text)
    signal logRefChosen(string ref)

    property bool refsOpen: false

    implicitHeight: 26

    Row {
        anchors.fill: parent
        spacing: Theme.spacingSmall

        KvTextField {
            id: campo

            width: parent.width - refChip.width - parent.spacing
            height: 24
            codeFont: false
            pixelSize: Theme.fontSizeSmall
            iconName: "search"
            clearable: true
            text: root.filterText
            placeholder: qsTr("Filtrar por mensagem, autor ou sha…")
            onEdited: (text) => root.filterTextEdited(text)
        }

        // O branch de onde o historico parte: HEAD por padrao.
        KvToggleChip {
            id: refChip

            anchors.verticalCenter: parent.verticalCenter
            labelText: root.logRef === "" ? qsTr("HEAD") : root.logRef
            active: root.logRef !== ""
            onToggled: root.refsOpen = !root.refsOpen
        }
    }

    // A lista de branches para escolher (clique fora fecha).
    Rectangle {
        id: lista

        anchors.top: parent.bottom
        anchors.right: parent.right
        anchors.topMargin: Theme.spacingXSmall
        visible: root.refsOpen
        z: 10
        width: 220
        height: Math.min(200, colunaRefs.implicitHeight + 2 * Theme.spacingXSmall)
        radius: Theme.radius
        color: Theme.background2
        border.color: Theme.borderStrong
        border.width: 1

        Flickable {
            anchors.fill: parent
            anchors.margins: Theme.spacingXSmall
            clip: true
            contentHeight: colunaRefs.implicitHeight
            contentWidth: width

            Column {
                id: colunaRefs

                width: parent.width

                Rectangle {
                    width: parent.width
                    height: 22
                    radius: Theme.radiusXSmall
                    color: headArea.containsMouse ? Theme.surface2 : "transparent"

                    Behavior on color {
                        ColorAnimation { duration: Theme.motionFast }
                    }

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        x: Theme.spacingSmall
                        text: qsTr("HEAD (o branch atual)")
                        color: root.logRef === "" ? Theme.accent : Theme.textSecondary
                        font.pixelSize: Theme.fontSizeSmall
                    }

                    MouseArea {
                        id: headArea

                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: { root.refsOpen = false; root.logRefChosen(""); }
                    }
                }

                Repeater {
                    model: root.branchesModel

                    delegate: Rectangle {
                        id: linhaRef

                        required property string name
                        required property bool current

                        width: colunaRefs.width
                        height: 22
                        radius: Theme.radiusXSmall
                        color: refArea.containsMouse ? Theme.surface2 : "transparent"

                        Behavior on color {
                            ColorAnimation { duration: Theme.motionFast }
                        }

                        Text {
                            anchors.verticalCenter: parent.verticalCenter
                            x: Theme.spacingSmall
                            width: parent.width - 2 * Theme.spacingSmall
                            text: linhaRef.name + (linhaRef.current ? qsTr("  (atual)") : "")
                            color: root.logRef === linhaRef.name ? Theme.accent : Theme.textSecondary
                            font.family: Theme.monoFont
                            font.pixelSize: Theme.fontSizeSmall
                            elide: Text.ElideMiddle
                        }

                        MouseArea {
                            id: refArea

                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: { root.refsOpen = false; root.logRefChosen(linhaRef.name); }
                        }
                    }
                }
            }
        }
    }
}
