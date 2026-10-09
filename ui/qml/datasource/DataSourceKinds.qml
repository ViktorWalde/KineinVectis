pragma Singleton
import QtQuick

// QUAL MOTOR e QUE TIPO DE OBJETO — dono unico (2026-10-03). A janela do
// Banco, a arvore, o dialogo da conexao e o console perguntam daqui: a mesma
// derivacao espalhada em cinco arquivos e' como uma tela passa a chamar de
// "visao" o que outra chama de "tabela" sem ninguem perceber (catraca de
// duplicacao QML).
QtObject {
    function isPostgres(engine) {
        return engine === "postgres";
    }

    function isMongo(engine) {
        return engine === "mongo";
    }

    function isOdbc(engine) {
        return engine === "odbc";
    }

    function isSqlite(engine) {
        return engine === "sqlite";
    }

    function engineIcon(engine) {
        return isSqlite(engine) ? "file" : (isMongo(engine) ? "documents" : "database");
    }

    function engineColor(engine) {
        return isSqlite(engine) ? Theme.databaseSqlite : (isMongo(engine) ? Theme.databaseMongo : (isOdbc(engine) ? Theme.databaseOdbc : Theme.databasePostgres));
    }

    function engineName(engine) {
        return isOdbc(engine) ? "Outro banco (ODBC)" : (isSqlite(engine) ? "SQLite" : (isMongo(engine) ? "MongoDB" : (isPostgres(engine) ? "PostgreSQL" : qsTr("Motor indisponível"))));
    }

    function engineShort(engine) {
        return isOdbc(engine) ? "ODBC" : (isSqlite(engine) ? "SQLite" : (isMongo(engine) ? "Mongo" : "PG"));
    }

    function statementName(engine, operation) {
        if (isMongo(engine)) return operation === "select" ? "find" : (operation === "insert" ? "insertOne" : "updateMany");
        return operation.toUpperCase();
    }

    function policyLabel(profile) {
        if (!profile) return "";
        const labels = [];
        if (profile.production === true) labels.push(qsTr("PRODUÇÃO"));
        if (profile.readOnly === true) labels.push(qsTr("somente leitura"));
        return labels.join(" · ");
    }

    // Os padroes de cada motor (2026-10-04): trocar o motor no formulario
    // trazia o PostgreSQL junto (`/var/run/postgresql`, 5432) para o MongoDB.
    function defaultsFor(engine) {
        if (isMongo(engine)) return { host: "localhost", port: 27017, database: "test" };
        if (isSqlite(engine) || isOdbc(engine)) return { host: "", port: 0, database: "" };
        return isPostgres(engine) ? { host: "/var/run/postgresql", port: 5432, database: "postgres" } : {};
    }

    // Descritores decidem campos/disponibilidade; esta camada so' apresenta.
    function providerFor(providers, engine) {
        return providers.find(provider => provider.engine === engine) || null;
    }

    function hasProfileFeature(provider, feature) {
        return provider !== null && provider.profileFeatures.indexOf(feature) >= 0;
    }

    function providerOptions(providers) {
        return providers.map(provider => ({ value: provider.engine,
            label: isOdbc(provider.engine) ? qsTr("Outro (ODBC)") : engineName(provider.engine),
            icon: isOdbc(provider.engine) ? "" : engineIcon(provider.engine),
            tooltip: isPostgres(provider.engine) ? qsTr("PostgreSQL e TimescaleDB")
                : (isOdbc(provider.engine) ? qsTr("Driver instalado e DSN registrado no unixODBC") : "") }));
    }

    // No rascunho `draft` (ja' com o motor novo), troca pelos padroes do
    // motor novo SO' os campos que ainda estao no padrao do anterior (ou
    // vazios): o que a pessoa digitou fica.
    function adoptDefaults(draft, fromEngine, toEngine) {
        if (isOdbc(toEngine) || isOdbc(fromEngine)) {
            Object.assign(draft, defaultsFor(toEngine), { tls: "disable", caFile: "", user: "", secretSource: "automatic", secretVariable: "" });
            return;
        }
        const before = defaultsFor(fromEngine);
        const after = defaultsFor(toEngine);
        for (const key of ["host", "port", "database"]) {
            if (after[key] !== undefined && (draft[key] === before[key] || draft[key] === "" || draft[key] === undefined)) draft[key] = after[key];
        }
    }

    // QUAL ADAPTADOR roda o perfil (9a.3, arquitetura/39 §6.2.2). O descritor
    // diz o que o motor oferece; sem `installations` (core antes do 0.167.0),
    // so' o interno. Tipo que esta versao nao conhece nao vira opcao.
    function installationOf(profile) {
        return profile && profile.installation ? profile.installation.kind : "builtin";
    }

    function installationOptions(provider) {
        const labels = { builtin: qsTr("Interno"), ide: qsTr("O da IDE") };
        const kinds = provider && provider.installations ? provider.installations : ["builtin"];
        return kinds.filter(kind => labels[kind] !== undefined).map(kind => ({ value: kind, label: labels[kind] }));
    }

    // Ausente = interno; o da IDE e' `{ kind: "ide" }`. O que o motor nao
    // oferece cai para o interno: trocar SQLite por PostgreSQL com o da IDE
    // faria o `save` responder INVALID_PARAMS.
    function chooseInstallation(profile, provider, kind) {
        const offered = installationOptions(provider).some(option => option.value === kind);
        if (offered && kind !== "builtin") profile.installation = { kind: kind };
        else delete profile.installation;
    }

    function emptyProfile() {
        return Object.assign({
            engine: "postgres",
            name: "",
            user: "",
            secretSource: "automatic",
            secretVariable: "",
            production: false,
            readOnly: false,
            tls: "disable",
            caFile: ""
        }, defaultsFor("postgres"));
    }


    function cloneProfile(source) {
        const profile = {
            engine: source.engine || "postgres",
            name: source.name,
            host: source.host,
            port: source.port,
            database: source.database,
            user: source.user,
            secretSource: source.secretSource || "automatic",
            secretVariable: source.secretVariable || "",
            production: source.production === true,
            readOnly: source.readOnly === true,
            sampleSize: source.sampleSize,
            tls: source.tls || "disable",
            caFile: source.caFile || ""
        };
        // O adaptador escolhido (`0.167.0`, passo 9a.2) atravessa a edicao: sem
        // isto, salvar pela tela voltaria o perfil ao adaptador interno.
        if (source.installation) profile.installation = { kind: source.installation.kind };
        return profile;
    }


    // Traducao do resultado: Mongo conta documentos; SQL conta linhas.
    function querySummary(outcome, engine) {
        if (outcome.affected !== undefined && outcome.affected !== null) {
            return (isMongo(engine) ? qsTr("%1 documento(s) afetado(s) em %2 ms")
                                   : qsTr("%1 linha(s) afetada(s) em %2 ms"))
                .arg(outcome.affected).arg(outcome.elapsedMs);
        }
        return qsTr("%1 linha(s)%2 em %3 ms").arg(outcome.rowCount)
            .arg(outcome.truncated ? qsTr(" — teto atingido") : "").arg(outcome.elapsedMs);
    }

    // CARREGAR MAIS (passo 14 da 0.3.9, roadmaps/59 §5.4.1): o teto seguinte
    // de uma LEITURA que o teto cortou, o dobro do anterior ate' o maximo do
    // contrato (arquitetura/03: `maxRows` 500 por padrao, 10.000 no maximo).
    // Cortada, ela devolveu exatamente o teto, entao `rowCount` E' o teto.
    // 0 = nada a carregar: escrita, previa, resultado inteiro ou o maximo.
    readonly property int maxQueryRows: 10000
    function nextRowCeiling(query) {
        if (query === null || query === undefined || query.truncated !== true || query.access !== "read"
                || query.preview === true || query.wrote === true) return 0;
        const next = Math.min(maxQueryRows, 2 * query.rowCount);
        return next > query.rowCount ? next : 0;
    }

    // EXPORTAR (passo 14b, roadmaps/59 §5.4.1): onde o CSV nasce, dentro do
    // projeto. O nome da conexao e' saneado (o resto vira `-`, sem ponto no
    // comeco); a hora local fecha o nome, e o core recusa sobrescrever.
    readonly property string exportDirectory: "exportacoes"
    function exportPath(connection, date) {
        const safe = String(connection).replace(/[^A-Za-z0-9._-]+/g, "-").replace(/^[-.]+|-+$/g, "");
        const two = value => String(value).padStart(2, "0");
        const stamp = String(date.getFullYear()) + two(date.getMonth() + 1) + two(date.getDate()) + "-"
            + two(date.getHours()) + two(date.getMinutes()) + two(date.getSeconds());
        return exportDirectory + "/" + (safe === "" ? "dados" : safe) + "-" + stamp + ".csv";
    }

    function isTimeseries(kind) {
        return kind === "timeseries";
    }

    // Como a tela desenha uma tabela (SQL) ou uma colecao (Mongo).
    function tableKind(kind) {
        return kind === "view" ? "view" : "table";
    }

    function collectionKind(kind) {
        return isTimeseries(kind) ? "timeseries" : (kind === "view" ? "view" : "collection");
    }

    function kindIcon(kind) {
        return ({ schema: "schema", table: "table", view: "eye", collection: "documents",
                  timeseries: "observability", read: "refresh", failed: "warning",
                  unavailable: "warning" })[kind] || "";
    }

    // Por que o perfil esta' preservado sem uso (0.165.0): o motivo tipado,
    // curto para caber na coluna da arvore; nunca conteudo do arquivo.
    function unavailableReason(reason, engine) {
        if (reason === "unknownProvider") return qsTr("%1 sem suporte").arg(engine);
        return ({ unsupportedInstallation: qsTr("instalação externa"),
                  unsupportedOptions: qsTr("opções mais novas"),
                  invalidOptions: qsTr("opções inválidas") })[reason] || qsTr("indisponível");
    }

    // Tem linhas para mostrar na secao de dados (clique duplo na arvore)?
    function isConnection(kind) { return kind === "connection"; }
    // Perfil preservado que esta versao nao usa (0.165.0, D1a.4).
    function isUnavailable(kind) { return kind === "unavailable"; }

    function hasData(kind) {
        return ["table", "view", "collection", "timeseries"].indexOf(kind) >= 0;
    }
}
