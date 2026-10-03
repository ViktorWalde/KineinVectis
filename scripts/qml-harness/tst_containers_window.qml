// A JANELA DE CONTAINERS PROMETE SO' O QUE FUNCIONA.
//
// Por que existe (2026-09-13, como tst_container_panel): o tst_container prova
// o controller; a tela e' burra e por isso nunca era instanciada — e foi nela
// que "compose up" ficou aceso sem projeto. Desde 2026-10-03 a tela e' a
// JANELA acoplada (ContainersWindow), e a prova veio junto: os botoes de
// compose sao lidos pelo texto, ligados so' quando o controller diz
// `canCompose`, e a linha do compose diz o que falta.
//
// MUTACAO QUE PROVA O GATE: troque o `enabled` do "compose up" por
// `root.controller !== null` e a assercao 2 cai.
import QtQuick
import KineinVectis

Item {
    id: root

    width: 900
    height: 700

    ContainerController { id: ctl; workspaceRoot: "" }

    ContainersWindow {
        id: window
        anchors.fill: parent
        controller: ctl
    }

    function button(label, item) {
        if (item === undefined) item = window;
        for (let i = 0; i < item.children.length; i++) {
            const child = item.children[i];
            if (child.hasOwnProperty("primary") && child.hasOwnProperty("text") && child.text === label) return child;
            const found = root.button(label, child);
            if (found !== null) return found;
        }
        return null;
    }

    function textContaining(fragment, item) {
        if (item === undefined) item = window;
        for (let i = 0; i < item.children.length; i++) {
            const child = item.children[i];
            if (child.hasOwnProperty("wrapMode") && typeof child.text === "string" && child.text.indexOf(fragment) >= 0
                    && child.visible) return child;
            const found = root.textContaining(fragment, child);
            if (found !== null) return found;
        }
        return null;
    }

    Component.onCompleted: {
        let failures = 0;
        const up = root.button("compose up");
        const down = root.button("compose down");
        if (up === null || down === null) { console.error("botoes de compose nao achados"); Qt.exit(1); return; }

        // Sem projeto, com o motor e a ferramenta: desligados, e a linha diz.
        ctl.handleStatus({ engine: "podman", version: "5.8.4", emulated: true, reachable: true,
                                  compose: "/usr/bin/podman compose" });
        if (up.enabled || down.enabled) failures += 2;
        if (root.textContaining("abra um projeto") === null) failures += 4;

        // Com projeto sem arquivo: desligados, e a linha diz o que criar.
        ctl.workspaceRoot = "/tmp/proj";
        if (up.enabled || down.enabled) failures += 8;
        if (root.textContaining("não tem compose.yaml") === null) failures += 16;

        // Com o arquivo: ligados, e a linha mostra qual.
        ctl.handleStatus({ engine: "podman", version: "5.8.4", emulated: true, reachable: true,
                                  compose: "/usr/bin/podman compose", composeFile: "compose.yaml" });
        if (!up.enabled || !down.enabled) failures += 32;
        if (root.textContaining("compose.yaml") === null) failures += 64;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
