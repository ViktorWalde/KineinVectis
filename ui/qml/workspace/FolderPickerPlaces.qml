pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A coluna de LOCAIS do navegador de pastas (0.3.9, modelo da JetBrains):
// o Inicio num clique, e os projetos recentes quando se abre um.
// Os locais vem do core (`places`, 0.147.0) — so' os que existem; o rotulo e
// o icone sao daqui, pelo `id`.
Rectangle {
    id: root

    property var controller
    // Recentes so' no "Abrir projeto"; criar e escolher pasta nao os mostram.
    property var recentProjects: []

    color: Theme.background1
    radius: Theme.radiusLarge

    // So' o Inicio (0.152.0, decisao do autor em 2026-10-03: os projetos
    // ficam la', e ele ja' contem Documentos, Downloads e o resto); a raiz
    // so' quando nao ha' home. A raiz continua na barra de caminho (`/`).
    readonly property var placeNames: ({ home: qsTr("Início"), root: qsTr("Raiz do sistema") })
    readonly property var placeIcons: ({ home: "home", root: "drive" })

    // A pasta atual dentro deste lugar acende a linha dele (o mais fundo).
    function isCurrent(path) {
        return root.controller.currentPath === path;
    }

    Flickable {
        anchors.fill: parent
        anchors.margins: Theme.spacingSmall
        contentHeight: column.height
        clip: true
        boundsBehavior: Flickable.StopAtBounds

        Column {
            id: column

            width: parent.width
            spacing: 2

            Text {
                width: parent.width
                leftPadding: Theme.spacingSmall
                bottomPadding: Theme.spacingXSmall
                text: qsTr("LOCAIS")
                color: Theme.textMuted
                font.pixelSize: Theme.fontSizeCaption
                font.bold: true
                font.letterSpacing: 0.6
            }

            Repeater {
                model: root.controller.places

                delegate: FolderPickerPlaceRow {
                    required property var modelData

                    width: column.width
                    iconName: root.placeIcons[modelData.id] !== undefined
                              ? root.placeIcons[modelData.id] : "folder"
                    label: root.placeNames[modelData.id] !== undefined
                           ? root.placeNames[modelData.id] : modelData.path
                    hint: modelData.path
                    current: root.isCurrent(modelData.path)
                    onClicked: root.controller.browsePath(modelData.path)
                }
            }

            Item {
                visible: root.recentProjects.length > 0
                width: parent.width
                height: Theme.spacingMedium
            }

            Text {
                visible: root.recentProjects.length > 0
                width: parent.width
                leftPadding: Theme.spacingSmall
                bottomPadding: Theme.spacingXSmall
                text: qsTr("RECENTES")
                color: Theme.textMuted
                font.pixelSize: Theme.fontSizeCaption
                font.bold: true
                font.letterSpacing: 0.6
            }

            Repeater {
                model: root.recentProjects

                delegate: FolderPickerPlaceRow {
                    required property var modelData

                    width: column.width
                    iconName: "recent"
                    label: modelData.name
                    hint: modelData.root
                    current: root.isCurrent(modelData.root)
                    onClicked: root.controller.browsePath(modelData.root)
                }
            }
        }
    }
}
