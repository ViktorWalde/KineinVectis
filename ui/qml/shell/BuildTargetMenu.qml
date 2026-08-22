pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Dropdown do seletor de alvo de build da Main Toolbar (L3 fatia 2).
//
// Vive nos overlays pelo mesmo motivo do RunConfigMenu: a barra tem 46px e os
// painéis desenhados depois dela cobririam o menu.
//
// O `kind` de cada alvo aparece ao lado do nome porque `executable` e
// `staticLibrary` se compilam igual mas se USAM diferente — e o spec pede que
// alvos de biblioteca não sejam oferecidos como Run. Mostrar o tipo aqui é o
// que deixa essa distinção visível antes de ela virar regra.
Item {
    id: root

    property real menuX: 0
    property real menuY: 0
    property var targetsModel
    property string activeTarget: ""

    signal dismissRequested()
    signal targetChosen(string name)

    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        onClicked: root.dismissRequested()
    }

    Rectangle {
        x: Math.min(root.menuX, root.width - width - Theme.spacingMedium)
        y: root.menuY
        width: 260
        height: menuColumn.height + 2 * Theme.spacingSmall
        radius: Theme.radius
        color: Theme.background2
        border.color: Theme.borderSoft
        border.width: 1

        Column {
            id: menuColumn

            x: Theme.spacingSmall
            y: Theme.spacingSmall
            width: parent.width - 2 * Theme.spacingSmall
            spacing: 1

            // "Todos os alvos" primeiro, e é o padrão: quem nunca escolheu
            // nada não pode ter o build silenciosamente reduzido a um alvo.
            BuildTargetMenuRow {
                width: menuColumn.width
                label: qsTr("Todos os alvos")
                kind: ""
                selected: root.activeTarget === ""
                onChosen: root.targetChosen("")
            }

            Rectangle {
                width: menuColumn.width
                height: 1
                color: Theme.borderSoft
            }

            Repeater {
                model: root.targetsModel

                delegate: BuildTargetMenuRow {
                    required property string name
                    required property string kind

                    width: menuColumn.width
                    label: name
                    kind: kind
                    selected: root.activeTarget === name
                    onChosen: root.targetChosen(name)
                }
            }
        }
    }
}
