import QtQuick

// Alvos de build do projeto e qual deles está ativo (L3 fatia 2).
//
// O `cmake.targets.list` existia no core desde 2026-07-16 e NENHUMA parte da
// UI o consumia — medido em 2026-08-21. Esta é a ponta que faltava.
//
// A regra que este controller carrega: **alvo ativo tem que MUDAR o build.**
// Um seletor que não altera o comando seria a "propaganda de rail" que a
// §12.2 proíbe, em outra roupa — ocupa espaço e não faz nada.
Item {
    id: root

    property string workspaceRoot: ""
    // "" significa "todos os alvos", que é o comportamento do `cmake --build`
    // sem `--target` e continua sendo o padrão: quem nunca escolheu nada não
    // pode ter o build silenciosamente reduzido a um alvo.
    property string activeTarget: ""
    property bool menuVisible: false
    property real menuX: 0
    property real menuY: 0

    readonly property alias targetsModel: targetItems
    readonly property int targetCount: targetItems.count

    // Rótulo do chip da barra. Sem alvos conhecidos ele não mente dizendo
    // "todos": diz que ainda não sabe, porque configure é o gesto que falta.
    readonly property string activeLabel: {
        if (targetItems.count === 0) {
            return qsTr("alvos não detectados");
        }
        return activeTarget !== "" ? activeTarget : qsTr("todos os alvos");
    }

    signal listRequested()

    ListModel {
        id: targetItems
    }

    function clear() {
        targetItems.clear();
        activeTarget = "";
        menuVisible = false;
    }

    function refresh() {
        if (workspaceRoot === "") {
            return;
        }
        listRequested();
    }

    function handleTargets(targets) {
        targetItems.clear();
        if (!targets) {
            activeTarget = "";
            return;
        }
        for (const target of targets) {
            targetItems.append({
                name: target.name !== undefined ? target.name : "",
                kind: target.kind !== undefined ? target.kind : ""
            });
        }
        // Alvo ativo que sumiu da lista (renomeado, removido do CMakeLists)
        // não pode continuar selecionado: o build falharia com um erro do
        // CMake que não explica que a escolha da barra é que está velha.
        if (activeTarget !== "" && !hasTarget(activeTarget)) {
            activeTarget = "";
        }
    }

    function hasTarget(name) {
        for (let i = 0; i < targetItems.count; i++) {
            if (targetItems.get(i).name === name) {
                return true;
            }
        }
        return false;
    }

    function choose(name) {
        menuVisible = false;
        if (name === "" || hasTarget(name)) {
            activeTarget = name;
        }
    }

    function openMenu(x, y) {
        menuX = x;
        menuY = y;
        menuVisible = true;
    }

    function closeMenu() {
        menuVisible = false;
    }
}
