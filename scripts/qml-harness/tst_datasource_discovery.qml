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

    property var pedidos: []
    property var salvos: []
    property var consultas: []

    Item {
        visible: false

        DataSourceController {
            id: fontes

            onListRequested: root.pedidos.push("list")
            onSaveRequested: profile => root.salvos.push(profile)
            onQueryRequested: (name, password, sql, confirmWrite) =>
                root.consultas.push({ name: name, sql: sql, confirmWrite: confirmWrite })

            discovery.onDiscoverRequested: root.pedidos.push("discover")
            discovery.onCreateSqliteRequested: (name, path) => root.pedidos.push("sqlite:" + name)
            discovery.onCreateServerRequested: (engine, name, port) =>
                root.pedidos.push("server:" + engine + ":" + name + ":" + port)
        }
    }

    Component.onCompleted: {
        let failures = 0;
        const d = fontes.discovery;

        // abrir descobre
        fontes.open();
        if (root.pedidos.indexOf("discover") < 0 || !d.discovering) failures += 1;
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
        const antes = root.pedidos.length;
        d.createSqlite("notas", "");
        if (root.pedidos[root.pedidos.length - 1] !== "sqlite:notas" || !d.creating) failures += 8;
        d.handleCreateResolved({ name: "notas", engine: "sqlite", host: "", port: 0,
                                 database: "/w/data/notas.sqlite", user: "", secretSource: "automatic" }, "", "");
        if (d.creating || !d.createOk || fontes.selectedName !== "notas" || fontes.draft.engine !== "sqlite"
            || root.pedidos.indexOf("list", antes) < 0) failures += 16;

        // servidor em container: job + comando; o perfil chega no evento
        d.createServer("postgres", "dev", 0);
        if (root.pedidos[root.pedidos.length - 1] !== "server:postgres:dev:5432") failures += 32;
        d.handleCreateResolved({}, "job_9", "podman run -d --name kinein-dev ...");
        if (!d.creating || d.createCommand.indexOf("podman run") !== 0) failures += 64;
        d.handleCreated(true, { name: "dev", engine: "postgres", host: "127.0.0.1", port: 5432,
                                database: "postgres", user: "postgres", secretSource: "automatic" }, "no ar");
        if (d.creating || !d.createOk || fontes.selectedName !== "dev" || fontes.draft.port !== 5432) failures += 128;

        // banco DENTRO do servidor: CREATE DATABASE confirmado, depois o clone salvo
        fontes.createDatabaseOnServer("loja");
        const q = root.consultas[root.consultas.length - 1];
        if (!q || q.sql !== 'CREATE DATABASE "loja"' || !q.confirmWrite || q.name !== "dev") failures += 256;
        fontes.handleQueried({ success: true, columns: [], rows: [], affected: 0, elapsedMs: 3 });
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

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
