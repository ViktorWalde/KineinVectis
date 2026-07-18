import QtQuick

// Interpreta GESTO sobre a arvore ja montada: para onde o arraste pode cair e
// para onde a seta leva a selecao. Nao guarda estado — o dono do modelo, da
// selecao e dos dialogos continua sendo o `ProjectTreeController`.
//
// Existe separado porque decidir "este drop e valido?" e "esta tecla move para
// onde?" e uma responsabilidade diferente de "guardar em que linha o usuario
// esta". Foram estas funcoes que empurraram o controller de 306 para 420 linhas
// (limite 400): o suspeito era a mudanca, nao o arquivo (§4 regra 9).
Item {
    id: root

    // O ProjectTreeController. Fonte do modelo, da selecao e do workspaceRoot.
    property var tree: null

    visible: false

    // --- arraste: quem pode cair onde ---------------------------------------
    //
    // O core ja recusa destino existente, fora do workspace e a propria raiz
    // (`fsops::rename`). O que a UI decide aqui e o que o core NAO tem como
    // decidir bem: soltar sobre um ARQUIVO significa soltar na pasta dele, e um
    // destino invalido tem que ficar visivelmente morto ANTES do gesto — erro
    // cru de `EINVAL` depois de arrastar nao ensina nada a quem esta usando.

    function dropTargetDir(targetPath, targetKind) {
        if (targetPath === "") {
            return tree.workspaceRoot;
        }
        return targetKind === "directory" ? targetPath : tree.parentDir(targetPath);
    }

    function canDropInto(sourcePath, destinationDir) {
        if (sourcePath === "" || destinationDir === ""
                || tree.workspaceRoot === "") {
            return false;
        }
        // Uma pasta dentro de si mesma: o `fs::rename` do Linux devolve EINVAL,
        // entao nao corrompe — mas o alvo tem que estar morto na tela.
        if (sourcePath === destinationDir
                || destinationDir.indexOf(sourcePath + "/") === 0) {
            return false;
        }
        // Ja esta la: mover para o proprio pai e gesto sem efeito, e deixar o
        // alvo aceso convida a um no-op que parece ter funcionado.
        return tree.parentDir(sourcePath) !== destinationDir;
    }

    function canDropOn(sourcePath, targetPath, targetKind) {
        return canDropInto(sourcePath, dropTargetDir(targetPath, targetKind));
    }

    function dropOn(sourcePath, targetPath, targetKind) {
        const destination = dropTargetDir(targetPath, targetKind);
        if (canDropInto(sourcePath, destination)) {
            tree.moveEntry(sourcePath, destination);
        }
    }

    // --- navegacao por teclado (memoria muscular JetBrains) ------------------

    function rowAtSelection() {
        const index = tree.rowIndexForPath(tree.selectedPath);
        return index < 0 ? null : { index: index, row: tree.entriesModel.get(index) };
    }

    function moveSelection(delta) {
        const model = tree.entriesModel;
        if (model.count === 0) {
            return;
        }
        let index = tree.rowIndexForPath(tree.selectedPath);
        if (index < 0) {
            index = delta > 0 ? -1 : model.count;
        }
        const next = Math.max(0, Math.min(model.count - 1, index + delta));
        const row = model.get(next);
        tree.selectEntry(row.path, row.kind);
    }

    function expandSelected() {
        const at = rowAtSelection();
        if (at === null) {
            return;
        }
        if (at.row.kind === "directory" && !at.row.expanded) {
            tree.listDirRequested(at.row.path);
            return;
        }
        // Pasta ja aberta (ou arquivo): a seta desce para o primeiro filho.
        moveSelection(1);
    }

    function collapseSelected() {
        const at = rowAtSelection();
        if (at === null) {
            return;
        }
        if (at.row.kind === "directory" && at.row.expanded) {
            tree.collapseRow(at.index);
            return;
        }
        // Item folha: a seta sobe para a pasta-mae, sem escapar da raiz.
        const parent = tree.parentDir(at.row.path);
        if (parent !== at.row.path && tree.rowIndexForPath(parent) >= 0) {
            tree.selectEntry(parent, "directory");
        }
    }

    function activateSelected() {
        const at = rowAtSelection();
        if (at === null) {
            return;
        }
        if (at.row.kind === "directory") {
            tree.toggleDirectory(at.row.path, at.index, at.row.expanded);
        } else {
            tree.readFileRequested(at.row.path);
        }
    }
}
