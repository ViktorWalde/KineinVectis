import QtQuick
import KineinVectis

// As listas cujo cabecalho de secao ou `header` vem de um arquivo de PARTES
// (G0.3, roadmap 53 §0.2).
//
// Por que existe: no Qt 6.4 do AppImage, um `section.delegate` ou `header`
// declarado num arquivo com `pragma ComponentBehavior: Bound` simplesmente nao
// nasce ("Component is not ready"), e o Qt 6.10 local cria normalmente. O
// defeito chegou ao usuario em 2026-10-01 porque nenhum harness INSTANCIAVA
// estas listas. Aqui elas sao montadas com dados e o teste exige que o
// cabecalho de cada secao e o `header` existam de fato. Rodado pelo
// verificar-qml-logica-qt64.sh, e' o 6.4 que responde.
Item {
    id: root

    width: 320
    height: 240

    ListModel {
        id: changes

        ListElement { path: "src/a.cpp"; absPath: "/tmp/p/src/a.cpp"; kind: "M"; staged: false; folder: "src" }
        ListElement { path: "docs/b.md"; absPath: "/tmp/p/docs/b.md"; kind: "A"; staged: true; folder: "docs" }
    }

    GitChangesList {
        id: gitList

        width: root.width
        height: 100
        changesModel: changes
        repo: true
    }

    QtObject {
        id: fakeSymbols

        readonly property var folderResults: [
            { name: "parse", kind: "fn", path: "src/a.rs", line: 3, source: "index" }
        ]
        readonly property var results: [
            { name: "Parser", kind: "struct", path: "src/b.rs", line: 9, source: "index" }
        ]
        readonly property bool searching: true
        readonly property bool active: true
        readonly property bool showSource: false
        readonly property string indexState: "ready"

        function open(symbol) {}
    }

    SymbolResultsList {
        id: symbolList

        y: 110
        width: root.width
        height: 120
        symbols: fakeSymbols
    }

    function sectionCount(list) {
        let count = 0;
        const children = list.contentItem.children;
        for (let i = 0; i < children.length; i++) {
            if (children[i].section !== undefined && children[i].visible) count += 1;
        }
        return count;
    }

    Timer {
        interval: 300
        running: true
        onTriggered: {
            let failures = 0;

            if (root.sectionCount(gitList) !== 2) failures += 1;
            if (root.sectionCount(symbolList) !== 2) failures += 2;
            if (symbolList.headerItem === null) failures += 4;
            else if (symbolList.headerItem.height <= 0) failures += 8;
            if (gitList.count !== 2 || symbolList.count !== 2) failures += 16;

            if (failures !== 0) console.error("FALHAS bitmask=" + failures);
            Qt.exit(failures === 0 ? 0 : 1);
        }
    }
}
