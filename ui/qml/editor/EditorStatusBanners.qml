import QtQuick
import KineinVectis

// Estado da leitura acima do texto: conflito do workspace e prévia externa.
Item {
    id: root

    property bool externalConflict: false
    property bool externalDeleted: false
    property string externalMessage: ""
    property string watchError: ""
    property bool readOnlyExternal: false
    property string externalPreviewError: ""
    property string currentFilePath: ""

    signal reloadRequested()
    signal keepLocalRequested()
    signal watchErrorDismissRequested()

    height: readOnlyBanner.y + readOnlyBanner.height

    EditorExternalChangeBanner {
        id: externalBanner

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.topMargin: visible ? Theme.spacingXSmall : 0
        active: root.externalConflict || root.watchError !== ""
        deleted: root.externalDeleted
        watcherFailure: !root.externalConflict && root.watchError !== ""
        message: root.externalConflict ? root.externalMessage
                 : qsTr("O monitor de arquivos falhou: %1. Saves continuam protegidos contra conflito.")
                   .arg(root.watchError)
        onReloadRequested: root.reloadRequested()
        onKeepLocalRequested: root.keepLocalRequested()
        onDismissRequested: root.watchErrorDismissRequested()
    }

    Rectangle {
        id: readOnlyBanner

        anchors.top: externalBanner.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.topMargin: visible ? Theme.spacingXSmall : 0
        height: visible ? 28 : 0
        radius: Theme.radius
        color: Theme.surface2
        visible: root.readOnlyExternal || root.externalPreviewError !== ""

        Text {
            anchors.fill: parent
            anchors.leftMargin: Theme.spacingSmall
            anchors.rightMargin: Theme.spacingSmall
            verticalAlignment: Text.AlignVCenter
            elide: Text.ElideRight
            color: Theme.textPrimary
            font.pixelSize: Theme.fontSizeStatus
            text: root.externalPreviewError !== "" ? root.externalPreviewError
                  : qsTr("Somente leitura · arquivo externo fora do projeto: %1")
                    .arg(root.currentFilePath)
        }
    }
}
