import QtQuick
import KineinVectis

// O seletor de toolchain reformulado (0.3.8 F3): os papeis DESTE projeto,
// pelos sistemas de build, vem primeiro; os outros ficam recolhidos; papel
// sem candidato detectado nao aparece; sem sistema, todos sao do projeto.
Item {
    id: root

    width: 1366
    height: 768

    QtObject {
        id: fakeController

        property string errorText: ""
        function candidatesFor(role) {
            return role === "serialMonitor" ? [] : [{ id: role + "-1", label: role }];
        }
        function selectionFor(role) { return null; }
        function labelFor(role) { return role; }
        function choose(role, id) {}
    }

    ToolchainMenu {
        id: menu

        anchors.fill: parent
        controller: fakeController
    }

    function keys(list) {
        return list.map(role => role.key).join(",");
    }

    Component.onCompleted: {
        let failures = 0;
        menu.buildSystems = ["cargo"];
        if (keys(menu.projectRoles) !== "cargo,debugAdapter") {
            console.error("cargo: " + keys(menu.projectRoles)); failures += 1;
        }
        if (menu.otherRoles.length !== 4) {
            console.error("outros sem o monitor sem candidato: " + keys(menu.otherRoles)); failures += 2;
        }
        if (menu.usesCmake) failures += 4;
        menu.buildSystems = ["cargo", "cmake"];
        if (keys(menu.projectRoles) !== "cxxCompiler,cCompiler,generator,cmake,cargo,debugAdapter") {
            console.error("hibrido: " + keys(menu.projectRoles)); failures += 8;
        }
        if (!menu.usesCmake) failures += 16;
        menu.buildSystems = [];
        if (menu.projectRoles.length !== 6 || menu.otherRoles.length !== 0) failures += 32;
        menu.toggle("cargo");
        menu.toggle("generator");
        if (menu.expandedKey !== "generator") failures += 64;
        menu.toggle("generator");
        if (menu.expandedKey !== "") failures += 128;
        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
