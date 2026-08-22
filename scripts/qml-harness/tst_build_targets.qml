import QtQuick
// Carrega o controller REAL dos alvos de build.
import "../../ui/qml/project"

Item {
    id: root
    width: 100
    height: 100

    property int pedidos: 0

    BuildTargetsController {
        id: alvos

        onListRequested: root.pedidos += 1
    }

    Component.onCompleted: {
        let failures = 0;

        // Sem workspace, não pede lista: o `cmake.targets.list` exige raiz.
        alvos.workspaceRoot = "";
        alvos.refresh();
        if (root.pedidos !== 0) failures += 1;

        // Sem alvos detectados, o rótulo NÃO mente dizendo "todos": diz que
        // ainda não sabe, porque configurar é o gesto que falta.
        if (alvos.activeLabel.indexOf("não detectados") < 0) failures += 2;
        if (alvos.targetCount !== 0) failures += 4;

        alvos.workspaceRoot = "/ws";
        alvos.refresh();
        if (root.pedidos !== 1) failures += 8;

        alvos.handleTargets([
            { name: "kinein-vectis", kind: "executable" },
            { name: "kinein-core", kind: "staticLibrary" }
        ]);
        if (alvos.targetCount !== 2) failures += 16;

        // O padrão é TODOS os alvos. Quem nunca escolheu nada não pode ter o
        // build silenciosamente reduzido a um alvo só.
        if (alvos.activeTarget !== "") failures += 32;
        if (alvos.activeLabel.indexOf("odos") < 0) failures += 64;

        // Escolher um alvo conhecido troca o ativo e fecha o menu.
        alvos.openMenu(10, 20);
        if (!alvos.menuVisible) failures += 128;
        alvos.choose("kinein-core");
        if (alvos.activeTarget !== "kinein-core") failures += 256;
        if (alvos.menuVisible) failures += 512;
        if (alvos.activeLabel !== "kinein-core") failures += 1024;

        // Alvo DESCONHECIDO não pode virar ativo: o build falharia com um
        // erro do CMake que não explica que a escolha da barra é que está má.
        alvos.choose("nunca-existiu");
        if (alvos.activeTarget !== "kinein-core") failures += 2048;

        // Voltar para "todos" é sempre possível.
        alvos.choose("");
        if (alvos.activeTarget !== "") failures += 4096;

        // Alvo ativo que SUMIU da lista (renomeado no CMakeLists, removido)
        // não pode continuar selecionado depois de reconfigurar.
        alvos.choose("kinein-core");
        alvos.handleTargets([{ name: "kinein-vectis", kind: "executable" }]);
        if (alvos.activeTarget !== "") failures += 8192;
        if (alvos.targetCount !== 1) failures += 16384;

        // Alvo que SOBREVIVEU à reconfiguração continua selecionado: perder a
        // escolha a cada configure seria hostil.
        alvos.choose("kinein-vectis");
        alvos.handleTargets([
            { name: "kinein-vectis", kind: "executable" },
            { name: "novo", kind: "executable" }
        ]);
        if (alvos.activeTarget !== "kinein-vectis") failures += 32768;

        // Resposta vazia limpa a lista sem manter escolha órfã.
        alvos.handleTargets(null);
        if (alvos.targetCount !== 0) failures += 65536;
        if (alvos.activeTarget !== "") failures += 131072;

        // clear() do workspace leva tudo junto.
        alvos.handleTargets([{ name: "x", kind: "executable" }]);
        alvos.choose("x");
        alvos.clear();
        if (alvos.targetCount !== 0) failures += 262144;
        if (alvos.activeTarget !== "") failures += 524288;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
