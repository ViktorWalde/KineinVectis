pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

Row {
    id: tabBar

    property string activeTab: ""
    property int problemCount: 0
    property bool processRunning: false

    signal tabRequested(string tab)
    signal refreshToolsRequested()

    spacing: Theme.spacingSmall

    Repeater {
        model: [
            { key: "build", label: qsTr("Build"), icon: "build" },
            { key: "jobs", label: qsTr("Jobs"), icon: "run" },
            { key: "problems", label: qsTr("Problemas"), icon: "problems" },
            { key: "tests", label: qsTr("Testes"), icon: "test" },
            { key: "terminal", label: qsTr("Terminal"), icon: "terminal" },
            { key: "assistant", label: qsTr("Assistente"), icon: "assistant" },
            { key: "debug", label: qsTr("Debug"), icon: "debug" },
            { key: "git", label: qsTr("Git"), icon: "git" },
            { key: "search", label: qsTr("Busca"), icon: "search" },
            { key: "logs", label: qsTr("IDE"), icon: "file" },
            { key: "tools", label: qsTr("Ferramentas"), icon: "tools" }
        ]

        delegate: Rectangle {
            id: bottomTab

            required property var modelData

            width: bottomTabContent.implicitWidth + 2 * Theme.spacingSmall
            height: 26
            radius: Theme.radius
            color: tabBar.activeTab === modelData.key ? Theme.surfaceSelected : "transparent"
            border.color: Theme.borderSoft
            border.width: 1

            Row {
                id: bottomTabContent

                anchors.centerIn: parent
                spacing: Theme.spacingXSmall

                KvIcon {
                    anchors.verticalCenter: parent.verticalCenter
                    name: bottomTab.modelData.icon
                    size: 14
                    active: tabBar.activeTab === bottomTab.modelData.key
                }

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    text: {
                        if (bottomTab.modelData.key === "problems" && tabBar.problemCount > 0) {
                            return qsTr("Problemas (%1)").arg(tabBar.problemCount);
                        }
                        if (bottomTab.modelData.key === "terminal" && tabBar.processRunning) {
                            return bottomTab.modelData.label + " ·";
                        }
                        return bottomTab.modelData.label;
                    }
                    color: tabBar.activeTab === bottomTab.modelData.key
                           ? Theme.accent : Theme.textSecondary
                    font.pixelSize: 10
                    font.bold: true
                }
            }

            MouseArea {
                anchors.fill: parent
                cursorShape: Qt.PointingHandCursor
                onClicked: tabBar.tabRequested(parent.modelData.key)
            }
        }
    }

    KvButton {
        height: 26
        visible: tabBar.activeTab === "tools"
        compact: true
        iconName: "refresh"
        text: qsTr("Redetectar")
        onClicked: tabBar.refreshToolsRequested()
    }
}
