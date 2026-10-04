import QtQuick
import KineinVectis

// Uma ACAO do alvo como linha (2026-10-04, pedido do autor: a fileira de
// botoes "Enviar (deploy)", "gdbserver → kit"… apagados e sem motivo estava no
// estilo antigo). Icone, o que faz, e — desligada — POR QUE nao da':
//
//   [⬆]  Enviar para o alvo                      ›
//        build/app → /opt/app
//   [🐞]  Depurar C/C++ (gdbserver)               ›
//        falta gdbserver no alvo                     (aviso em ambar)
Rectangle {
    id: root

    property string iconName: "run"
    property string title: ""
    property string detail: ""
    // Desligada: a linha apaga e o `detail` deve dizer o motivo.
    property bool available: true
    // Funciona, mas com um porem (o alvo nao tem a ferramenta): ambar.
    property bool caution: false
    property bool busy: false

    signal triggered()

    implicitHeight: Math.max(46, texts.implicitHeight + 2 * Theme.spacingSmall)
    radius: Theme.radius
    color: !root.available ? "transparent" : (area.pressed ? Theme.surfaceSelected : (area.containsMouse ? Theme.surface2 : "transparent"))
    opacity: root.available ? 1 : 0.6
    scale: area.pressed ? 0.98 : 1
    Accessible.role: Accessible.Button
    Accessible.name: root.title

    Behavior on color {
        ColorAnimation { duration: Theme.motionFast }
    }

    Behavior on scale {
        NumberAnimation { duration: Theme.motionFast }
    }

    Rectangle {
        id: badge

        anchors.left: parent.left
        anchors.leftMargin: Theme.spacingSmall
        anchors.verticalCenter: parent.verticalCenter
        width: 30
        height: 30
        radius: Theme.radius
        color: Theme.background0
        border.width: 1
        border.color: area.containsMouse && root.available ? Theme.accentDim : Theme.borderSoft

        Behavior on border.color {
            ColorAnimation { duration: Theme.motionFast }
        }

        KvIcon {
            anchors.centerIn: parent
            name: root.iconName
            size: 16
            active: area.containsMouse && root.available
            disabled: !root.available
        }
    }

    Column {
        id: texts

        anchors.left: badge.right
        anchors.leftMargin: Theme.spacingMedium
        anchors.right: chevron.left
        anchors.rightMargin: Theme.spacingSmall
        anchors.verticalCenter: parent.verticalCenter
        spacing: 1

        Text {
            width: parent.width
            text: root.busy ? root.title + "…" : root.title
            color: root.available ? Theme.textPrimary : Theme.textSecondary
            font.pixelSize: Theme.fontSizeBody
            elide: Text.ElideRight
        }

        Text {
            width: parent.width
            visible: root.detail !== ""
            text: root.detail
            color: root.caution ? Theme.warningSoft : Theme.textMuted
            font.pixelSize: Theme.fontSizeCaption
            // Quebra em ate' duas linhas: cortado no meio, o motivo nao se le'.
            wrapMode: Text.WordWrap
            maximumLineCount: 2
            elide: Text.ElideRight
        }
    }

    KvIcon {
        id: chevron

        anchors.right: parent.right
        anchors.rightMargin: Theme.spacingSmall
        anchors.verticalCenter: parent.verticalCenter
        name: "chevron-right"
        size: 14
        opacity: area.containsMouse && root.available ? 1 : 0.4
        active: area.containsMouse && root.available

        Behavior on opacity {
            NumberAnimation { duration: Theme.motionFast }
        }
    }

    MouseArea {
        id: area

        anchors.fill: parent
        enabled: root.available && !root.busy
        hoverEnabled: true
        cursorShape: enabled ? Qt.PointingHandCursor : Qt.ArrowCursor
        onClicked: root.triggered()
    }
}
