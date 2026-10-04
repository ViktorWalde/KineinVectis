pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// As Configuracoes (redesenhadas em 2026-10-03, passo 4 do roadmap 59).
//
//   ⚙ Configurações                                              ×
//     Preferências da IDE — valem para todos os projetos
//   ┌────────────┬────────────────────────────────────────────────┐
//   │▌✎ Editor   │ fonte (com previa), salvar, formatar, pares     │
//   │ ▶ Build    │ perfil de rigor, cada opcao dizendo o que faz   │
//   │ ▭ Interface│ a animacao da tela de boas-vindas               │
//   └────────────┴────────────────────────────────────────────────┘
//
// Antes era uma lista corrida numa coluna estreita, com o −/+ desenhado a
// mao e so' a bolinha do interruptor respondendo. Agora: secoes com icone,
// a linha inteira clicavel e acesa ao pairar, e o selo "neste projeto" onde
// o valor vem do .kinein/settings.json (e' la' que a troca grava).
//
// Componente burro: recebe os valores efetivos e emite settingChanged(key,
// value); o pai encaminha ao SettingsController.setWhereItLives.
Item {
    id: root

    property bool formatOnSave: false
    property bool autoSave: true
    property int editorFontSize: 14
    property bool autoClosePairs: true
    property string rigorProfile: "strict"
    property bool welcomeAnimation: true
    property bool grafanaWebView: false
    // As chaves que o projeto aberto define no proprio settings (o pai passa
    // todas; aqui ficam so' as que esta tela mostra — o layout tambem e' do
    // projeto e nao tem selo).
    property var projectKeys: []
    readonly property var shownProjectKeys: root.projectKeys.filter(function(key) {
        return ["editorFontSize", "autoSave", "formatOnSave", "autoClosePairs", "rigorProfile"].indexOf(key) >= 0;
    })
    property real maxAvailableWidth: 900
    property real maxAvailableHeight: 600
    property string page: "editor"

    readonly property var pages: [
        { key: "editor", label: qsTr("Editor"), icon: "file" },
        { key: "build", label: qsTr("Build"), icon: "build" },
        { key: "interface", label: qsTr("Interface"), icon: "desktop" }
    ]

    signal dismissRequested()
    signal settingChanged(string key, var value)

    onVisibleChanged: if (visible) forceActiveFocus()
    Keys.onEscapePressed: root.dismissRequested()

    KvBackdrop {
        anchors.fill: parent
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        onClicked: root.dismissRequested()
    }

    Rectangle {
        id: frame

        anchors.centerIn: parent
        width: Math.min(700, root.maxAvailableWidth)
        height: Math.min(500, root.maxAvailableHeight)
        radius: Theme.radiusLarge
        color: Theme.background2
        border.color: Theme.borderStrong
        border.width: 1

        // O clique dentro nao fecha (o fundo fecha).
        MouseArea {
            anchors.fill: parent
        }

        KvIcon {
            id: titleIcon

            anchors.left: parent.left
            anchors.top: parent.top
            anchors.leftMargin: Theme.spacingLarge
            anchors.topMargin: Theme.spacingLarge
            name: "settings"
            size: 20
            active: true
        }

        Column {
            id: header

            anchors.left: titleIcon.right
            anchors.leftMargin: Theme.spacingMedium
            anchors.right: closeButton.left
            anchors.top: parent.top
            anchors.topMargin: Theme.spacingMedium + 2
            spacing: 2

            Text {
                text: qsTr("Configurações")
                color: Theme.textPrimary
                font.pixelSize: Theme.fontSizeLarge
                font.bold: true
            }

            Text {
                width: parent.width
                text: root.shownProjectKeys.length > 0
                      ? qsTr("Valem para todos os projetos; o selo “neste projeto” marca o que este projeto define.")
                      : qsTr("Valem para todos os projetos.")
                color: Theme.textMuted
                font.pixelSize: Theme.fontSizeCaption
                elide: Text.ElideRight
            }
        }

        KvIconButton {
            id: closeButton

            anchors.top: parent.top
            anchors.right: parent.right
            anchors.margins: Theme.spacingSmall
            iconName: "close"
            tooltip: qsTr("Fechar (Esc)")
            onClicked: root.dismissRequested()
        }

        // ---- as secoes -----------------------------------------------------
        Column {
            id: nav

            anchors.top: header.bottom
            anchors.left: parent.left
            anchors.bottom: parent.bottom
            anchors.topMargin: Theme.spacingLarge
            anchors.margins: Theme.spacingMedium
            width: 168
            spacing: 2

            Repeater {
                model: root.pages

                delegate: Rectangle {
                    id: navItem

                    required property var modelData

                    readonly property bool current: root.page === navItem.modelData.key

                    width: nav.width
                    height: 34
                    radius: Theme.radius
                    color: navItem.current ? Theme.surfaceSelected : (navArea.containsMouse ? Theme.surface2 : "transparent")
                    Accessible.role: Accessible.PageTab
                    Accessible.name: navItem.modelData.label

                    Behavior on color {
                        ColorAnimation { duration: Theme.motionFast }
                    }

                    Rectangle {
                        anchors.left: parent.left
                        anchors.verticalCenter: parent.verticalCenter
                        width: 3
                        height: navItem.current ? parent.height - 14 : 0
                        radius: width / 2
                        color: Theme.accent

                        Behavior on height {
                            NumberAnimation { duration: Theme.motionFast }
                        }
                    }

                    KvIcon {
                        id: navIcon

                        anchors.left: parent.left
                        anchors.leftMargin: Theme.spacingMedium
                        anchors.verticalCenter: parent.verticalCenter
                        name: navItem.modelData.icon
                        size: 16
                        active: navItem.current
                    }

                    Text {
                        anchors.left: navIcon.right
                        anchors.leftMargin: Theme.spacingSmall
                        anchors.verticalCenter: parent.verticalCenter
                        text: navItem.modelData.label
                        color: navItem.current ? Theme.textPrimary : Theme.textSecondary
                        font.pixelSize: Theme.fontSizeBody
                        font.weight: navItem.current ? Font.DemiBold : Font.Normal
                    }

                    MouseArea {
                        id: navArea

                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            root.page = navItem.modelData.key;
                            pageScroll.contentY = 0;
                        }
                    }
                }
            }
        }

        Rectangle {
            id: divider

            anchors.top: nav.top
            anchors.bottom: parent.bottom
            anchors.bottomMargin: Theme.spacingMedium
            anchors.left: nav.right
            anchors.leftMargin: Theme.spacingMedium
            width: 1
            color: Theme.borderSoft
        }

        // ---- a pagina -------------------------------------------------------
        Flickable {
            id: pageScroll

            anchors.top: nav.top
            anchors.left: divider.right
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            anchors.leftMargin: Theme.spacingLarge
            anchors.rightMargin: Theme.spacingLarge
            anchors.bottomMargin: Theme.spacingMedium
            clip: true
            contentWidth: width
            contentHeight: pageColumn.implicitHeight
            boundsBehavior: Flickable.StopAtBounds
            // Nunca alem do fim; trocar de seccao volta ao topo (o mesmo
            // defeito achado no Remoto em 2026-10-04).
            onContentHeightChanged: pageScroll.returnToBounds()

            Column {
                id: pageColumn

                width: pageScroll.width
                spacing: Theme.spacingMedium

                Text {
                    text: root.pages.filter(function(entry) { return entry.key === root.page; })[0].label
                    color: Theme.textPrimary
                    font.pixelSize: Theme.fontSizePanelTitle
                    font.bold: true
                }

                SettingsEditorPage {
                    width: parent.width
                    visible: root.page === "editor"
                    settings: root
                }

                SettingsBuildPage {
                    width: parent.width
                    visible: root.page === "build"
                    settings: root
                }

                SettingsInterfacePage {
                    width: parent.width
                    visible: root.page === "interface"
                    settings: root
                }
            }
        }
    }
}
