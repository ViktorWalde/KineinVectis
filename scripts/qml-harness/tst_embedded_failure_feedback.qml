import QtQuick
import KineinVectis

// Superfícies e controllers reais. Nenhum pedido abre dispositivo/ferramenta.
Item {
    id: root
    width: 320
    height: 1600
    property int failures: 0
    property int phase: 0
    // Column positioners settle on a rendered frame, not in the JS callback.
    Timer {
        id: settle
        interval: 30
        onTriggered: {
            const step = root.phase++;
            if (step === 0) root.verifyNarrow();
            else if (step === 1) root.verifyWide();
            else root.verifyRecovery();
        }
    }
    readonly property string reason: "<a href=\"https://example.invalid/tool\">ferramenta</a>: porta /dev/serial/by-id/adaptador-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx. Feche o monitor no Terminal e tente novamente."
    EmbeddedController { id: controller; workspaceRoot: "/fixture" }
    Column {
        anchors.left: parent.left
        anchors.right: parent.right
        spacing: 20
        EmbeddedIdentityView { id: identity; width: parent.width; controller: controller }
        EmbeddedFilesView { id: files; width: parent.width; controller: controller }
        EmbeddedFlashView { id: flash; width: parent.width; controller: controller }
    }
    function check(ok, message) {
        if (!ok) { failures++; console.error("FALHOU: " + message); }
    }
    function find(item, name) {
        if (!item || item.children === undefined) return null;
        if (item.objectName === name) return item;
        for (let index = 0; index < item.children.length; index++) {
            const found = find(item.children[index], name);
            if (found) return found;
        }
        return null;
    }
    function findText(item, raw) {
        if (!item || item.children === undefined) return null;
        if (item.text === raw && typeof item.linkAt === "function") return item;
        for (let index = 0; index < item.children.length; index++) {
            const found = findText(item.children[index], raw);
            if (found) return found;
        }
        return null;
    }
    function checkBand(view, name) {
        const band = find(view, name);
        check(band !== null && band.visible, name + ": causa visível na superfície");
        if (!band) return;
        check(band.heading !== "" && band.message === reason, name + ": título e causa integral do backend");
        const text = findText(band, reason);
        check(text !== null, name + ": mensagem desenhada");
        if (!text) return;
        let links = false;
        for (let y = 0; y < text.height && !links; y += 2) {
            for (let x = 0; x < text.width; x += 2) {
                if (text.linkAt(x, y) !== "") { links = true; break; }
            }
        }
        check(!links, name + ": mensagem do backend é literal, sem links HTML");
        check(text.contentHeight <= text.height + 1 && text.contentWidth <= text.width + 1 && text.width <= view.width,
              name + ": causa longa cabe sem recorte");
        check(band.y + band.height <= view.height + 1, name + ": erro ocupa espaço no painel (" + root.width + ": " + band.y + "+" + band.height + " / " + view.height + ")");
    }
    Component.onCompleted: {
        controller.identity.identify("/fixture-port");
        controller.identity.handleFailed("serial.identify", reason);
        controller.files.list("/fixture-port");
        controller.files.handleFailed("serial.files", reason);
        controller.flash.propose("/fixture-port", 0);
        controller.flash.bindProposal(1);
        controller.flash.handleProposalFailed(1, reason);
        settle.start();
    }
    function verifyNarrow() {
        checkBand(identity, "identityFailure");
        checkBand(files, "filesFailure");
        checkBand(flash, "flashFailure");
        controller.identity.identity = {chip: "esp32"};
        width = 736;
        settle.start();
    }
    function verifyWide() {
        checkBand(identity, "identityFailure");
        checkBand(files, "filesFailure");
        checkBand(flash, "flashFailure");
        check(find(identity, "identityFailure").heading === "Identificação incompleta",
              "identidade parcial permanece visível sem anunciar sucesso completo");
        controller.identity.clear();
        controller.files.clear();
        controller.flash.clear();
        settle.start();
    }
    function verifyRecovery() {
        check(!find(identity, "identityFailure").visible && !identity.visible,
              "nova tentativa remove causa anterior de identificação");
        check(!find(files, "filesFailure").visible && !files.visible,
              "limpeza remove causa anterior dos arquivos");
        check(!find(flash, "flashFailure").visible,
              "prévia limpa remove faixa de erro");
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
