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

    // 0.3.7 F2 (roadmap 53 §5.5): o painel de baixo CONTEXTUAL. Sempre
    // aparecem as abas de `alwaysVisible`, a ativa e as que o usuario fixou;
    // as outras, quando ha' atividade (`facts`, lidos pelo host de quem sabe).
    // Escondida nao e' perdida: o menu Exibir, a paleta e o atalho continuam
    // abrindo, e abrir a torna a ativa. Quais ficam SEMPRE e' a proposta do 53
    // §13 item 2 (Terminal e Problemas), aplicada como dado ate' o autor
    // confirmar.
    readonly property var alwaysVisible: ["terminal", "problems"]
    property var facts: ({})
    property var pinnedTabs: []

    readonly property var allTabs: [
        { key: "terminal", label: qsTr("Terminal"), icon: "terminal" },
        { key: "build", label: qsTr("Build"), icon: "build" },
        { key: "problems", label: qsTr("Problemas"), icon: "problems" },
        { key: "tests", label: qsTr("Testes"), icon: "test" },
        { key: "jobs", label: qsTr("Jobs"), icon: "run" },
        { key: "debug", label: qsTr("Debug"), icon: "debug" },
        { key: "search", label: qsTr("Busca"), icon: "search" },
        { key: "tools", label: qsTr("Ferramentas"), icon: "tools" },
        { key: "logs", label: qsTr("IDE"), icon: "file" }
    ]

    function listHas(list, key) {
        if (list === undefined || list === null) return false;
        for (let i = 0; i < list.length; i++) {
            if (list[i] === key) return true;
        }
        return false;
    }

    // Funcao pura: as abas a desenhar, na ordem de `all`.
    function visibleTabs(all, active, facts, pinned, always) {
        return all.filter(function(tab) {
            return tab.key === active || tabBar.listHas(always, tab.key)
                || tabBar.listHas(pinned, tab.key) || facts[tab.key] === true;
        });
    }

    // O menu do botao direito numa aba.
    function tabMenuItems(key) {
        const pinnedNow = listHas(pinnedTabs, key);
        return [
            // O rotulo diz o efeito: fixar e' "fica sempre", desafixar e' "some
            // quando nao houver atividade" (0.3.9, prova com mouse real).
            pinnedNow ? { label: qsTr("Desafixar — some sem uso"),
                          action: "bottom.unpin:" + key, enabled: true }
                      : { label: qsTr("Fixar — sempre visível"), action: "bottom.pin:" + key,
                          enabled: !listHas(alwaysVisible, key) }
        ];
    }

    signal tabMenuRequested(string key, real menuX, real menuY)
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

    // As abas param antes das acoes da direita: a 1024 px "IDE" batia no x.
    Flickable {
        id: faixa

        anchors.left: parent.left
        anchors.right: acoesDireita.left
        anchors.rightMargin: Theme.spacingSmall
        anchors.verticalCenter: parent.verticalCenter
        height: 24
        clip: true
        contentWidth: tabs.width
        contentHeight: height
        boundsBehavior: Flickable.StopAtBounds
        flickableDirection: Flickable.HorizontalFlick
        interactive: tabs.width > width

    Row {
        id: tabs

        anchors.verticalCenter: parent.verticalCenter
        spacing: 2

        Repeater {
            model: tabBar.visibleTabs(tabBar.allTabs, tabBar.activeTab, tabBar.facts,
                                      tabBar.pinnedTabs, tabBar.alwaysVisible)

            delegate: Rectangle {
                id: bottomTab

                required property var modelData

                readonly property bool active: tabBar.activeTab === modelData.key
                readonly property string badge: tabBar.badgeFor(modelData.key)
                readonly property bool pinned: tabBar.listHas(tabBar.pinnedTabs, modelData.key)
                                               && !tabBar.listHas(tabBar.alwaysVisible, modelData.key)

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

                    // Fixada pelo botao direito: o alfinete diz por que ela nao some.
                    KvIcon {
                        anchors.verticalCenter: parent.verticalCenter
                        visible: bottomTab.pinned
                        name: "pin"
                        size: 10
                        iconColor: Theme.textMuted
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
                    acceptedButtons: Qt.LeftButton | Qt.RightButton
                    cursorShape: Qt.PointingHandCursor
                    onClicked: function(mouse) {
                        if (mouse.button === Qt.RightButton) {
                            // Embaixo da aba, sem cobri-la.
                            const pos = mapToItem(tabBar, 0, bottomTab.height + Theme.spacingXSmall);
                            tabBar.tabMenuRequested(bottomTab.modelData.key, pos.x, pos.y);
                        } else {
                            tabBar.tabRequested(bottomTab.modelData.key);
                        }
                    }
                }
            }
        }
    }

    }

    Row {
        id: acoesDireita

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
