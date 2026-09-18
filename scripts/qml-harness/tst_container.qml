// Containers: o que a UI mostra do motor e dos containers, e o que ela pede
// (roadmaps/28 §0: Docker e Podman NATIVOS; priorizado em 2026-09-12).
//
// Por que existe: a traducao do ContainerController — motor achado/ausente/
// nao-responde, a acao oferecida por ESTADO (parar so' ao que roda, remover
// so' ao que parou), o job que falhou virando motivo na tela e o que deu certo
// so' atualizando a lista — quebra sem compilador que reclame. E' a classe do
// `cdbStale`, de novo.
import QtQuick
import "../../ui/qml/container"

Item {
    id: root

    property int statusPedidos: 0
    property int listaPedidos: 0
    property bool ultimoAll: false
    property int imagensPedidos: 0
    property var acoes: []
    property var aberturas: []
    property var composes: []

    ContainerController {
        id: controller

        onStatusRequested: root.statusPedidos += 1
        onListRequested: function(all) { root.listaPedidos += 1; root.ultimoAll = all; }
        onImagesRequested: root.imagensPedidos += 1
        onActionRequested: function(id, action) { root.acoes.push(id + ":" + action); }
        onOpenRequested: function(id, mode) { root.aberturas.push(id + ":" + mode); }
        onComposeRequested: function(action, file) { root.composes.push(action + ":" + file); }
    }

    Component.onCompleted: {
        let failures = 0;

        // Abrir PERGUNTA as tres coisas, e a lista vem com os parados (padrao).
        controller.open();
        if (!controller.panelVisible) failures += 1;
        if (root.statusPedidos !== 1 || root.listaPedidos !== 1 || root.imagensPedidos !== 1) failures += 2;
        if (!root.ultimoAll) failures += 4;
        if (!controller.statusBusy || !controller.listBusy) failures += 8;

        // O motor desta maquina em 2026-09-12: podman rootless atras do shim.
        controller.handleStatus({ engine: "podman", version: "5.8.4", emulated: true, rootless: true,
                                  socket: "/run/user/1000/podman/podman.sock", reachable: true,
                                  compose: "/usr/bin/podman compose" });
        if (!controller.engineFound || !controller.reachable) failures += 16;
        if (controller.engineLabel.indexOf("Podman 5.8.4") !== 0) failures += 32;
        if (controller.engineLabel.indexOf("shim") < 0) failures += 64;
        if (controller.statusBusy) failures += 128;

        // Sem motor: nao e' "nao responde", e' outro estado.
        controller.handleStatus({ emulated: false, reachable: false, hint: "instale o podman" });
        if (controller.engineFound || controller.engineLabel !== "") failures += 256;

        // A lista: a acao oferecida depende do ESTADO declarado pelo motor.
        const rodando = { id: "27b17e15d40c", names: ["postgres-dev", "pg"], image: "docker.io/library/postgres:16-alpine",
                          state: "running", status: "Up 3 minutes", ports: ["0.0.0.0:5433->5432/tcp"] };
        const parado = { id: "fdd8de359c22", names: ["timescaledb"], image: "docker.io/timescale/timescaledb:latest-pg16",
                         state: "exited", status: "Exited (0) 2 weeks ago", ports: [] };
        controller.handleContainers([rodando, parado], "podman", "", "");
        if (!controller.containerFound || controller.listBusy) failures += 512;
        if (!controller.isRunning(rodando) || controller.isRunning(parado)) failures += 1024;
        if (controller.containerSummary(rodando)
                !== "postgres-dev, pg · docker.io/library/postgres:16-alpine · 0.0.0.0:5433->5432/tcp") failures += 2048;
        // Sem nome, o id curto; sem portas, nada de "undefined".
        if (controller.containerSummary({ id: "abcdef0123456789", image: "x" }) !== "abcdef012345 · x") failures += 4096;

        // Bytes viram GB/MB; texto pronto (Docker) passa direto.
        if (controller.formatSize(3035568151) !== "3.04 GB") failures += 8192;
        if (controller.formatSize("431MB") !== "431MB") failures += 16384;
        // F8: a grade comum recebe linhas com o tamanho ja' formatado.
        controller.handleImages([{ repository: "postgres", tag: "16", size: 452984832, created: "2026-09-01" }], "");
        const linhaImagem = controller.imageRows()[0];
        if (linhaImagem.repository !== "postgres" || linhaImagem.tag !== "16"
            || linhaImagem.size !== "453.0 MB" || linhaImagem.created !== "2026-09-01") failures += 32768;

        // Agir e' PEDIR: o controller nao roda nada. Logs/shell sao abas de
        // terminal, e a aba e' do projeto: sem workspace o pedido nao sai e o
        // motivo aparece (2026-09-13 — antes o core recusava depois do clique).
        controller.act("pg", "stop");
        const rootAntes = controller.workspaceRoot;
        controller.workspaceRoot = "";
        controller.openLogs("pg");
        if (root.aberturas.length !== 0 || controller.errorText.indexOf("abra um projeto") !== 0
                || controller.canOpenTerminals) failures += 536870912;
        controller.workspaceRoot = rootAntes === "" ? "/tmp/proj" : rootAntes;
        controller.errorText = "";
        controller.openLogs("pg");
        controller.openShell("pg");
        if (root.acoes.join(",") !== "pg:stop") failures += 65536;
        if (root.aberturas.join(",") !== "pg:logs,pg:shell") failures += 131072;

        // O job que FALHOU vira motivo na tela; o que deu certo so' relista.
        const listasAntes = root.listaPedidos;
        controller.handleFinished({ jobId: "job_1", action: "stop", target: "pg", ok: false, message: "permission denied" });
        if (controller.errorText.indexOf("stop pg falhou") !== 0 || controller.errorText.indexOf("permission denied") < 0) failures += 262144;
        if (root.listaPedidos !== listasAntes + 1) failures += 524288;
        controller.handleFinished({ jobId: "job_2", action: "start", target: "pg", ok: true, message: "pg" });
        if (controller.errorText !== "") failures += 1048576;
        if (root.listaPedidos !== listasAntes + 2) failures += 2097152;

        // Erro de OUTRO dominio nao acende o erro deste painel; o deste, sim.
        controller.handleFailed("git.status", "nada a ver");
        if (controller.errorText !== "") failures += 4194304;
        controller.handleFailed("container.list", "nenhum motor no PATH");
        if (controller.errorText !== "nenhum motor no PATH" || controller.listBusy) failures += 8388608;

        // Compose e' do projeto: os pedidos saem com o arquivo padrao (vazio).
        controller.composeUp();
        controller.composeDown();
        if (root.composes.join(",") !== "up:,down:") failures += 16777216;

        // O botao so' promete o que funciona: ferramenta DA MAQUINA + arquivo
        // DO PROJETO (o core diz qual). Cada falta tem a sua frase na linha do
        // motor (2026-09-13: "compose up" aceso sem projeto parecia defeito).
        controller.workspaceRoot = "";
        controller.handleStatus({ engine: "podman", reachable: true, emulated: true });
        if (controller.canCompose || controller.composeSummary !== "sem compose") failures += 268435456;
        controller.handleStatus({ engine: "podman", reachable: true, emulated: true, compose: "/usr/bin/podman compose" });
        if (controller.canCompose || controller.composeSummary.indexOf("abra um projeto") < 0) failures += 268435456;
        controller.workspaceRoot = "/tmp/proj";
        if (controller.canCompose || controller.composeSummary.indexOf("não tem compose.yaml") < 0) failures += 268435456;
        controller.handleStatus({ engine: "podman", reachable: true, emulated: true,
                                  compose: "/usr/bin/podman compose", composeFile: "docker-compose.yml" });
        if (!controller.canCompose
                || controller.composeSummary !== "compose: /usr/bin/podman compose · docker-compose.yml") failures += 268435456;
        controller.handleStatus({ engine: "podman", reachable: false, emulated: true,
                                  compose: "/usr/bin/podman compose", composeFile: "docker-compose.yml" });
        if (controller.canCompose) failures += 268435456;

        // "parados tambem" desligado relista SEM os parados.
        controller.setShowAll(false);
        if (root.ultimoAll !== false) failures += 33554432;

        // Trocar de workspace FECHA o painel mas NAO esquece o motor: ele e'
        // da maquina, nao do projeto.
        controller.handleStatus({ engine: "podman", reachable: true, emulated: false });
        controller.workspaceRoot = "/tmp/outro";
        if (controller.panelVisible) failures += 67108864;
        if (!controller.engineFound) failures += 134217728;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
