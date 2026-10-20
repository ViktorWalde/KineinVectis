import QtQuick
import KineinVectis

// Real owners: accepted context -> facts and health -> inspector action.
// Toolchain replies must invalidate the context instead of leaving an old alert.
Item {
    id: root
    property int failures: 0
    property int requests: 0
    function check(ok, message) {
        if (!ok) { failures++; console.error(message); }
    }
    function answer(path, configured) {
        return {path: path, language: "cpp", drift: [{kind: "target", configured: configured,
            effective: {value: "arm-none-eabi", origin: "cdb"}, evidence: "explicit target option",
            action: "inspectKit"}]};
    }

    IndexController {
        id: index
        workspaceRoot: "/project"
        onContextRequested: root.requests++
    }
    ProjectHealthController {
        id: health
        workspaceRoot: index.workspaceRoot
        workspaceTrusted: true
        workspaceKind: "cmake"
        workspaceBuildSystems: ["cmake"]
        toolsList: [{id: "cmake", status: "detected"}, {id: "ninja", status: "detected"},
                    {id: "clangd", status: "detected"}, {id: "gxx", status: "detected"}]
        compileContext: index.fileContext
    }
    ProjectHealthActions { id: actions; indexController: index }
    ToolchainController { id: toolchain }
    QtObject {
        id: bridge
        signal toolchainResolved(var selections, var candidates, string preset, string sysroot,
                                 string targetTriple, string chip, string presetToolchainFile)
        signal cmakePresetsResolved(var presets)
        signal toolchainAdvice(string sysrootHint, var rustTargets, bool rustTargetsKnown)
        signal toolchainKitFileResolved(string toolchainFile)
        signal toolchainKitSvdResolved(string svdFile)
        signal sysrootInspected(var report)
        signal kitImported(var proposal)
        signal toolchainInstallableResolved(var toolchains, string installRoot, string projectFamily)
        signal toolchainInstalled(var outcome)
        signal requestFailed(string method, string message)
    }
    ToolchainEventRouter {
        coreClient: bridge
        toolchainController: toolchain
        indexController: index
    }

    Component.onCompleted: {
        health.handleCmakeStatus(true, false, "");
        index.setActivePath("/project/main.cpp");
        const reply = answer(index.contextPath, "aarch64-linux-gnu");
        index.handleContext(reply);
        check(health.active && health.status === "warning"
              && health.actionTarget === "compileContext" && health.message.indexOf("main.cpp") >= 0,
              "accepted differences produce an actionable file-specific alert");
        const first = index.contextFacts.rows[0];
        check(first.warning && first.label === "Diferença de target"
              && first.value.indexOf("aarch64-linux-gnu") >= 0 && first.value.indexOf("arm-none-eabi") >= 0
              && first.value.indexOf("explicit target option") >= 0
              && first.origin.indexOf("compile_commands") >= 0,
              "differences precede facts and show both values, evidence and origin");
        actions.perform(health.actionTarget);
        check(index.inspectorVisible, "health action opens the real inspector owner");
        index.closeInspector();
        health.dismiss();
        check(!health.active, "dismiss hides this alert");
        index.handleContext(JSON.parse(JSON.stringify(reply)));
        check(!health.active, "an identical reply stays dismissed");
        index.handleContext(answer(index.contextPath, "riscv32-unknown-elf"));
        check(health.active, "changed differences reappear even with the same banner text");
        health.dismiss();
        index.setActivePath("/project/other/main.cpp");
        check(!health.active, "changing file clears the old alert while loading");
        index.handleContext(reply);
        check(!health.active, "late reply for the previous file cannot restore it");
        index.handleContext(answer(index.contextPath, "riscv32-unknown-elf"));
        check(health.active, "same basename at another path has its own alert");

        health.handleCmakeStatus(true, true, "CMakeLists.txt");
        check(health.actionTarget === "cmakeConfigure", "stale CDB has priority over drift");
        health.handleCmakeStatus(false, false, "");
        check(health.actionTarget !== "compileContext", "missing configure has priority");
        health.handleTrust(false);
        check(health.actionTarget === "trust", "trust has priority");
        const tools = health.toolsList;
        health.toolsList = [{id: "clangd", status: "detected"}];
        check(health.actionTarget === "tools", "missing tools have priority");
        health.toolsList = tools;
        health.handleCmakeStatus(true, false, "");
        health.handleTrust(true);
        check(health.actionTarget === "compileContext", "drift returns after higher priority problems clear");

        const before = root.requests;
        bridge.toolchainResolved({}, {}, "cross", "", "arm-none-eabi", "", "");
        check(toolchain.preset === "cross", "toolchain reply still reaches its owner");
        check(root.requests === before + 1 && index.contextLoading && !health.active,
              "kit reply invalidates and re-requests the active context");
        index.handleContext({path: index.contextPath, language: "cpp"});
        check(!health.active && !index.contextFacts.rows.some(row => row.warning),
              "agreement removes the alert and difference rows");
        index.handleContext(answer(index.contextPath, "aarch64-linux-gnu"));
        index.workspaceRoot = "/other";
        check(!health.active, "workspace change clears drift");
        const closed = root.requests;
        index.refreshContext();
        check(root.requests === closed, "no active path never sends a context query");
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
