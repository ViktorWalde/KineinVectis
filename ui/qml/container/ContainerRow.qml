pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// UMA LINHA da janela de Containers (2026-10-03, polimento pedido pelo autor:
// "melhorar a UI dos botoes de interacao e deixar mais interativo
// visualmente"). Tres formas, pelo `kind` do ContainerRows:
//
//   secao      ▾ EM EXECUCAO  (2)
//   container  ●  web                          [:8080 ↗]  ■  ▤  >_
//                 nginx:1.27 · Up 2 hours
//   imagem     ⬡  docker.io/library/nginx:1.27   em uso    187 MB
//
// As acoes ficam SEMPRE a vista, discretas, e acendem com o mouse na linha;
// a cor diz a intencao (verde inicia, vermelho para/remove). Enquanto o motor
// trabalha, o ponto pulsa e a linha diz "parando…". Remover pede um segundo
// clique ("confirmar") — o primeiro so' arma, por alguns segundos.
Rectangle {
    id: root

    required property var modelData
    property var controller: null
    property var rowsModel: null

    readonly property bool section: root.modelData.kind === "section"
    readonly property bool image: root.modelData.kind === "image"
    readonly property bool isContainer: root.rowsModel !== null && root.rowsModel.isContainer(root.modelData)
    readonly property bool running: root.isContainer && ContainerStates.isActive(root.modelData.state)
    readonly property bool busy: root.isContainer && root.controller !== null && root.controller.isPending(root.modelData.target)
    readonly property bool selected: root.isContainer && root.controller !== null
                                     && root.controller.selectedId === root.modelData.id

    height: root.section ? 26 : (root.isContainer ? 42 : 28)
    radius: Theme.radius
    color: root.selected ? Theme.surfaceSelected : (hover.hovered && !root.section ? Theme.surface2 : "transparent")

    HoverHandler {
        id: hover
    }

    MouseArea {
        anchors.fill: parent
        cursorShape: root.section || root.isContainer ? Qt.PointingHandCursor : Qt.ArrowCursor
        onClicked: {
            if (root.section) root.rowsModel.toggle(root.modelData.key);
            else if (root.isContainer) root.controller.select(root.selected ? "" : root.modelData.id);
        }
    }

    // ---- secao ---------------------------------------------------------------

    Row {
        anchors.left: parent.left
        anchors.leftMargin: Theme.spacingXSmall
        anchors.verticalCenter: parent.verticalCenter
        visible: root.section
        spacing: Theme.spacingSmall

        Text {
            anchors.verticalCenter: parent.verticalCenter
            text: root.section && root.modelData.expanded ? "▾" : "▸"
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeSmall
        }

        Text {
            anchors.verticalCenter: parent.verticalCenter
            text: root.section ? root.modelData.title : ""
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeMicro
            font.weight: Font.DemiBold
            font.letterSpacing: 0.8
        }

        Rectangle {
            anchors.verticalCenter: parent.verticalCenter
            width: countText.implicitWidth + 2 * Theme.spacingSmall
            height: 16
            radius: height / 2
            color: Theme.surface2

            Text {
                id: countText

                anchors.centerIn: parent
                text: root.section ? root.modelData.count : ""
                color: Theme.textSecondary
                font.pixelSize: Theme.fontSizeMicro
            }
        }
    }

    // ---- container -------------------------------------------------------------

    Rectangle {
        id: dot

        anchors.left: parent.left
        anchors.leftMargin: Theme.spacingSmall + 4
        anchors.verticalCenter: parent.verticalCenter
        visible: root.isContainer
        width: 9
        height: 9
        radius: width / 2
        // Parado: so' o contorno; rodando/pausado: a cor do estado.
        color: root.running ? ContainerStates.color(root.modelData.state) : "transparent"
        border.width: root.running ? 0 : 1
        border.color: Theme.textMuted

        // Trabalhando: o ponto pulsa ate' o motor responder.
        SequentialAnimation on opacity {
            running: root.busy
            loops: Animation.Infinite
            alwaysRunToEnd: true
            NumberAnimation { to: 0.25; duration: Theme.motionPulse }
            NumberAnimation { to: 1; duration: Theme.motionPulse }
        }
    }

    Column {
        anchors.left: dot.right
        anchors.leftMargin: Theme.spacingSmall + 2
        anchors.right: trailing.left
        anchors.rightMargin: Theme.spacingSmall
        anchors.verticalCenter: parent.verticalCenter
        visible: root.isContainer
        spacing: 1

        Text {
            width: parent.width
            text: root.isContainer ? root.modelData.name : ""
            color: Theme.textPrimary
            font.pixelSize: Theme.fontSizeSmall
            font.weight: Font.DemiBold
            elide: Text.ElideRight
        }

        Text {
            width: parent.width
            text: !root.isContainer ? ""
                  : (root.busy ? ContainerStates.pendingLabel(root.controller.pending[root.modelData.target])
                     : root.modelData.imageShort + (root.modelData.status !== "" ? "  ·  " + root.modelData.status : ""))
            color: root.busy ? Theme.accent : Theme.textMuted
            font.family: root.busy ? Theme.uiFont : Theme.monoFont
            font.pixelSize: Theme.fontSizeCaption
            elide: Text.ElideRight
        }
    }

    ContainerRowActions {
        id: trailing

        anchors.right: parent.right
        anchors.rightMargin: Theme.spacingXSmall
        anchors.verticalCenter: parent.verticalCenter
        visible: root.isContainer
        row: root.isContainer ? root.modelData : null
        controller: root.controller
        running: root.running
        busy: root.busy
        lit: hover.hovered || root.selected
    }

    // ---- imagem --------------------------------------------------------------

    KvIcon {
        id: imageIcon

        anchors.left: parent.left
        anchors.leftMargin: Theme.spacingSmall + 1
        anchors.verticalCenter: parent.verticalCenter
        visible: root.image
        size: 14
        name: "container"
    }

    Text {
        anchors.left: imageIcon.right
        anchors.leftMargin: Theme.spacingSmall
        anchors.right: imageTail.left
        anchors.rightMargin: Theme.spacingSmall
        anchors.verticalCenter: parent.verticalCenter
        visible: root.image
        // Corta no MEIO: a tag (`:16`) fica a vista.
        text: root.image ? root.modelData.name : ""
        color: Theme.textSecondary
        font.family: Theme.monoFont
        font.pixelSize: Theme.fontSizeSmall
        elide: Text.ElideMiddle
    }

    Row {
        id: imageTail

        anchors.right: parent.right
        anchors.rightMargin: Theme.spacingSmall
        anchors.verticalCenter: parent.verticalCenter
        visible: root.image
        spacing: Theme.spacingSmall

        Rectangle {
            anchors.verticalCenter: parent.verticalCenter
            visible: root.image && root.modelData.inUse === true
            width: inUseText.implicitWidth + 2 * Theme.spacingSmall
            height: 16
            radius: height / 2
            color: "transparent"
            border.width: 1
            border.color: Theme.successSoft

            Text {
                id: inUseText

                anchors.centerIn: parent
                text: qsTr("em uso")
                color: Theme.successSoft
                font.pixelSize: Theme.fontSizeMicro
            }
        }

        Text {
            anchors.verticalCenter: parent.verticalCenter
            text: root.image && root.controller ? root.controller.formatSize(root.modelData.size) : ""
            color: Theme.textMuted
            font.family: Theme.monoFont
            font.pixelSize: Theme.fontSizeCaption
        }
    }
}
