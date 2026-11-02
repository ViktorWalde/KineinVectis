import QtQuick
import KineinVectis

Item {
    id: root
    property int failures: 0
    property int applies: 0
    property int trusts: 0
    property int cancels: 0
    property string content: "// ação 😀\nint value = 1;\n"
    function check(ok, message) { if (!ok) { failures++; console.error(message); } }

    ListModel { id: tabs; ListElement { path: "/project/main.cpp"; modified: false } }
    QtObject {
        id: documents
        property var filesModel: tabs
        property int currentTab: 0
        function currentFilePath() { return "/project/main.cpp"; }
    }
    QtObject { id: surface; function text() { return root.content; } }
    EditorLanguageController {
        id: language
        documentController: documents
        surfaceBridge: surface
        onWorkspaceEditApplyRequested: root.applies++
        onWorkspaceTrustRequested: root.trusts++
        onWorkspaceEditCancelRequested: root.cancels++
    }
    EditorWorkspaceEditPreviewDialog {
        id: preview
        verification: language.editVerification.evidence
        obsolete: language.editVerification.obsolete
        files: language.workspaceEditFiles
        onApplyRequested: language.applyWorkspaceEdit()
        onTrustRequested: language.trustVerificationWorkspace()
    }
    EditorEditVerificationSummary { id: summary; evidence: language.editVerification.evidence }

    Component.onCompleted: {
        const files = [{ path: "/project/main.cpp", beforeContent: content, afterContent: "int value = missing;\n", edits: 1 }];
        const evidence = {status: "newError", reason: "Não prova equivalência, link ou testes.", durationMs: 37,
            compiler: "clang++", directory: "/project", arguments: ["-DNAME=a b", "main.cpp"],
            contextId: "context", documentVersion: 7, beforeErrors: 0, afterErrors: 1,
            newErrors: [{file: "/project/main.cpp", line: 1, message: "undeclared identifier missing"}]};
        language.handleWorkspaceEditPreview("tx1", "Correction", files, 1, evidence);
        check(applies === 0 && summary.heading === "Erro novo observado", "new errors never auto apply");
        check(summary.details().indexOf("undeclared identifier") >= 0 && summary.details().indexOf("37 ms") >= 0,
              "diagnostic and duration visible in real presentation");
        preview.applyRequested();
        check(applies === 1, "explicit confirmation uses the existing transaction");
        content += "// newer buffer\n";
        // No didChange notification yet: applying must compare the live surface.
        preview.applyRequested();
        check(applies === 1 && preview.obsolete && summary.heading === "Não verificado", "live buffer guard prevents overwrite");
        content = files[0].beforeContent;
        language.handleWorkspaceEditPreview("tx2", "Correction", files, 1, {status: "noNewError", reason: "observed"});
        check(summary.heading === "Sem erro novo observado", "all three outcomes have exact labels");
        language.editVerification.invalidate("CDB changed");
        preview.applyRequested();
        check(applies === 1 && summary.details() === "CDB changed", "changed context retires positive evidence");
        tabs.append({path: "/project/header.h", modified: true});
        language.handleWorkspaceEditPreview("tx3", "Correction", files, 1, evidence);
        preview.applyRequested();
        check(applies === 1 && preview.obsolete, "unsaved other buffer cannot inherit disk evidence");
        tabs.setProperty(1, "modified", false);
        language.handleWorkspaceEditPreview("tx4", "Correction", files, 1,
                                           {status: "notVerified", obsolete: true, reason: "cancelled"});
        preview.applyRequested();
        check(applies === 1 && preview.obsolete, "obsolete producer reply cannot reactivate apply");
        language.handleWorkspaceEditPreview("tx5", "Correction", files, 1,
                                           {status: "noNewError", reason: "observed"});
        language.handleRequestFailed("lsp.workspaceEdit.apply", "context changed");
        check(language.editVerification.obsolete && summary.heading === "Não verificado",
              "an apply rejection cannot retain positive evidence");
        language.handleWorkspaceEditPreview("tx6", "Correction", files, 1,
            {status: "notVerified", reasonCode: "untrustedWorkspace", reason: "trust required"});
        check(summary.trustRequired, "configured projects expose trust in the consumer");
        preview.trustRequested();
        check(trusts === 1 && cancels === 1 && applies === 1, "trust cancels the old preview without applying");
        language.clear();
        check(language.editVerification.evidence === null, "workspace reset forgets evidence");
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
