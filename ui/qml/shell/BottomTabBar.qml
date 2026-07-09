pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

Row {
    id: tabBar

    property string activeTab: ""
    property int problemCount: 0

    signal tabRequested(string tab)
    signal refreshToolsRequested()

    spacing: Theme.spacingSmall

    Repeater {
        model: [
            { key: "build", label: qsTr("Build") },
            { key: "jobs", label: qsTr("Jobs") },
            { key: "problems", label: qsTr("Problemas") },
            { key: "tests", label: qsTr("Testes") },
            { key: "run", label: qsTr("Executar") },
            { key: "terminal", label: qsTr("Terminal") },
            { key: "search", label: qsTr("Busca") },
            { key: "logs", label: qsTr("IDE") },
            { key: "tools", label: qsTr("Ferramentas") }
        ]

        delegate: Rectangle {
            required property var modelData

            width: bottomTabLabel.width + 2 * Theme.spacingSmall
            height: 20
            radius: Theme.radius
            color: tabBar.activeTab === modelData.key ? Theme.accentDim : "transparent"
            border.color: Theme.borderSoft
            border.width: 1

            Text {
                id: bottomTabLabel

                anchors.centerIn: parent
                text: parent.modelData.key === "problems" && tabBar.problemCount > 0
                      ? qsTr("Problemas (%1)").arg(tabBar.problemCount)
                      : parent.modelData.label
                color: tabBar.activeTab === parent.modelData.key
                       ? Theme.accent : Theme.textSecondary
                font.pixelSize: 10
                font.bold: true
            }

            MouseArea {
                anchors.fill: parent
                cursorShape: Qt.PointingHandCursor
                onClicked: tabBar.tabRequested(parent.modelData.key)
            }
        }
    }

    Rectangle {
        width: refreshToolsLabel.width + 2 * Theme.spacingSmall
        height: 20
        radius: Theme.radius
        visible: tabBar.activeTab === "tools"
        color: refreshToolsArea.containsMouse ? Theme.surface2 : "transparent"
        border.color: Theme.borderSoft
        border.width: 1

        Text {
            id: refreshToolsLabel

            anchors.centerIn: parent
            text: qsTr("⟳ redetectar")
            color: Theme.textSecondary
            font.pixelSize: 10
        }

        MouseArea {
            id: refreshToolsArea

            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: tabBar.refreshToolsRequested()
        }
    }
}
