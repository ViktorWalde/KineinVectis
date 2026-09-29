import QtQuick

// Selecao da arvore visivel. Paths sao a identidade; indices so servem para
// calcular o intervalo no retrato atual do ListModel.
Item {
    id: root

    property var model: null
    property var selectedPaths: []
    property string selectedPath: ""
    property string selectedKind: ""
    property string anchorPath: ""

    visible: false

    function clear() {
        selectedPaths = [];
        selectedPath = "";
        selectedKind = "";
        anchorPath = "";
    }

    function indexOf(path) {
        if (model === null) return -1;
        for (let i = 0; i < model.count; ++i) {
            if (model.get(i).path === path) return i;
        }
        return -1;
    }

    function kindOf(path) {
        const index = indexOf(path);
        return index < 0 ? "" : model.get(index).kind;
    }

    function select(path, kind, modifiers) {
        const ctrl = (modifiers & Qt.ControlModifier) !== 0;
        const shift = (modifiers & Qt.ShiftModifier) !== 0;
        const index = indexOf(path);
        const anchor = indexOf(anchorPath);
        if (shift && index >= 0 && anchor >= 0) {
            const paths = ctrl ? selectedPaths.slice() : [];
            for (let i = Math.min(index, anchor); i <= Math.max(index, anchor); ++i) {
                const candidate = model.get(i).path;
                if (paths.indexOf(candidate) < 0) paths.push(candidate);
            }
            selectedPaths = paths;
            selectedPath = path;
            selectedKind = kind;
            return;
        }
        if (ctrl && index >= 0) {
            const paths = selectedPaths.slice();
            const existing = paths.indexOf(path);
            if (existing >= 0) paths.splice(existing, 1);
            else paths.push(path);
            selectedPaths = paths;
            selectedPath = paths.length > 0 ? paths[paths.length - 1] : "";
            selectedKind = selectedPath === path ? kind : kindOf(selectedPath);
            anchorPath = path;
            return;
        }
        selectedPaths = [path];
        selectedPath = path;
        selectedKind = kind;
        anchorPath = path;
    }

    function selectAll() {
        if (model === null || model.count === 0) {
            clear();
            return;
        }
        const paths = [];
        for (let i = 0; i < model.count; ++i) paths.push(model.get(i).path);
        selectedPaths = paths;
        if (indexOf(selectedPath) < 0) {
            selectedPath = paths[0];
            selectedKind = model.get(0).kind;
        }
        if (indexOf(anchorPath) < 0) anchorPath = selectedPath;
    }

    // Uma listagem ou collapse pode remover linhas. Nenhuma operacao futura
    // pode agir numa selecao que ja nao esta visivel.
    function reconcile() {
        const paths = selectedPaths.filter(path => indexOf(path) >= 0);
        selectedPaths = paths;
        if (paths.indexOf(selectedPath) < 0) {
            selectedPath = paths.length > 0 ? paths[paths.length - 1] : "";
        }
        selectedKind = kindOf(selectedPath);
        if (indexOf(anchorPath) < 0) anchorPath = selectedPath;
    }
}
