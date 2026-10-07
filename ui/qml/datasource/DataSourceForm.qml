pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O formulario do perfil. Burro: recebe o rascunho, devolve edicoes.
//
// NAO HA' CAMPO DE SENHA AQUI, e e' deliberado. O que se escolhe neste
// formulario e' DE ONDE a senha vem, nunca qual e' ela — o perfil e' o que o
// core persiste, e senha nao vai para o disco (`DocsPublic/seguranca/40`). O campo
// de senha, quando aparece, vive no veredito do teste e some com a sessao.
Item {
    id: root

    property var draft: null
    property var providers: []
    readonly property var provider: root.draft
        ? DataSourceKinds.providerFor(root.providers, root.draft.engine) : null
    readonly property bool credentials: DataSourceKinds.hasProfileFeature(root.provider, "credentials")

    property var odbcSources: []
    property bool odbcLoading: false
    property string odbcMessage: ""
    signal odbcRefreshRequested()
    signal fieldEdited(string field, var value)

    readonly property string secretSource:
        root.draft ? (root.draft.secretSource || "automatic") : "automatic"

    // `SQLite` e' um ARQUIVO: nao tem servidor, porta, usuario nem senha.
    // Mostrar esses campos vazios seria pedir ao autor que preenchesse o que
    // nao existe — que e' como a maioria das IDEs trata SQLite.
    readonly property bool fileConnection: root.provider !== null && root.provider.connectionKind === "file"

    // O MongoDB tem servidor e porta como o Postgres, mas NAO exige usuario —
    // um servidor local sem autenticacao e' o caso comum de desenvolvimento. E
    // ele ganha um campo que os outros dois nao tem: o tamanho da amostra,
    // porque nele a estrutura e' INFERIDA e o custo dessa inferencia e' uma
    // escolha do autor.
    //
    // Campos vêm do descritor do core; não inferir suporte pelo nome do motor.
    readonly property bool mongo: DataSourceKinds.hasProfileFeature(root.provider, "sampling")
    readonly property bool verifiedTls: DataSourceKinds.hasProfileFeature(root.provider, "verifiedTls")
    readonly property bool odbc: root.provider !== null && root.provider.connectionKind === "dsn"
    readonly property bool server: root.provider !== null && root.provider.connectionKind === "server"
    readonly property string tls: root.draft ? (root.draft.tls || "disable") : "disable"

    implicitHeight: coluna.implicitHeight
    enabled: root.providers.length > 0

    Column {
        id: coluna

        anchors.left: parent.left
        anchors.right: parent.right
        spacing: Theme.spacingSmall

        Text {
            width: parent.width
            text: qsTr("Motor")
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeCaption
        }

        Text {
            width: parent.width
            visible: root.provider === null
            text: root.providers.length === 0 ? qsTr("Aguardando os motores disponíveis…")
                : qsTr("Este motor está indisponível. Escolha um motor disponível para editar.")
            wrapMode: Text.WordWrap
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeCaption
        }

        // Padrao novo (2026-10-04): o seletor segmentado no lugar dos chips
        // em fonte mono; "SQLite (arquivo)" virou "SQLite" (pedido do autor).
        KvSegmentedControl {
            width: parent.width
            current: root.draft ? root.draft.engine : "postgres"
            options: DataSourceKinds.providerOptions(root.providers)
            onSelected: value => root.fieldEdited("engine", value)
        }

        DataSourceField {
            width: parent.width
            label: qsTr("Nome")
            placeholder: qsTr("como a IDE vai chamar esta fonte")
            value: root.draft ? root.draft.name : ""
            onEdited: text => root.fieldEdited("name", text)
        }

        Text {
            text: qsTr("Ambiente")
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeCaption
        }

        KvSegmentedControl {
            width: parent.width
            current: root.draft && root.draft.production ? "production" : "development"
            options: [ { value: "development", label: qsTr("Desenvolvimento") },
                       { value: "production", label: qsTr("Produção"), icon: "warning" } ]
            onSelected: value => root.fieldEdited("production", value === "production")
        }

        KvSegmentedControl {
            width: parent.width
            current: root.draft && root.draft.readOnly ? "read" : "write"
            options: [ { value: "write", label: qsTr("Permitir escrita") },
                       { value: "read", label: qsTr("Somente leitura"), icon: "eye" } ]
            onSelected: value => root.fieldEdited("readOnly", value === "read")
        }

        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            text: qsTr("Produção pede confirmação para toda escrita. Somente leitura recusa escrita e comandos desconhecidos. Salve antes de testar estas opções.")
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeCaption
        }

        DataSourceDsnPicker {
            width: parent.width
            visible: root.odbc
            sources: root.odbcSources
            current: root.draft ? root.draft.database : ""
            loading: root.odbcLoading
            message: root.odbcMessage
            onSelected: dsn => root.fieldEdited("database", dsn)
            onRefreshRequested: root.odbcRefreshRequested()
        }

        DataSourceField {
            width: parent.width
            visible: root.fileConnection
            label: qsTr("Arquivo .db")
            placeholder: qsTr("caminho do banco SQLite neste projeto")
            value: root.draft ? root.draft.database : ""
            onEdited: text => root.fieldEdited("database", text)
        }

        DataSourceField {
            width: parent.width
            visible: root.server
            label: root.mongo ? qsTr("Host") : qsTr("Host ou diretório de socket")
            placeholder: root.mongo ? qsTr("localhost, ou db.exemplo.com") : qsTr("/var/run/postgresql, ou db.exemplo.com")
            value: root.draft ? root.draft.host : ""
            onEdited: text => root.fieldEdited("host", text)
        }

        Row {
            width: parent.width
            visible: root.server
            spacing: Theme.spacingSmall

            DataSourceField {
                width: (parent.width - Theme.spacingSmall) / 3
                label: qsTr("Porta")
                numeric: true
                value: root.draft ? String(root.draft.port) : ""
                onEdited: text => root.fieldEdited("port", parseInt(text, 10) || 0)
            }

            DataSourceField {
                width: (parent.width - Theme.spacingSmall) * 2 / 3
                label: qsTr("Banco")
                value: root.draft ? root.draft.database : ""
                onEdited: text => root.fieldEdited("database", text)
            }
        }

        // O CUSTO DA AMOSTRA E' ESCOLHA DO AUTOR, e a tela conta qual e'.
        // O padrao NAO e' os 1.000 do Compass: o `$sample` do MongoDB varre a
        // colecao inteira quando N nao e' menor que 5% dela, e 1.000 dispara
        // essa varredura em toda colecao com menos de 20.000 documentos.
        DataSourceField {
            width: parent.width
            visible: root.mongo
            label: qsTr("Documentos na amostra")
            numeric: true
            placeholder: "200"
            value: root.draft && root.draft.sampleSize !== undefined
                   ? String(root.draft.sampleSize) : ""
            onEdited: text => root.fieldEdited("sampleSize", parseInt(text, 10) || 0)
        }

        Text {
            width: parent.width
            visible: root.mongo
            wrapMode: Text.WordWrap
            text: qsTr("A estrutura de uma coleção é inferida da amostra, não declarada — a leitura diz quantos documentos leu e se precisou varrer a coleção inteira.")
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeMicro
        }

        DataSourceField {
            width: parent.width
            visible: root.credentials
            label: root.mongo ? qsTr("Usuário (vazio = sem autenticação)") : qsTr("Usuário")
            placeholder: qsTr("o papel que conecta")
            value: root.draft ? root.draft.user : ""
            onEdited: text => root.fieldEdited("user", text)
        }

        Text {
            width: parent.width
            visible: root.credentials
            text: qsTr("De onde vem a senha")
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeCaption
        }

        KvSegmentedControl {
            width: parent.width
            visible: root.credentials
            current: root.secretSource
            options: [
                { value: "automatic", label: qsTr("Automático") },
                { value: "environment", label: qsTr("Variável de ambiente") },
                { value: "prompt", label: qsTr("Perguntar") }
            ]
            onSelected: value => root.fieldEdited("secretSource", value)
        }

        Text {
            width: parent.width
            visible: root.credentials
            wrapMode: Text.WordWrap
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeCaption
            text: {
                if (root.secretSource === "environment") {
                    return qsTr("A IDE lê a variável na hora de conectar. "
                                + "Só o NOME dela é salvo.");
                }
                if (root.secretSource === "prompt") {
                    return qsTr("A senha é pedida a cada sessão e vive só em "
                                + "memória.");
                }
                return root.mongo || root.odbc
                    ? qsTr("Não manda senha: o servidor decide (um servidor sem autenticação, por exemplo).")
                    : qsTr("Não manda senha: o servidor decide. Cobre socket "
                           + "unix com peer, trust local e o ~/.pgpass.");
            }
        }

        DataSourceField {
            width: parent.width
            visible: root.credentials && root.secretSource === "environment"
            label: qsTr("Variável de ambiente")
            placeholder: root.odbc ? "DB_PASSWORD" : "PGPASSWORD"
            value: root.draft ? (root.draft.secretVariable || "") : ""
            onEdited: text => root.fieldEdited("secretVariable", text)
        }

        // TLS (0.121.0): so' o PostgreSQL. `require` e' o verify-full do
        // libpq — cadeia E nome do host conferidos; nao existe "cifra sem
        // conferir" aqui.
        KvSegmentedControl {
            width: parent.width
            visible: root.verifiedTls
            current: root.tls === "require" ? "require" : "disable"
            options: [
                { value: "disable", label: qsTr("Sem TLS") },
                { value: "require", label: qsTr("TLS verificado"),
                  tooltip: qsTr("verify-full: confere a cadeia e o nome do host") }
            ]
            onSelected: value => root.fieldEdited("tls", value)
        }

        DataSourceField {
            width: parent.width
            visible: root.verifiedTls && root.tls === "require"
            label: qsTr("Certificado (PEM) em que confiar — vazio = raízes públicas")
            placeholder: "/etc/ssl/certs/meu-postgres.pem"
            value: root.draft ? (root.draft.caFile || "") : ""
            onEdited: text => root.fieldEdited("caFile", text)
        }
    }
}
