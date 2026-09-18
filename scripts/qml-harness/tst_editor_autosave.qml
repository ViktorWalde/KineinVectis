import QtQuick
import "../../ui/qml/editor"

// O AUTOSAVE (Etapa 2, F3, decisao do autor em 2026-09-18) no
// EditorPersistenceController REAL: apos 2 s de pausa, ao perder o foco e
// no flush (troca de aba) — so' com buffer sujo, so' quando ligado.
//
// O que se prova: nada antes dos 2 s; o pedido chega depois; buffer limpo
// nao pede; desligado nao pede (nem no flush); o foco saindo com buffer
// sujo pede na hora; o flush pede na hora.
Item {
    id: root

    property int pedidos: 0
    property bool sujo: true
    property int falhas: 0

    QtObject {
        id: superficie

        property bool editorActiveFocus: true
    }

    QtObject {
        id: pontefalsa

        property var editorSurface: superficie
        function ready() { return true; }
        function text() { return "x"; }
    }

    ListModel { id: abas }

    QtObject {
        id: documentosFalsos

        function currentFilePath() { return "/tmp/p/a.rs"; }
        function currentIsModified() { return root.sujo; }
    }

    EditorPersistenceController {
        id: persistence

        workspaceRoot: "/tmp/p"
        surfaceBridge: pontefalsa
        documentController: documentosFalsos
        filesModel: abas
        onAutoSaveRequested: root.pedidos += 1
    }

    Component.onCompleted: {
        // Sincrono: flush com sujo pede; limpo... o flush nao filtra (quem
        // filtra e' o EditorController); desligado nao pede.
        persistence.flushAutoSave();
        if (root.pedidos !== 1) root.falhas += 1;
        persistence.autoSaveEnabled = false;
        persistence.flushAutoSave();
        persistence.scheduleDraftSave();
        if (root.pedidos !== 1) root.falhas += 2;
        persistence.autoSaveEnabled = true;

        // Perder o foco com buffer sujo pede na hora; com limpo, nao.
        superficie.editorActiveFocus = false;
        if (root.pedidos !== 2) root.falhas += 4;
        superficie.editorActiveFocus = true;
        root.sujo = false;
        superficie.editorActiveFocus = false;
        if (root.pedidos !== 2) root.falhas += 8;
        superficie.editorActiveFocus = true;

        // A pausa: agenda com buffer sujo; nada aos 1000 ms, um pedido aos 2600.
        root.sujo = true;
        persistence.scheduleDraftSave();
        cedo.start();
        tarde.start();
    }

    Timer {
        id: cedo
        interval: 1000
        onTriggered: if (root.pedidos !== 2) root.falhas += 16;
    }

    Timer {
        id: tarde
        interval: 2600
        onTriggered: {
            if (root.pedidos !== 3) root.falhas += 32;
            // Buffer limpo no disparo: nada.
            root.sujo = false;
            persistence.scheduleDraftSave();
            fim.start();
        }
    }

    Timer {
        id: fim
        interval: 2600
        onTriggered: {
            if (root.pedidos !== 3) root.falhas += 64;
            if (root.falhas !== 0) console.error("FALHAS bitmask=" + root.falhas);
            Qt.exit(root.falhas === 0 ? 0 : 1);
        }
    }
}
