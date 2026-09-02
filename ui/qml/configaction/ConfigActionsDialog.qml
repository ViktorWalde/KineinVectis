pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Configuration Actions: a caixa de ferramentas contextual do projeto.
//
// Layout da spec 9.1 §12 — lista a esquerda, preview a direita — com o seletor
// de escopo no topo (spec 9.2 §6). E dialogo, e nao aba do painel inferior,
// porque o fluxo e modal por natureza: explicar, previsualizar, consentir,
// aplicar. Sem o consentimento no meio, nao ha Configuration Action; ha um
// botao que mexe no CMakeLists do usuario.
Item {
    id: root

    property var controller: null
    property real maxAvailableWidth: 1000
    property real maxAvailableHeight: 640

    signal dismissRequested()

    onVisibleChanged: {
        if (visible) {
            forceActiveFocus();
        }
    }

    Keys.onEscapePressed: root.dismissRequested()

    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        onClicked: root.dismissRequested()
    }

    Rectangle {
        anchors.centerIn: parent
        width: Math.min(1020, root.maxAvailableWidth)
        height: Math.min(660, root.maxAvailableHeight)
        radius: Theme.radiusDialog
        color: Theme.background2
        border.color: Theme.borderStrong
        border.width: 1

        // Bloqueia o clique-fora de atravessar o corpo do dialogo.
        MouseArea {
            anchors.fill: parent
        }

        Text {
            id: dialogTitle

            anchors.top: parent.top
            anchors.left: parent.left
            anchors.margins: Theme.spacingMedium
            text: qsTr("Configuration Actions")
            color: Theme.textPrimary
            font.pixelSize: 14
            font.bold: true
        }

        Text {
            id: scopeLabel

            anchors.verticalCenter: dialogTitle.verticalCenter
            anchors.left: dialogTitle.right
            anchors.leftMargin: Theme.spacingMedium
            text: root.controller.activeBuildSystems.length > 0
                  ? qsTr("Escopo: %1").arg(root.controller.activeBuildSystems.join(" + "))
                  : qsTr("Nenhum build system detectado")
            color: Theme.textMuted
            font.pixelSize: 11
        }

        Rectangle {
            id: closeChip

            anchors.top: parent.top
            anchors.right: parent.right
            anchors.margins: Theme.spacingMedium
            width: 22
            height: 22
            radius: Theme.radius
            color: closeArea.containsMouse ? Theme.surface2 : "transparent"
            border.color: Theme.borderSoft
            border.width: 1

            Text {
                anchors.centerIn: parent
                text: "✕"
                color: Theme.textSecondary
                font.pixelSize: 11
            }

            MouseArea {
                id: closeArea

                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: root.dismissRequested()
            }
        }

        // Filtro por escopo: so aparece quando ha mais de um build system,
        // porque num projeto so-CMake um seletor de uma opcao so e ruido.
        Row {
            id: scopeRow

            anchors.top: dialogTitle.bottom
            anchors.left: parent.left
            anchors.margins: Theme.spacingMedium
            anchors.topMargin: Theme.spacingSmall
            spacing: Theme.spacingXSmall
            visible: root.controller.activeBuildSystems.length > 1

            Repeater {
                model: ["", "cmake", "cargo"]

                Rectangle {
                    required property string modelData

                    readonly property bool selected: root.controller.scopeFilter === modelData

                    width: chipText.width + 2 * Theme.spacingSmall
                    height: 22
                    radius: Theme.radius
                    color: selected ? Theme.accentDim : Theme.surface1
                    border.color: selected ? Theme.accent : Theme.borderSoft
                    border.width: 1

                    Text {
                        id: chipText

                        anchors.centerIn: parent
                        text: parent.modelData === "" ? qsTr("Todas") : parent.modelData
                        color: parent.selected ? Theme.textPrimary : Theme.textSecondary
                        font.pixelSize: 11
                    }

                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: root.controller.scopeFilter = parent.modelData
                    }
                }
            }
        }

        Rectangle {
            id: searchBox

            anchors.top: scopeRow.visible ? scopeRow.bottom : dialogTitle.bottom
            anchors.left: parent.left
            anchors.margins: Theme.spacingMedium
            anchors.topMargin: Theme.spacingSmall
            width: 300
            height: 26
            radius: Theme.radius
            color: Theme.background0
            border.color: searchInput.activeFocus ? Theme.accent : Theme.borderSoft
            border.width: 1

            TextInput {
                id: searchInput

                anchors.fill: parent
                anchors.leftMargin: Theme.spacingSmall
                anchors.rightMargin: Theme.spacingSmall
                verticalAlignment: TextInput.AlignVCenter
                color: Theme.textPrimary
                selectionColor: Theme.accentDim
                selectedTextColor: Theme.textPrimary
                font.pixelSize: 12
                clip: true
                selectByMouse: true
                onTextChanged: root.controller.searchQuery = text
                Keys.onEscapePressed: root.dismissRequested()
            }

            Text {
                anchors.left: parent.left
                anchors.leftMargin: Theme.spacingSmall
                anchors.verticalCenter: parent.verticalCenter
                text: qsTr("Buscar ação...")
                color: Theme.textMuted
                font.pixelSize: 12
                visible: searchInput.text === ""
            }
        }

        ConfigActionList {
            id: actionList

            anchors.top: searchBox.bottom
            anchors.bottom: statusLine.top
            anchors.left: parent.left
            anchors.margins: Theme.spacingMedium
            anchors.topMargin: Theme.spacingSmall
            width: 340
            actionsModel: root.controller.actionsModel
            scopeFilter: root.controller.scopeFilter
            searchQuery: root.controller.searchQuery
            selectedId: root.controller.selectedId
            onActionSelected: function (id) {
                root.controller.select(id);
            }
        }

        ConfigActionPreview {
            anchors.top: actionList.top
            anchors.bottom: actionList.bottom
            anchors.left: actionList.right
            anchors.right: parent.right
            anchors.leftMargin: Theme.spacingMedium
            anchors.rightMargin: Theme.spacingMedium
            controller: root.controller
        }

        Text {
            id: statusLine

            anchors.bottom: parent.bottom
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.margins: Theme.spacingMedium
            text: root.controller.errorText !== "" ? root.controller.errorText
                                                   : root.controller.statusText
            color: root.controller.errorText !== "" ? Theme.errorSoft : Theme.textMuted
            font.pixelSize: 11
            elide: Text.ElideRight
        }
    }
}
