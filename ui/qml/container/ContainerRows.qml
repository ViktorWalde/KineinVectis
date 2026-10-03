import QtQuick
import KineinVectis
import "KvLists.js" as KvLists

// AS LINHAS da janela de Containers (2026-10-03, pedido do autor: a mesma
// base da janela do Banco, com inspiracao no Docker Desktop, no estilo da
// IDE). Puro: recebe o que o motor listou, o filtro e as secoes abertas, e
// devolve as linhas ja' achatadas — a janela so' desenha.
//
//   CONTAINERS · 3             secao (abre/fecha); ORDEM ESTAVEL, por nome
//     ○ db    postgres:16
//     ● web   nginx:1.27   8080 → 80/tcp
//   IMAGENS · 3
//     ▣ docker.io/library/nginx:1.27   187 MB
//
// O estado vem do MOTOR (`state`): "running" roda; "paused" pausou; o resto
// (exited, created, dead) esta' parado. A tela nao interpreta o `status`
// humano ("Up 2 hours"), que muda de idioma e de forma entre Docker e Podman.
QtObject {
    id: root

    property var containers: []
    property var images: []
    property string filter: ""
    // Secoes fechadas pelo usuario ("containers", "images").
    property var collapsed: ({})

    readonly property var rows: root.buildRows()

    function toggle(section) {
        const next = Object.assign({}, root.collapsed);
        next[section] = !root.isCollapsed(section);
        root.collapsed = next;
    }

    function isCollapsed(section) {
        return root.collapsed[section] === true;
    }

    function nameOf(container) {
        const names = KvLists.listOf(container.names);
        return names.length > 0 ? names.join(", ") : String(container.id).substring(0, 12);
    }

    function isContainer(row) {
        return row !== null && row !== undefined && row.kind === "container";
    }

    // "0.0.0.0:8080->80/tcp" -> { host: "8080", target: "80/tcp", url };
    // sem porta no host (so' exposta) -> host "" e sem url. So' TCP vira link:
    // o navegador nao fala UDP.
    function parsePort(text) {
        const arrow = text.indexOf("->");
        if (arrow < 0) return { host: "", target: text.trim(), url: "" };
        const left = text.substring(0, arrow);
        const target = text.substring(arrow + 2).trim();
        const host = left.substring(left.lastIndexOf(":") + 1).trim();
        const tcp = target.indexOf("/udp") < 0;
        return { host: host, target: target, url: host !== "" && tcp ? "http://localhost:" + host : "" };
    }

    function portsOf(container) {
        const seen = {};
        const out = [];
        for (const text of KvLists.listOf(container.ports)) {
            for (const piece of String(text).split(",")) {
                const port = root.parsePort(piece);
                const key = port.host + ">" + port.target;
                if (piece.trim() === "" || seen[key]) continue;
                seen[key] = true;
                out.push(port);
            }
        }
        return out;
    }

    function matches(text) {
        const needle = root.filter.trim().toLowerCase();
        return needle === "" || text.toLowerCase().indexOf(needle) >= 0;
    }

    function containerRow(container) {
        // `target`: o alvo das acoes (o primeiro nome, senao o id), o mesmo
        // do ContainerController.selectedTarget.
        const names = KvLists.listOf(container.names);
        return { kind: "container", key: "k|" + container.id, id: String(container.id),
                 target: names.length > 0 ? String(names[0]) : String(container.id),
                 name: root.nameOf(container), image: container.image || "", state: ContainerStates.stateOf(container),
                 // O ultimo trecho da imagem ("mongo:7"): cabe na linha
                 // estreita; o nome inteiro fica no detalhe.
                 imageShort: String(container.image || "").substring(String(container.image || "").lastIndexOf("/") + 1),
                 status: container.status || "", created: container.created || "",
                 ports: root.portsOf(container) };
    }

    // Secao VAZIA nao aparece: "EM EXECUCAO · 0" ao lado de "nenhum container"
    // so' repetia o vazio (achado na tela real, 2026-10-03).
    function section(out, key, title, items) {
        if (items.length === 0) return;
        out.push({ kind: "section", key: key, title: title, count: items.length,
                   expanded: !root.isCollapsed(key) });
        if (!root.isCollapsed(key)) for (const item of items) out.push(item);
    }

    // UMA lista de containers, por NOME (achado na tela real, 2026-10-03):
    // separar "em execucao" de "parados" fazia a linha PULAR de secao ao
    // parar, e sob o mouse parado aparecia OUTRO container com o "Iniciar"
    // embaixo do cursor — o segundo clique agia no container errado. O
    // estado esta' no ponto e nos botoes; a linha nao muda de lugar.
    function buildRows() {
        const containers = [];
        for (const container of KvLists.listOf(root.containers)) {
            const row = root.containerRow(container);
            if (!root.matches(row.name + " " + row.image + " " + row.id)) continue;
            containers.push(row);
        }
        containers.sort((a, b) => a.name.localeCompare(b.name));
        // Imagem EM USO: algum container (de qualquer estado) roda dela.
        const used = {};
        for (const container of KvLists.listOf(root.containers)) used[container.image] = true;
        const images = [];
        for (const image of KvLists.listOf(root.images)) {
            const reference = image.repository + (image.tag && image.tag !== "<none>" ? ":" + image.tag : "");
            if (!root.matches(reference + " " + image.id)) continue;
            images.push({ kind: "image", key: "i|" + image.id + "|" + reference, id: String(image.id),
                          name: reference, size: image.size, created: image.created || "",
                          inUse: used[reference] === true || used[image.id] === true });
        }
        const out = [];
        root.section(out, "containers", qsTr("CONTAINERS"), containers);
        root.section(out, "images", qsTr("IMAGENS"), images);
        return out;
    }
}
