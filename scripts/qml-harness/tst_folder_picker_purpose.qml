import QtQuick
import "../../ui/qml/workspace"

// O FolderPicker com PROPOSITO (2026-09-17): o mesmo navegador de pastas da
// Start Screen serve para escolher o SDK/sysroot do kit. "workspace" abre o
// projeto (openRequested, como sempre); outro proposito devolve o caminho
// por folderPicked e nao abre nada. Sem dialogo novo: um picker, dois usos.
Item {
    id: root

    property var abertos: []
    property var escolhidos: []
    property var navegados: []

    FolderPickerController {
        id: picker

        onOpenRequested: function(path) { root.abertos.push(path); }
        onFolderPicked: function(purpose, path) { root.escolhidos.push(purpose + "|" + path); }
        onBrowseRequested: function(path) { root.navegados.push(path); }
    }

    Component.onCompleted: {
        let failures = 0;

        // O uso de sempre: open() e' "workspace", e Abrir abre.
        picker.open("/home/x");
        if (picker.purpose !== "workspace" || root.navegados[0] !== "/home/x") failures += 1;
        picker.selectedPath = "/home/x/proj";
        picker.openSelected();
        if (root.abertos.join(",") !== "/home/x/proj" || root.escolhidos.length !== 0) failures += 2;

        // Escolher para o kit: Escolher devolve o caminho e NAO abre.
        picker.openFor("kitPath", "/opt");
        if (picker.purpose !== "kitPath" || root.navegados[1] !== "/opt") failures += 4;
        picker.selectedPath = "/opt/zephyr-sdk-1.0.1";
        picker.openSelected();
        if (root.escolhidos.join(",") !== "kitPath|/opt/zephyr-sdk-1.0.1") failures += 8;
        if (root.abertos.length !== 1) failures += 16;

        // Sem selecao, a pasta atual e' a escolhida; sem nada, nada sai.
        picker.selectedPath = "";
        picker.currentPath = "/opt";
        picker.openSelected();
        if (root.escolhidos[1] !== "kitPath|/opt") failures += 32;
        picker.currentPath = "";
        picker.openSelected();
        if (root.escolhidos.length !== 2) failures += 64;

        // Proposito vazio/indefinido volta ao padrao; startPath vazio e' "/".
        picker.openFor("", "");
        if (picker.purpose !== "workspace" || root.navegados[2] !== "/") failures += 128;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
