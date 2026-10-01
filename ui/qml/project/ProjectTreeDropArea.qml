import QtQuick
import KineinVectis

// Destino de arrasto da árvore: linha ou espaço vazio, com a mesma política.
DropArea {
    id: root

    property string destination: ""
    property bool validDrag: false
    property bool copy: false
    property bool externalDrag: false
    property var urlDecoder: null
    signal filesDropped(var paths, string destination, bool copy, bool external)
    signal dragPosition(real y)
    signal dragEnded()

    onEntered: function(drag) {
        const internal = ProjectDragRules.internalPaths(drag).length > 0;
        externalDrag = !internal && drag.hasUrls;
        validDrag = externalDrag
                    ? urlDecoder !== null && urlDecoder.localFilePathsFromUrls(drag.urls).length > 0
                    : internal;
        copy = externalDrag || drag.proposedAction === Qt.CopyAction;
        drag.accepted = validDrag;
    }
    onPositionChanged: function(drag) {
        copy = externalDrag || drag.proposedAction === Qt.CopyAction;
        dragPosition(drag.y);
    }
    onExited: {
        validDrag = false;
        externalDrag = false;
        dragEnded();
    }
    onDropped: function(drop) { acceptDrop(drop); }

    function acceptDrop(drop) {
        validDrag = false;
        dragEnded();
        const paths = ProjectDragRules.internalPaths(drop);
        const external = paths.length === 0;
        if (external && drop.hasUrls) {
            const localPaths = urlDecoder === null ? []
                               : urlDecoder.localFilePathsFromUrls(drop.urls);
            if (localPaths.length === 0) {
                drop.accepted = false;
                return;
            }
            drop.accept(Qt.CopyAction);
            // QStringList da ponte Qt é sequência QML, não Array JS.
            filesDropped(Array.from(localPaths), destination, true, true);
            return;
        }
        if (external) {
            drop.accepted = false;
            return;
        }
        const copy = drop.proposedAction === Qt.CopyAction;
        drop.accept(copy ? Qt.CopyAction : Qt.MoveAction);
        filesDropped(paths, destination, copy, false);
    }
}
