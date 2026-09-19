pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A faixa de abas do painel de baixo (HUD, 2026-09-18, teste do autor): a
// linguagem das F1-F8 — sem uma borda por chip, a ativa como pilula, e o
// que cada painel TEM como contagem ao lado do nome (Problemas 3, Testes
// 12/14, Jobs 2, a bolinha do Terminal enquanto roda). A ordem e' a do uso
// diario; o log da IDE fica por ultimo. O x da direita esconde o painel.
Item {
    id: tabBar

    property string activeTab: ""
    property int problemCount: 0
    property string testsBadge: ""
    property bool testsOk: true
    property int jobsRunning: 0
    property bool processRunning: false

    signal tabRequested(string tab)
    signal refreshToolsRequested()
    signal hideRequested()

    implicitHeight: 24

    // A regra do que cada aba mostra ao lado do nome, num lugar so'.
    function badgeFor(key) {
        if (key === "problems" && problemCount > 0) return String(problemCount);
        if (key === "tests" && testsBadge !== "") return testsBadge;
        if (key === "jobs" && jobsRunning > 0) return String(jobsRunning);
        return "";
    }

    function badgeColorFor(key) {
        if (key === "problems") return Theme.errorSoft;
        if (key === "tests") return testsOk ? Theme.successSoft : Theme.errorSoft;
        return Theme.accent;
    }

    Row {
        id: tabs

        anchors.left: parent.left
        anchors.verticalCenter: parent.verticalCenter
        spacing: 2

        Repeater {
            model: [
                { key: "terminal", label: qsTr("Terminal"), icon: "terminal" },
                { key: "build", label: qsTr("Build"), icon: "build" },
                { key: "problems", label: qsTr("Problemas"), icon: "problems" },
                { key: "tests", label: qsTr("Testes"), icon: "test" },
                { key: "jobs", label: qsTr("Jobs"), icon: "run" },
                { key: "debug", label: qsTr("Debug"), icon: "debug" },
                { key: "git", label: qsTr("Git"), icon: "git" },
                { key: "search", label: qsTr("Busca"), icon: "search" },
                { key: "tools", label: qsTr("Ferramentas"), icon: "tools" },
                { key: "logs", label: qsTr("IDE"), icon: "file" }
            ]

            delegate: Rectangle {
                id: bottomTab

                required property var modelData

                readonly property bool active: tabBar.activeTab === modelData.key
                readonly property string badge: tabBar.badgeFor(modelData.key)

                width: bottomTabContent.implicitWidth + 2 * Theme.spacingSmall
                height: 24
                radius: Theme.radius
                color: active ? Theme.surfaceSelected
                       : (tabArea.containsMouse ? Theme.surface2 : "transparent")

                Row {
                    id: bottomTabContent

                    anchors.centerIn: parent
                    spacing: Theme.spacingXSmall

                    KvIcon {
                        anchors.verticalCenter: parent.verticalCenter
                        name: bottomTab.modelData.icon
                        size: 13
                        active: bottomTab.active
                    }

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        text: bottomTab.modelData.label
                        color: bottomTab.active ? Theme.textPrimary : Theme.textSecondary
                        font.pixelSize: 11
                        font.weight: bottomTab.active ? Font.DemiBold : Font.Normal
                    }

                    // A contagem do painel, quando ha' o que contar.
                    Rectangle {
                        anchors.verticalCenter: parent.verticalCenter
                        visible: bottomTab.badge !== ""
                        width: badgeText.implicitWidth + 8
                        height: 14
                        radius: 7
                        color: tabBar.badgeColorFor(bottomTab.modelData.key)
                        opacity: 0.9

                        Text {
                            id: badgeText

                            anchors.centerIn: parent
                            text: bottomTab.badge
                            color: Theme.background0
                            font.pixelSize: 9
                            font.bold: true
                        }
                    }

                    // A execucao em curso: a bolinha da aba Terminal.
                    Rectangle {
                        anchors.verticalCenter: parent.verticalCenter
                        visible: bottomTab.modelData.key === "terminal" && tabBar.processRunning
                        width: 6
                        height: 6
                        radius: 3
                        color: Theme.successSoft
                    }
                }

                MouseArea {
                    id: tabArea

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: tabBar.tabRequested(bottomTab.modelData.key)
                }
            }
        }
    }

    Row {
        anchors.right: parent.right
        anchors.verticalCenter: parent.verticalCenter
        spacing: Theme.spacingSmall

        KvButton {
            height: 24
            visible: tabBar.activeTab === "tools"
            compact: true
            iconName: "refresh"
            text: qsTr("Redetectar")
            onClicked: tabBar.refreshToolsRequested()
        }

        KvIconButton {
            compact: true
            iconName: "close"
            tooltip: qsTr("Esconder o painel")
            onClicked: tabBar.hideRequested()
        }
    }
}
