import QtQuick

Item {
    id: root

    property int handleThickness: 6
    property bool resizeEnabled: true

    signal resizeRequested(int edges)

    enabled: resizeEnabled
    visible: resizeEnabled

    MouseArea {
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.leftMargin: root.handleThickness
        anchors.rightMargin: root.handleThickness
        height: root.handleThickness
        acceptedButtons: Qt.LeftButton
        cursorShape: Qt.SizeVerCursor
        onPressed: root.resizeRequested(Qt.TopEdge)
    }

    MouseArea {
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.leftMargin: root.handleThickness
        anchors.rightMargin: root.handleThickness
        height: root.handleThickness
        acceptedButtons: Qt.LeftButton
        cursorShape: Qt.SizeVerCursor
        onPressed: root.resizeRequested(Qt.BottomEdge)
    }

    MouseArea {
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.topMargin: root.handleThickness
        anchors.bottomMargin: root.handleThickness
        width: root.handleThickness
        acceptedButtons: Qt.LeftButton
        cursorShape: Qt.SizeHorCursor
        onPressed: root.resizeRequested(Qt.LeftEdge)
    }

    MouseArea {
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        anchors.right: parent.right
        anchors.topMargin: root.handleThickness
        anchors.bottomMargin: root.handleThickness
        width: root.handleThickness
        acceptedButtons: Qt.LeftButton
        cursorShape: Qt.SizeHorCursor
        onPressed: root.resizeRequested(Qt.RightEdge)
    }

    MouseArea {
        anchors.top: parent.top
        anchors.left: parent.left
        width: root.handleThickness
        height: root.handleThickness
        acceptedButtons: Qt.LeftButton
        cursorShape: Qt.SizeFDiagCursor
        onPressed: root.resizeRequested(Qt.LeftEdge | Qt.TopEdge)
    }

    MouseArea {
        anchors.top: parent.top
        anchors.right: parent.right
        width: root.handleThickness
        height: root.handleThickness
        acceptedButtons: Qt.LeftButton
        cursorShape: Qt.SizeBDiagCursor
        onPressed: root.resizeRequested(Qt.RightEdge | Qt.TopEdge)
    }

    MouseArea {
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        width: root.handleThickness
        height: root.handleThickness
        acceptedButtons: Qt.LeftButton
        cursorShape: Qt.SizeBDiagCursor
        onPressed: root.resizeRequested(Qt.LeftEdge | Qt.BottomEdge)
    }

    MouseArea {
        anchors.bottom: parent.bottom
        anchors.right: parent.right
        width: root.handleThickness
        height: root.handleThickness
        acceptedButtons: Qt.LeftButton
        cursorShape: Qt.SizeFDiagCursor
        onPressed: root.resizeRequested(Qt.RightEdge | Qt.BottomEdge)
    }
}
