import QtQuick
import QtQuick.Effects
import "IconRegistry.js" as IconRegistry

Item {
    id: root

    property string iconId: ""
    property color color: "#AAA39A"
    property int iconSize: 20
    property bool colorize: true

    implicitWidth: iconSize
    implicitHeight: iconSize

    Image {
        id: sourceImage
        anchors.fill: parent
        source: IconRegistry.source(root.iconId)
        sourceSize.width: root.iconSize
        sourceSize.height: root.iconSize
        fillMode: Image.PreserveAspectFit
        smooth: true
        mipmap: true
        cache: true
        visible: !root.colorize
    }

    MultiEffect {
        anchors.fill: sourceImage
        source: sourceImage
        visible: root.colorize && sourceImage.status === Image.Ready
        colorization: 1.0
        colorizationColor: root.color
        autoPaddingEnabled: false
    }
}
