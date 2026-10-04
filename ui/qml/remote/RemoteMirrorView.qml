pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O workspace ESPELHADO (P6 fatia 2, 2026-09-18): abrir uma pasta do alvo
// como espelho local, e — quando o workspace aberto E' um espelho — de quem
// ele e' e os dois gestos de sincronia. Burro: recebe, pede por sinal.
Item {
    id: root

    property string openPath: ""
    property var mirror: null
    property bool isMirror: false
    property bool canOpen: false
    property bool syncing: false
    property string syncMessage: ""
    property var browseState: null
    readonly property var browser: browseState || ({ browseVisible: false, browseLoading: false, browsePath: "", browseParent: "", browseEntries: [], browseError: "" })

    signal openPathEdited(string text)
    signal openFolderRequested()
    signal browseAction(string kind, string path)
    signal syncRequested(string direction)

    implicitHeight: coluna.implicitHeight

    Column {
        id: coluna

        anchors.left: parent.left
        anchors.right: parent.right
        spacing: Theme.spacingXSmall

        // Campo e botoes EMPILHADOS (2026-10-04): na janela acoplada, lado a
        // lado, o campo sobrava com uns poucos pixels.
        DataSourceField {
            width: parent.width
            label: qsTr("Pasta no alvo (abre como espelho local)")
            placeholder: "/home/pi/projeto"
            value: root.openPath
            onEdited: text => root.openPathEdited(text)
            onAccepted: root.openFolderRequested()
        }

        Row {
            anchors.right: parent.right
            spacing: Theme.spacingSmall

            KvButton {
                text: qsTr("Escolher…")
                iconName: "folder"
                compact: true
                enabled: root.canOpen
                onClicked: root.browseAction("start", "")
            }

            KvButton {
                text: qsTr("Abrir espelho")
                compact: true
                primary: true
                enabled: root.canOpen && root.openPath.trim() !== ""
                onClicked: root.openFolderRequested()
            }
        }

        Text {
            width: parent.width
            visible: !root.canOpen && !root.syncing
            wrapMode: Text.WordWrap
            text: qsTr("Salve um alvo em Configurar para abrir uma pasta dele.")
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeCaption
        }

        Rectangle {
            width: parent.width
            visible: root.browser.browseVisible
            height: browseColumn.implicitHeight + 2 * Theme.spacingSmall
            radius: Theme.radius
            color: Theme.surface2

            Column {
                id: browseColumn

                anchors.left: parent.left
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                anchors.margins: Theme.spacingSmall
                spacing: Theme.spacingXSmall

                Text {
                    width: parent.width
                    text: root.browser.browseLoading ? qsTr("Listando pastas do alvo…")
                          : (root.browser.browsePath || qsTr("Home do alvo"))
                    color: Theme.textPrimary
                    font.family: Theme.monoFont
                    font.pixelSize: Theme.fontSizeSmall
                    elide: Text.ElideMiddle
                }

                Row {
                    spacing: Theme.spacingSmall

                    KvButton {
                        text: qsTr("Subir")
                        iconName: "chevron-up"
                        compact: true
                        enabled: !root.browser.browseLoading && root.browser.browseParent !== ""
                        onClicked: root.browseAction("navigate", root.browser.browseParent)
                    }

                    KvButton {
                        text: qsTr("Abrir esta pasta")
                        compact: true
                        primary: true
                        enabled: !root.browser.browseLoading && root.browser.browseError === ""
                                 && root.browser.browsePath !== ""
                        onClicked: root.browseAction("open", "")
                    }
                }

                ListView {
                    width: parent.width
                    height: Math.min(root.browser.browseEntries.length * 30, 180)
                    clip: true
                    model: root.browser.browseEntries
                    spacing: 2

                    // Uma pasta por linha, como no seletor de pastas.
                    delegate: Rectangle {
                        id: folderRow

                        required property var modelData

                        width: ListView.view.width
                        height: 28
                        radius: Theme.radius
                        color: folderArea.containsMouse ? Theme.surfaceSelected : "transparent"

                        Behavior on color {
                            ColorAnimation { duration: Theme.motionFast }
                        }

                        KvIcon {
                            id: folderIcon

                            anchors.left: parent.left
                            anchors.leftMargin: Theme.spacingSmall
                            anchors.verticalCenter: parent.verticalCenter
                            name: "folder"
                            size: 14
                            active: folderArea.containsMouse
                        }

                        Text {
                            anchors.left: folderIcon.right
                            anchors.leftMargin: Theme.spacingSmall
                            anchors.right: parent.right
                            anchors.verticalCenter: parent.verticalCenter
                            text: folderRow.modelData.name
                            color: Theme.textPrimary
                            font.pixelSize: Theme.fontSizeSmall
                            elide: Text.ElideRight
                        }

                        MouseArea {
                            id: folderArea

                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: root.browseAction("navigate", folderRow.modelData.path)
                        }
                    }
                }

                Text {
                    width: parent.width
                    visible: !root.browser.browseLoading && root.browser.browseEntries.length === 0
                             && root.browser.browseError === ""
                    text: qsTr("Nenhuma subpasta disponível.")
                    color: Theme.textMuted
                    font.pixelSize: Theme.fontSizeCaption
                }

                Text {
                    width: parent.width
                    visible: root.browser.browseError !== ""
                    wrapMode: Text.WordWrap
                    text: root.browser.browseError
                    color: Theme.errorSoft
                    font.pixelSize: Theme.fontSizeCaption
                }

            }
        }

        Rectangle {
            width: parent.width
            visible: root.isMirror
            height: espelho.implicitHeight + 2 * Theme.spacingSmall
            radius: Theme.radius
            color: Theme.surface2

            Column {
                id: espelho

                anchors.left: parent.left
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                anchors.margins: Theme.spacingSmall
                spacing: Theme.spacingXSmall

                Text {
                    width: parent.width
                    wrapMode: Text.WrapAnywhere
                    // `isMirror` e `mirror` chegam em sinais separados: um
                    // instante com o primeiro e sem o segundo (TypeError no log).
                    text: root.isMirror && root.mirror !== null
                          ? qsTr("Este projeto é um espelho de %1:%2 — salvar empurra o arquivo; "
                                 + "o que mudar no alvo só aparece ao Puxar.").arg(root.mirror.name).arg(root.mirror.path)
                          : ""
                    color: Theme.textSecondary
                    font.pixelSize: Theme.fontSizeCaption
                }

                Row {
                    spacing: Theme.spacingSmall

                    KvButton {
                        text: qsTr("Puxar do alvo")
                        iconName: "pull"
                        compact: true
                        enabled: !root.syncing
                        onClicked: root.syncRequested("pull")
                    }

                    KvButton {
                        text: qsTr("Empurrar tudo")
                        iconName: "push"
                        compact: true
                        enabled: !root.syncing
                        onClicked: root.syncRequested("push")
                    }
                }
            }
        }

        Text {
            width: parent.width
            visible: root.syncing || root.syncMessage !== ""
            wrapMode: Text.WordWrap
            text: root.syncing ? qsTr("Sincronizando (rsync)...") : root.syncMessage
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeCaption
        }
    }
}
