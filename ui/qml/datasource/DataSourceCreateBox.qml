pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// "Criar banco" (0.124.0; "Novo banco" ate' 2026-10-04): um arquivo SQLite no projeto, um PostgreSQL ou
// MongoDB em container no loopback (o comando aparece ANTES e DEPOIS —
// baixa imagem, nunca em silencio), ou um banco dentro do PostgreSQL do
// perfil em edicao (`CREATE DATABASE`, pela escrita confirmada). Burro:
// recebe estado, pede por sinal.
Item {
    id: root

    property bool canServe: false
    property string containerEngine: ""
    property bool creating: false
    property string command: ""
    property string message: ""
    property bool ok: false
    // O perfil em edicao e' um PostgreSQL salvo? Habilita "banco no servidor".
    property bool serverProfileNamed: false
    property string serverProfileName: ""

    signal createSqliteRequested(string name, string path)
    signal createServerRequested(string engine, string name, int port)
    signal createDatabaseRequested(string name)

    property string kind: "sqlite"
    property string name: ""
    property string port: ""

    // O que acontece: frase para o SQLite, o COMANDO (em caixa de codigo)
    // para o que roda num servidor.
    readonly property string sentence: kind === "sqlite"
        ? qsTr("Cria data/%1.sqlite no projeto e salva a conexão.").arg(name || "<nome>")
        : (kind === "database" ? qsTr("Cria o banco dentro do PostgreSQL da conexão %1.").arg(serverProfileName || "?")
                               : qsTr("Sobe um servidor novo em contêiner, só no loopback desta máquina. O comando:"))
    readonly property string preview: {
        if (kind === "sqlite") return "";
        if (kind === "database") return "CREATE DATABASE \"" + (name || "<nome>") + "\"";
        const motor = containerEngine || "podman";
        const imagem = kind === "mongo" ? "docker.io/library/mongo:7" : "docker.io/library/postgres:16";
        const interna = kind === "mongo" ? 27017 : 5432;
        const p = port !== "" ? port : String(interna);
        const auth = kind === "mongo" ? "" : " -e POSTGRES_HOST_AUTH_METHOD=trust";
        return motor + " run -d --name kinein-" + (name || "<nome>") + " -p 127.0.0.1:" + p + ":" + interna + auth + " " + imagem;
    }

    implicitHeight: coluna.implicitHeight + 2 * Theme.spacingSmall

    Column {
        id: coluna

        x: Theme.spacingSmall
        y: Theme.spacingSmall
        width: parent.width - 2 * Theme.spacingSmall
        spacing: Theme.spacingSmall

        // Padrao novo (2026-10-04): o seletor segmentado; o titulo e o x
        // repetidos sairam (o dialogo ja' diz "Criar banco").
        KvSegmentedControl {
            width: parent.width
            current: root.kind
            options: [
                { value: "sqlite", label: qsTr("SQLite"), icon: DataSourceKinds.engineIcon("sqlite"),
                  tooltip: qsTr("Um arquivo de banco dentro do projeto") },
                { value: "postgres", label: qsTr("PostgreSQL"), icon: DataSourceKinds.engineIcon("postgres"),
                  enabled: root.canServe, tooltip: qsTr("Um servidor novo em contêiner (Podman/Docker)") },
                { value: "mongo", label: qsTr("MongoDB"), icon: DataSourceKinds.engineIcon("mongo"),
                  enabled: root.canServe, tooltip: qsTr("Um servidor novo em contêiner (Podman/Docker)") },
                { value: "database", label: qsTr("No servidor"), icon: "schema",
                  enabled: root.serverProfileNamed,
                  tooltip: qsTr("Um banco dentro do PostgreSQL da conexão escolhida") }
            ]
            onSelected: value => root.kind = value
        }

        Text {
            width: parent.width
            visible: !root.canServe
            wrapMode: Text.WordWrap
            text: qsTr("Sem Podman/Docker no PATH não há como subir um servidor em container.")
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeCaption
        }

        Row {
            width: parent.width
            spacing: Theme.spacingSmall

            DataSourceField {
                width: root.kind === "postgres" || root.kind === "mongo" ? parent.width - 110 - parent.spacing : parent.width
                label: qsTr("Nome")
                placeholder: qsTr("letras, dígitos, - e _")
                value: root.name
                onEdited: text => root.name = text
            }

            DataSourceField {
                width: 110
                visible: root.kind === "postgres" || root.kind === "mongo"
                label: qsTr("Porta (loopback)")
                placeholder: root.kind === "mongo" ? "27017" : "5432"
                numeric: true
                value: root.port
                onEdited: text => root.port = text
            }
        }

        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            text: root.sentence
            color: Theme.textSecondary
            font.pixelSize: Theme.fontSizeCaption
        }

        Rectangle {
            width: parent.width
            visible: root.preview !== ""
            height: previewText.implicitHeight + 2 * Theme.spacingSmall
            radius: Theme.radius
            color: Theme.background0
            border.width: 1
            border.color: Theme.borderSoft

            Text {
                id: previewText

                anchors.fill: parent
                anchors.margins: Theme.spacingSmall
                wrapMode: Text.WrapAnywhere
                text: "$ " + root.preview
                color: Theme.textPrimary
                font.family: Theme.monoFont
                font.pixelSize: Theme.fontSizeCaption
            }
        }

        Text {
            width: parent.width
            visible: root.kind === "postgres"
            wrapMode: Text.WordWrap
            text: qsTr("Autenticação `trust` SÓ no loopback: a IDE não guarda senha, e a porta não sai desta máquina.")
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeCaption
        }

        KvVerdict {
            width: parent.width
            busy: root.creating
            busyText: root.command !== "" ? qsTr("rodando: %1").arg(root.command) : qsTr("criando…")
            ok: root.ok
            message: root.message
        }

        KvButton {
            activeFocusOnTab: true
            width: parent.width
            text: root.kind === "sqlite" ? qsTr("Criar o arquivo")
                  : root.kind === "database" ? qsTr("Criar o banco no servidor")
                  : qsTr("Subir o servidor (baixa a imagem)")
            primary: true
            enabled: !root.creating && root.name.trim() !== ""
                     && (root.kind === "sqlite" || (root.kind === "database" ? root.serverProfileNamed : root.canServe))
            onClicked: {
                if (root.kind === "sqlite") root.createSqliteRequested(root.name, "");
                else if (root.kind === "database") root.createDatabaseRequested(root.name);
                else root.createServerRequested(root.kind, root.name, Number(root.port));
            }
        }
    }
}
