import QtQuick
import KineinVectis

// 16.4b (roadmaps/59 §7.1.4): sem a confianca da pessoa, a faixa de saude
// nao pede o configure automatico (ele executa o CMakeLists.txt do projeto)
// e oferece o gesto; depois do gesto, pede. O despacho da acao "trust" chega
// ao core pelo ProjectHealthActions real.
Item {
    id: root

    property int configureRequests: 0
    property var calls: []
    property int failures: 0

    function check(condition, label) {
        if (!condition) { root.failures++; console.error("FALHOU: " + label); }
    }

    ProjectHealthController {
        id: controller

        workspaceRoot: "/tmp/hostil"
        workspaceKind: "cmake"
        workspaceBuildSystems: ["cmake", "cargo"]
        toolsList: [
            { id: "cmake", status: "detected" }, { id: "ninja", status: "detected" },
            { id: "clangd", status: "detected" }, { id: "gxx", status: "detected" },
            { id: "cargo", status: "detected" }, { id: "rustc", status: "detected" },
            { id: "rust-analyzer", status: "detected" }
        ]
        onAutoConfigureRequested: root.configureRequests += 1
    }

    // 0.4.0e: o projeto PlatformIO tambem pede o gesto (o `compiledb` roda
    // os scripts do projeto), e cobra o `pio` antes.
    ProjectHealthController {
        id: pio

        workspaceRoot: "/tmp/pio"
        workspaceKind: "platformIo"
        workspaceBuildSystems: ["platformIo"]
        toolsList: [{ id: "clangd", status: "detected" }, { id: "pio", status: "missing" }]
    }

    QtObject {
        id: fakeCore
        function trustWorkspace() { root.calls = root.calls.concat(["trust"]); }
        function cmakeConfigure() { root.calls = root.calls.concat(["configure"]); }
        function scanEnvironment() { root.calls = root.calls.concat(["scan"]); }
        function cargoMetadata() { root.calls = root.calls.concat(["metadata"]); }
    }
    QtObject {
        id: fakeShell
        function showTab(tab) { root.calls = root.calls.concat(["tab:" + tab]); }
    }
    ProjectHealthActions {
        id: actions
        coreClient: fakeCore
        shellController: fakeShell
    }

    Component.onCompleted: {
        controller.handleCmakeStatus(false, false, "", "");
        root.check(root.configureRequests === 0, "sem confiança, nada de configure automático");
        root.check(controller.active && controller.actionTarget === "trust",
            "a faixa oferece o gesto: " + controller.actionTarget);
        root.check(controller.message.indexOf("não confiável") >= 0, controller.message);

        actions.perform(controller.actionTarget);
        root.check(JSON.stringify(root.calls) === '["trust"]', "o gesto chega ao core: " + JSON.stringify(root.calls));

        controller.handleTrust(true);
        root.check(root.configureRequests === 1, "com a confiança, o configure que esperava roda");
        root.check(controller.actionTarget !== "trust", "a faixa sai do aviso de confiança");
        controller.handleTrust(true);
        root.check(root.configureRequests === 1, "uma vez só por workspace");

        // As outras ações continuam iguais (o despacho saiu do Main.qml).
        actions.perform("cmakeConfigure");
        actions.perform("jobs");
        root.check(JSON.stringify(root.calls) === '["trust","tab:jobs","configure","tab:jobs"]',
            "despacho: " + JSON.stringify(root.calls));

        root.check(pio.actionTarget === "tools" && pio.message.indexOf("pio") >= 0,
            "sem pio, a faixa cobra a ferramenta: " + pio.message);
        pio.toolsList = [{ id: "clangd", status: "detected" }, { id: "pio", status: "detected" }];
        root.check(pio.active && pio.actionTarget === "trust" && pio.message.indexOf("pio") >= 0,
            "PlatformIO pede o gesto: " + pio.message);
        pio.handleTrust(true);
        root.check(!pio.active, "com a confiança, nada a dizer: " + pio.message);
        // Misto (CMakeLists do ESP-IDF ao lado): o tipo e' cmake, o MOTOR e' o
        // pio, e ele e' cobrado.
        pio.workspaceKind = "cmake";
        pio.workspaceBuildSystems = ["cmake", "platformIo"];
        pio.toolsList = [
            { id: "cmake", status: "detected" }, { id: "ninja", status: "detected" },
            { id: "clangd", status: "detected" }, { id: "gxx", status: "detected" },
            { id: "pio", status: "missing" }
        ];
        root.check(pio.actionTarget === "tools" && pio.message.indexOf("pio") >= 0,
            "misto: o pio e' cobrado: " + pio.message);
        Qt.exit(root.failures === 0 ? 0 : 1);
    }
}
