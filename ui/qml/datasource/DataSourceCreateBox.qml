pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// "Novo banco" (0.124.0): um arquivo SQLite no projeto, um PostgreSQL ou
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
    signal closeRequested()

    property string kind: "sqlite"
    property string name: ""
    property string port: ""

    readonly property string preview: {
        if (kind === "sqlite") return qsTr("cria data/%1.sqlite no projeto e salva o perfil").arg(name || "<nome>");
        if (kind === "database") return "CREATE DATABASE \"" + (name || "<nome>") + "\"  —  " + qsTr("no perfil %1").arg(serverProfileName || "?");
        const motor = containerEngine || "podman";
        const imagem = kind === "mongo" ? "docker.io/library/mongo:7" : "docker.io/library/postgres:16";
        const interna = kind === "mongo" ? 27017 : 5432;
        const p = port !== "" ? port : String(interna);
        const auth = kind === "mongo" ? "" : " -e POSTGRES_HOST_AUTH_METHOD=trust";
        return motor + " run -d --name kinein-" + (name || "<nome>") + " -p 127.0.0.1:" + p + ":" + interna + auth + " " + imagem;
    }

    implicitHeight: coluna.implicitHeight + 2 * Theme.spacingSmall

    Rectangle {
        anchors.fill: parent
        radius: Theme.radius
        color: Theme.background2
        border.width: 1
        border.color: Theme.borderSoft
    }

    Column {
        id: coluna

        x: Theme.spacingSmall
        y: Theme.spacingSmall
        width: parent.width - 2 * Theme.spacingSmall
        spacing: Theme.spacingSmall

        Row {
            width: parent.width

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: qsTr("Novo banco")
                color: Theme.textPrimary
                font.pixelSize: 12
                font.bold: true
            }

            Item { width: parent.width - x - fechar.width; height: 1 }

            KvIconButton {
                id: fechar

                compact: true
                iconName: "close"
                tooltip: qsTr("Fechar")
                onClicked: root.closeRequested()
            }
        }

        // Flow, nao Row: com ~400 px a coluna nao cabia os quatro chips e o
        // "MongoDB em container" ficava cortado (o autor viu, 2026-09-19).
        Flow {
            width: parent.width
            spacing: Theme.spacingXSmall

            KvToggleChip {
                labelText: qsTr("SQLite (arquivo)")
                active: root.kind === "sqlite"
                onToggled: root.kind = "sqlite"
            }

            KvToggleChip {
                labelText: qsTr("PostgreSQL em container")
                active: root.kind === "postgres"
                enabled: root.canServe
                onToggled: root.kind = "postgres"
            }

            KvToggleChip {
                labelText: qsTr("MongoDB em container")
                active: root.kind === "mongo"
                enabled: root.canServe
                onToggled: root.kind = "mongo"
            }

            KvToggleChip {
                labelText: qsTr("Banco no servidor")
                active: root.kind === "database"
                enabled: root.serverProfileNamed
                onToggled: root.kind = "database"
            }
        }

        Text {
            width: parent.width
            visible: !root.canServe
            wrapMode: Text.WordWrap
            text: qsTr("Sem Podman/Docker no PATH não há como subir um servidor em container.")
            color: Theme.textMuted
            font.pixelSize: 10
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
            wrapMode: Text.WrapAnywhere
            text: root.preview
            color: Theme.textSecondary
            font.family: Theme.monoFont
            font.pixelSize: 10
        }

        Text {
            width: parent.width
            visible: root.kind === "postgres"
            wrapMode: Text.WordWrap
            text: qsTr("Autenticação `trust` SÓ no loopback: a IDE não guarda senha, e a porta não sai desta máquina.")
            color: Theme.textMuted
            font.pixelSize: 10
        }

        KvVerdict {
            width: parent.width
            busy: root.creating
            busyText: root.command !== "" ? qsTr("rodando: %1").arg(root.command) : qsTr("criando…")
            ok: root.ok
            message: root.message
        }

        KvButton {
            text: root.kind === "sqlite" ? qsTr("Criar arquivo")
                  : root.kind === "database" ? qsTr("Criar banco (escreve no servidor)")
                  : qsTr("Subir servidor (baixa a imagem)")
            primary: true
            compact: true
            enabled: !root.creating && root.name.trim() !== ""
            onClicked: {
                if (root.kind === "sqlite") root.createSqliteRequested(root.name, "");
                else if (root.kind === "database") root.createDatabaseRequested(root.name);
                else root.createServerRequested(root.kind, root.name, Number(root.port));
            }
        }
    }
}
