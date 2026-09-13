// O PAINEL DE CONTAINERS PROMETE SO' O QUE FUNCIONA.
//
// Por que existe (2026-09-13): o tst_container prova o controller; o painel e'
// burro e por isso nunca era instanciado — e foi no painel que "compose up"
// ficou aceso sem projeto. Aqui o ContainerPanel REAL nasce sobre o controller
// real, e os botoes de compose sao lidos pelo texto: ligados so' quando o
// controller diz `canCompose`, e a linha do motor diz o que falta.
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

    ContainerPanel {
        id: painel
        anchors.fill: parent
        controller: ctl
    }

    function botao(texto, item) {
        if (item === undefined) item = painel;
        for (let i = 0; i < item.children.length; i++) {
            const filho = item.children[i];
            if (filho.hasOwnProperty("primary") && filho.hasOwnProperty("text") && filho.text === texto) return filho;
            const achado = root.botao(texto, filho);
            if (achado !== null) return achado;
        }
        return null;
    }

    function textoQueContem(trecho, item) {
        if (item === undefined) item = painel;
        for (let i = 0; i < item.children.length; i++) {
            const filho = item.children[i];
            if (filho.hasOwnProperty("wrapMode") && typeof filho.text === "string" && filho.text.indexOf(trecho) >= 0
                    && filho.visible) return filho;
            const achado = root.textoQueContem(trecho, filho);
            if (achado !== null) return achado;
        }
        return null;
    }

    Component.onCompleted: {
        let failures = 0;
        const up = root.botao("compose up");
        const down = root.botao("compose down");
        if (up === null || down === null) { console.error("botoes de compose nao achados"); Qt.exit(1); return; }

        // Sem projeto, com o motor e a ferramenta: desligados, e a linha diz.
        ctl.open();
        ctl.handleStatus({ engine: "podman", version: "5.8.4", emulated: true, reachable: true,
                                  compose: "/usr/bin/podman compose" });
        if (up.enabled || down.enabled) failures += 2;
        if (root.textoQueContem("abra um projeto") === null) failures += 4;

        // Com projeto sem arquivo: desligados, e a linha diz o que criar.
        ctl.workspaceRoot = "/tmp/proj";
        if (up.enabled || down.enabled) failures += 8;
        if (root.textoQueContem("não tem compose.yaml") === null) failures += 16;

        // Com o arquivo: ligados, e a linha mostra qual.
        ctl.handleStatus({ engine: "podman", version: "5.8.4", emulated: true, reachable: true,
                                  compose: "/usr/bin/podman compose", composeFile: "compose.yaml" });
        if (!up.enabled || !down.enabled) failures += 32;
        if (root.textoQueContem("compose.yaml") === null) failures += 64;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
