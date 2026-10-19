import QtQuick
import KineinVectis

// Actual presentation, lifecycle, menu and command owners. Replies and
// navigation are observed at their boundaries, including late replies.
Item {
    id: root
    property int failures: 0
    property int requests: 0
    property var opened: null
    property string kit: "unrequested"

    function check(ok, message) {
        if (!ok) { failures++; console.error(message); }
    }
    function row(label) { return index.contextFacts.rows.find(entry => entry.label === label); }

    IndexController {
        id: index
        onContextRequested: root.requests++
        onOpenSymbolRequested: (file, line, column) => root.opened = { file, line, column }
        onOpenKitRequested: preset => root.kit = preset
    }
    CommandDispatcher { id: dispatcher; indexController: index }
    AppMenuItems { id: menus }

    Component.onCompleted: {
        index.openInspector();
        check(!index.inspectorVisible && requests === 0, "no workspace stays closed");
        index.workspaceRoot = "/project";
        check(dispatcher.execute("index.context") && index.inspectorVisible,
              "palette opens the actual inspector owner");
        check(index.contextFacts.rows.length === 0 && !index.contextLoading, "no file has no invented facts");
        index.setActivePath("/project/main.cpp");
        check(index.contextLoading && requests === 1, "active file starts loading");
        const answer = {
            path: "/project/main.cpp", language: "cpp", source: "compile_commands.json em build",
            unit: { directory: "/project/build", includes: ["/sdk/include", "/project/include"],
                    defines: ["NAME=a b"], arguments: ["clang++", "-DNAME=a b", "-c", "main.cpp"] },
            cdbLocation: {path: "/project/build/compile_commands.json", line: 12, column: 3},
            effective: { compiler: {value: "clang++", origin: "cdb"},
                         standard: {value: "c++23", origin: "cdb"},
                         sysroot: {value: "/sdk", origin: "kit"}, flagsOrigin: "cdb", kit: {preset: "cross"} },
            targets: ["app"]
        };
        index.handleContext(answer);
        check(!index.contextLoading, "reply ends loading");
        check(row("Compilador").value === "clang++" && row("Compilador").origin.indexOf("compile_commands") >= 0, "compiler with CDB origin");
        check(row("Sysroot").value === "/sdk" && row("Sysroot").origin === "do kit", "kit fallback stays labelled kit");
        check(row("Target").value === "não informado" && row("Target").origin === "", "unknown target is explicit");
        check(row("Includes (2)").value === "/sdk/include\n/project/include", "include order survives");
        check(row("Defines (1)").value === "NAME=a b", "defines retain spaces");
        check(row("Argumentos (4)").value === "clang++\n-DNAME=a b\n-c\nmain.cpp", "argument boundaries survive");
        index.openContextKit();
        check(kit === "cross" && !index.inspectorVisible, "opens the effective kit, not a guessed current selection");
        index.openInspector();
        index.openContextCdb();
        check(opened.file === answer.cdbLocation.path && opened.line === 12 && opened.column === 3
              && !index.inspectorVisible, "CDB navigation uses exact object position");
        index.openInspector();
        index.setActivePath("/project/header.hpp");
        index.handleContext(answer);
        check(index.contextLoading && index.contextFacts.rows.length === 0, "late reply cannot fill another file");
        opened = null;
        index.openContextCdb();
        check(opened === null && index.inspectorVisible, "pending reply cannot navigate to stale CDB");
        index.handleContext({ path: "/project/header.hpp", language: "cpp", hint: "header without a unit",
                              effective: {compiler: {value: "g++", origin: "cmakeCache"}, kit: {preset: ""}} });
        check(row("Compilador").origin === "do cache do CMake", "cache fallback stays labelled cache");
        check(index.fileContext.hint === "header without a unit" && !row("Includes (0)"), "header never borrows includes");
        index.openContextCdb();
        check(opened === null, "no CDB location cannot navigate");
        index.openContextKit();
        check(kit === "", "default kit is a valid kit");
        index.handleContext({ path: "/project/header.hpp", language: "cpp", source: "file-api",
            unit: {directory: "/project/build", output: "x.o", includes: [], defines: [], arguments: []},
            effective: {flagsOrigin: "cmakeFileApi"} });
        check(row("Includes (0)").origin === "do file-api do CMake", "fallback origins are retained");
        index.setActivePath("/project/main.py");
        index.handleContext({path: "/project/main.py", language: "python", python: {interpreter: "/venv/python", origin: ".venv"}});
        check(row("Interpretador Python").value === "/venv/python" && !row("Compilador"), "Python doesn't invent a compiler");
        index.setActivePath("/project/lib.rs");
        index.handleContext({path: "/project/lib.rs", language: "rust", crate: {
            package: "motor", target: "motor", kind: "lib", edition: "2024", manifest: "/project/Cargo.toml", features: ["std"]}});
        check(row("Pacote Rust").value === "motor" && row("Features").value === "std", "Cargo facts are shown");
        menus.workspaceOpen = true;
        for (const menu of ["code", "environment"]) {
            check(menus.menuItems(menu).find(entry => entry.action === "index.context").enabled === false,
                  menu + " requires an active file");
            menus.hasActiveFile = true;
            check(menus.menuItems(menu).find(entry => entry.action === "index.context").enabled,
                  menu + " enables context for the active file");
            menus.hasActiveFile = false;
        }
        index.openInspector();
        index.workspaceRoot = "/other";
        check(!index.inspectorVisible && index.contextPath === "" && index.contextFacts.rows.length === 0,
              "workspace change closes and clears");
        index.handleContext(answer);
        check(index.contextFacts.rows.length === 0, "reply from closed workspace is discarded");
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
