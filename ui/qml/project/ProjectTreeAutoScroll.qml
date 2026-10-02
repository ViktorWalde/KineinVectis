import QtQuick

// Rolagem automatica da arvore durante um arrasto: perto da borda de cima ou
// de baixo, a lista anda sozinha — mais rapido quanto mais perto (0.3.9: 4
// px por passo a 32 px da borda, ate' 20 px colado nela). `edgeY` e' a
// altura do cursor na lista; -1 quando nao ha' arrasto.
Timer {
    id: root

    property Flickable view: null
    property real edgeY: -1

    interval: 35
    repeat: true
    running: root.edgeY >= 0 && root.view !== null
    onTriggered: {
        const fromTop = root.edgeY;
        const fromBottom = root.view.height - root.edgeY;
        if (fromTop < 32)
            root.view.contentY = Math.max(0, root.view.contentY - (4 + (32 - fromTop) / 2));
        else if (fromBottom < 32)
            root.view.contentY = Math.min(
                Math.max(0, root.view.contentHeight - root.view.height),
                root.view.contentY + 4 + (32 - fromBottom) / 2);
    }
}
