import QtQuick
import KineinVectis

// As linhas da janela de Containers (2026-10-03): uma secao de containers em
// ordem estavel por nome e a de imagens (vazia nao aparece), contagem,
// secao fechada some com os itens e a contagem fica, filtro por nome, imagem
// ou id, portas com link so' quando ha' porta no host e e' TCP, porta repetida
// (IPv4 e IPv6) uma vez so', e nome ausente vira o id curto.
Item {
    id: root

    ContainerRows {
        id: model

        containers: [
            { id: "aaa111bbb222ccc", names: ["web"], image: "nginx:1.27", state: "running",
              status: "Up 2 hours", ports: ["0.0.0.0:8080->80/tcp, [::]:8080->80/tcp", "5353/udp"] },
            { id: "ddd444", names: ["db"], image: "postgres:16", state: "exited", status: "Exited (0)", ports: [] },
            { id: "eee555fff666ggg", names: [], image: "redis", state: "paused", status: "Paused", ports: [] }
        ]
        images: [{ id: "i1", repository: "docker.io/library/nginx", tag: "1.27", size: "187MB", created: "" },
                 { id: "i2", repository: "<none>", tag: "<none>", size: "1MB", created: "" }]
    }

    function check(condition, message) {
        if (!condition) {
            console.error("FALHOU: " + message);
            return 1;
        }
        return 0;
    }

    Component.onCompleted: {
        let failures = 0;
        const kinds = rows => rows.map(r => r.kind === "section" ? r.key + ":" + r.count : r.name).join(",");
        // UMA lista, por nome (estavel: parar nao muda a linha de lugar).
        failures += check(kinds(model.rows) === "containers:3,db,eee555fff666,web,images:2,docker.io/library/nginx:1.27,<none>",
                          "secoes: " + kinds(model.rows));
        const web = model.rows[3];
        failures += check(web.state === "running" && web.ports.length === 2, "web: " + JSON.stringify(web.ports));
        failures += check(web.imageShort === "nginx:1.27" && model.rows[1].imageShort === "postgres:16", "imagem curta");
        failures += check(web.ports[0].host === "8080" && web.ports[0].target === "80/tcp"
                          && web.ports[0].url === "http://localhost:8080", "porta TCP com link");
        failures += check(web.ports[1].host === "" && web.ports[1].url === "", "porta so' exposta (udp) sem link");
        failures += check(model.rows[2].state === "paused" && model.rows[1].state === "stopped", "estados");

        model.toggle("containers");
        failures += check(kinds(model.rows) === "containers:3,images:2,docker.io/library/nginx:1.27,<none>",
                          "secao fechada: " + kinds(model.rows));
        model.toggle("containers");

        // Secao vazia nao aparece (achado na tela real).
        model.filter = "POSTGRES";
        failures += check(kinds(model.rows) === "containers:1,db", "filtro: " + kinds(model.rows));
        model.filter = "aaa111";
        failures += check(kinds(model.rows) === "containers:1,web", "filtro por id");
        model.filter = "nada-assim";
        failures += check(model.rows.length === 0, "nada: nenhuma secao");
        model.filter = "";

        failures += check(model.parsePort("127.0.0.1:5432->5432/tcp").url === "http://localhost:5432", "endereco local");
        // Imagem EM USO: um container roda dela (pela referencia).
        model.images = [{ id: "i9", repository: "nginx", tag: "1.27", size: "1MB" },
                        { id: "i8", repository: "alpine", tag: "3", size: "1MB" }];
        const imageRows = model.rows.filter(r => r.kind === "image");
        failures += check(imageRows[0].inUse === true && imageRows[1].inUse === false, "em uso");
        failures += check(model.parsePort("80/tcp").host === "", "so' exposta");

        if (failures !== 0) console.error("FALHAS " + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
