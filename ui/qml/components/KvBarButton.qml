import QtQuick
import KineinVectis

// Botao pequeno de barra: OU um icone (22x22) OU um rotulo de texto, que
// entao manda na largura.
//
// POR QUE ESTE ARQUIVO EXISTE (2026-09-03). Era um `component ActionButton`
// declarado DENTRO do EditorFindBar — 50 linhas de desenho de botao presas num
// arquivo cuja responsabilidade e' a barra de busca. Componente inline nao e'
// reusavel fora do arquivo que o declara, entao a proxima barra copiaria.
//
// DESABILITADO E' VISUAL E DE COMPORTAMENTO: opacidade cai, o cursor volta a
// ser seta e o clique nao emite. Faltando qualquer um dos tres, o botao mente.
Rectangle {
    id: root

    property string labelText: ""
    property string iconName: ""
    property bool enabledAction: true

    signal activated()

    width: iconName !== "" ? 22
                           : Math.max(22, label.implicitWidth
                                          + 2 * Theme.spacingSmall)
    height: 22
    radius: Theme.radiusXSmall
    color: mouse.containsMouse && root.enabledAction
           ? Theme.surfaceSelected : "transparent"
    border.color: Theme.borderSoft
    border.width: 1
    opacity: root.enabledAction ? 1.0 : 0.4

    Text {
        id: label

        anchors.centerIn: parent
        visible: root.iconName === ""
        text: root.labelText
        color: Theme.textSecondary
        font.pixelSize: 11
    }

    KvIcon {
        anchors.centerIn: parent
        visible: root.iconName !== ""
        name: root.iconName
        size: 14
        disabled: !root.enabledAction
    }

    MouseArea {
        id: mouse

        anchors.fill: parent
        hoverEnabled: true
        cursorShape: root.enabledAction ? Qt.PointingHandCursor
                                        : Qt.ArrowCursor
        onClicked: {
            if (root.enabledAction) {
                root.activated();
            }
        }
    }
}
