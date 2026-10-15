import QtQuick
import "../../ui/qml/datasource"

// A descoberta e a criacao (0.124.0): o candidato adotado vai ao formulario
// SEM ser salvo; o SQLite criado vem salvo e selecionado; o servidor em
// container e' um job com o comando visivel e o perfil chega no evento;
// `CREATE DATABASE` no PostgreSQL do perfil clona e salva com o banco novo;
// a falha do create nao vira erro do painel.
Item {
    id: root
    width: 100; height: 100

    property var requests: []
    property var salvos: []
    property var consultas: []

    Item {
        visible: false

        DataSourceController {
            id: fontes

            onListRequested: root.requests.push("list")
            onSaveRequested: profile => root.salvos.push(profile)
            onQueryRequested: (name, password, sql, confirmWrite) =>
                root.consultas.push({ name: name, sql: sql, confirmWrite: confirmWrite })

            discovery.onDiscoverRequested: root.requests.push("discover")
            discovery.onCreateSqliteRequested: (name, path) => root.requests.push("sqlite:" + name)
            discovery.onCreateServerRequested: (engine, name, port) =>
                root.requests.push("server:" + engine + ":" + name + ":" + port)
            discovery.onDestroyRequested: (name, data) => root.requests.push("destroy:" + name + ":" + data)
        }
    }

    Component.onCompleted: {
        let failures = 0;
        const d = fontes.discovery;

        // abrir descobre
        fontes.open();
        if (root.requests.indexOf("discover") < 0 || !d.discovering) failures += 1;
        d.handleDiscovered([
            { kind: "container", label: "container kinein-pg", detail: "Up", running: true,
              profile: { name: "kinein-pg", engine: "postgres", host: "127.0.0.1", port: 5433,
                         database: "postgres", user: "postgres", secretSource: "automatic" } },
            { kind: "file", label: "data/app.sqlite", detail: "8 KB", running: true,
              profile: { name: "app", engine: "sqlite", host: "", port: 0,
                         database: "/w/data/app.sqlite", user: "", secretSource: "automatic" } }
        ], "podman", "");
        if (d.discovering || d.candidates.length !== 2 || !d.canServe || d.defaultPort("mongo") !== 27017) failures += 2;

        // adotar: formulario preenchido, nada salvo, nada selecionado
        d.adopt(0);
        if (fontes.draft.name !== "kinein-pg" || fontes.draft.port !== 5433 || fontes.selectedName !== ""
            || root.salvos.length !== 0) failures += 4;

        // criar SQLite: vem pronto e salvo -> relista e seleciona
        const antes = root.requests.length;
        d.createSqlite("notas", "");
        if (root.requests[root.requests.length - 1] !== "sqlite:notas" || !d.creating) failures += 8;
        d.handleCreateResolved({ name: "notas", engine: "sqlite", host: "", port: 0,
                                 database: "/w/data/notas.sqlite", user: "", secretSource: "automatic" }, "", "");
        if (d.creating || !d.createOk || fontes.selectedName !== "notas" || fontes.draft.engine !== "sqlite"
            || root.requests.indexOf("list", antes) < 0) failures += 16;

        // servidor em container: job + comando; o perfil chega no evento
        d.createServer("postgres", "dev", 0);
        if (root.requests[root.requests.length - 1] !== "server:postgres:dev:5432") failures += 32;
        d.handleCreateResolved({}, "job_9", "podman run -d --name kinein-dev ...");
        if (!d.creating || d.createCommand.indexOf("podman run") !== 0) failures += 64;
        d.handleCreated(true, { name: "dev", engine: "postgres", host: "127.0.0.1", port: 5432,
                                database: "postgres", user: "postgres", secretSource: "automatic" }, "no ar");
        if (d.creating || !d.createOk || fontes.selectedName !== "dev" || fontes.draft.port !== 5432) failures += 128;

        // banco DENTRO do servidor: CREATE DATABASE confirmado, depois o clone salvo
        fontes.workspaceRoot = "/w";
        fontes.handleList([{ name: "dev", engine: "postgres", host: "127.0.0.1", port: 5432,
            database: "postgres", user: "postgres", secretSource: "automatic" }]);
        fontes.select("dev");
        fontes.createDatabaseOnServer("loja");
        const q = root.consultas[root.consultas.length - 1];
        if (!q || q.sql !== 'CREATE DATABASE "loja"' || q.confirmWrite || q.name !== "dev") failures += 256;
        fontes.handleQueried({ name: fontes.lastQuery.name, clientContext: fontes.lastQuery.clientContext, success: true, columns: [], rows: [], affected: 0, elapsedMs: 3 });
        const salvo = root.salvos[root.salvos.length - 1];
        if (!salvo || salvo.name !== "dev-loja" || salvo.database !== "loja" || salvo.host !== "127.0.0.1") failures += 512;
        fontes.createDatabaseOnServer("nome ruim");
        if (fontes.errorText === "") failures += 1024;

        // falha do create fica no dono certo
        fontes.errorText = "";
        d.createSqlite("x", "");
        d.handleFailed("datasource.create", "ja' existe");
        fontes.handleFailed("datasource.create", "ja' existe", "INVALID_PARAMS");
        if (d.creating || d.createOk || d.createMessage !== "ja' existe" || fontes.errorText !== "") failures += 2048;
        d.handleFailed("datasource.discover", "sem workspace");
        if (d.discovering || d.hint !== "sem workspace") failures += 4096;

        // Remover (0.129.0): imediato relista e volta ao formulario vazio; o
        // job espera o evento; a nota fica visivel; a falha no dono certo.
        fontes.handleList([{ name: "a", engine: "sqlite", host: "", port: 0, database: "/w/a.sqlite", user: "" },
                           { name: "b", engine: "postgres", host: "h", port: 5432, database: "postgres", user: "u" }]);
        fontes.select("a");
        d.destroyProfile("a", true);
        if (root.requests[root.requests.length - 1] !== "destroy:a:true" || !d.destroying) failures += 8192;
        d.handleDestroyResolved([{ name: "b", engine: "postgres", host: "h", port: 5432, database: "postgres", user: "u" }], true, "", "", "", d.pendingDestroy.clientContext);
        if (d.destroying || !d.destroyOk || fontes.profiles.length !== 1 || fontes.selectedName !== "") failures += 16384;
        d.destroyProfile("b", true);
        d.handleDestroyResolved([], false, "job_3", "podman rm -f kinein-b", "", d.pendingDestroy.clientContext);
        if (!d.destroying || d.destroyMessage.indexOf("podman rm") < 0) failures += 32768;
        d.handleDestroyed(true, "b removido", [], d.pendingDestroy.clientContext);
        if (d.destroying || !d.destroyOk || fontes.profiles.length !== 0) failures += 65536;
        fontes.handleList([{ name: "x", host: "h", port: 5432, database: "d", user: "u" }]);
        d.destroyProfile("x", false);
        d.handleDestroyResolved([], true, "", "", "o servidor fica", d.pendingDestroy.clientContext);
        if (d.destroyNote !== "o servidor fica") failures += 131072;
        fontes.handleList([{ name: "y", host: "h", port: 5432, database: "d", user: "u" }]);
        d.destroyProfile("y", true);
        d.handleFailed("datasource.destroy", "nao ha perfil", "INVALID_PARAMS", d.pendingDestroy);
        if (d.destroying || d.destroyOk || d.destroyMessage !== "nao ha perfil") failures += 262144;

        // 16b.2 (roadmaps/59 §2.5.2): sem motor que responda, o porque; a
        // porta livre que o core mediu vai no pedido quando o campo fica vazio.
        d.handleDiscovered([], "", "", "o Docker nao respondeu: usermod", { postgres: 5434, mongo: 27017 });
        if (d.canServe || d.containerHint.indexOf("usermod") < 0 || d.defaultPort("postgres") !== 5434)
            failures += 524288;
        d.handleDiscovered([], "podman", "", "", { postgres: 5434 });
        d.createServer("postgres", "livre", 0);
        if (!d.canServe || d.containerHint !== "" || d.defaultPort("mongo") !== 27017
            || root.requests[root.requests.length - 1] !== "server:postgres:livre:5434") failures += 1048576;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
