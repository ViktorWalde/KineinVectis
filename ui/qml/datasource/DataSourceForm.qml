pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O formulario do perfil. Burro: recebe o rascunho, devolve edicoes.
//
// NAO HA' CAMPO DE SENHA AQUI, e e' deliberado. O que se escolhe neste
// formulario e' DE ONDE a senha vem, nunca qual e' ela — o perfil e' o que o
// core persiste, e senha nao vai para o disco (`docs/seguranca/40`). O campo
// de senha, quando aparece, vive no veredito do teste e some com a sessao.
Item {
    id: root

    property var draft: null

    signal fieldEdited(string field, var value)

    readonly property string secretSource:
        root.draft ? (root.draft.secretSource || "automatic") : "automatic"

    // `SQLite` e' um ARQUIVO: nao tem servidor, porta, usuario nem senha.
    // Mostrar esses campos vazios seria pedir ao autor que preenchesse o que
    // nao existe — que e' como a maioria das IDEs trata SQLite.
    readonly property bool arquivo:
        root.draft ? root.draft.engine === "sqlite" : false

    // O MongoDB tem servidor e porta como o Postgres, mas NAO exige usuario —
    // um servidor local sem autenticacao e' o caso comum de desenvolvimento. E
    // ele ganha um campo que os outros dois nao tem: o tamanho da amostra,
    // porque nele a estrutura e' INFERIDA e o custo dessa inferencia e' uma
    // escolha do autor.
    //
    // VEM DE FORA, e nao de uma comparacao local com o motor do rascunho: o
    // controller ja' responde essa pergunta para escolher a VISAO da
    // estrutura, e duas copias da mesma derivacao divergem em silencio — o
    // gate de duplicacao pegou a segunda no mesmo dia em que ela nasceu.
    property bool mongo: false

    implicitHeight: coluna.implicitHeight

    Column {
        id: coluna

        anchors.left: parent.left
        anchors.right: parent.right
        spacing: Theme.spacingSmall

        Text {
            width: parent.width
            text: qsTr("Motor")
            color: Theme.textMuted
            font.pixelSize: 10
        }

        Row {
            width: parent.width
            spacing: Theme.spacingXSmall

            KvToggleChip {
                labelText: qsTr("PostgreSQL / TimescaleDB")
                active: !root.arquivo
                onToggled: root.fieldEdited("engine", "postgres")
            }

            KvToggleChip {
                labelText: qsTr("SQLite (arquivo)")
                active: root.arquivo
                onToggled: root.fieldEdited("engine", "sqlite")
            }

            KvToggleChip {
                labelText: qsTr("MongoDB")
                active: root.mongo
                onToggled: root.fieldEdited("engine", "mongo")
            }
        }

        DataSourceField {
            width: parent.width
            label: qsTr("Nome")
            placeholder: qsTr("como a IDE vai chamar esta fonte")
            value: root.draft ? root.draft.name : ""
            onEdited: text => root.fieldEdited("name", text)
        }

        DataSourceField {
            width: parent.width
            visible: root.arquivo
            label: qsTr("Arquivo .db")
            placeholder: qsTr("caminho do banco SQLite neste projeto")
            value: root.draft ? root.draft.database : ""
            onEdited: text => root.fieldEdited("database", text)
        }

        DataSourceField {
            width: parent.width
            visible: !root.arquivo
            label: qsTr("Host ou diretório de socket")
            placeholder: qsTr("/var/run/postgresql, ou db.exemplo.com")
            value: root.draft ? root.draft.host : ""
            onEdited: text => root.fieldEdited("host", text)
        }

        Row {
            width: parent.width
            visible: !root.arquivo
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
            font.pixelSize: 9
        }

        DataSourceField {
            width: parent.width
            visible: !root.arquivo
            label: root.mongo ? qsTr("Usuário (vazio = sem autenticação)") : qsTr("Usuário")
            placeholder: qsTr("o papel que conecta")
            value: root.draft ? root.draft.user : ""
            onEdited: text => root.fieldEdited("user", text)
        }

        Text {
            width: parent.width
            visible: !root.arquivo
            text: qsTr("De onde vem a senha")
            color: Theme.textMuted
            font.pixelSize: 10
        }

        Row {
            width: parent.width
            visible: !root.arquivo
            spacing: Theme.spacingXSmall

            KvToggleChip {
                labelText: qsTr("Automático")
                active: root.secretSource === "automatic"
                onToggled: root.fieldEdited("secretSource", "automatic")
            }

            KvToggleChip {
                labelText: qsTr("Ambiente")
                active: root.secretSource === "environment"
                onToggled: root.fieldEdited("secretSource", "environment")
            }

            KvToggleChip {
                labelText: qsTr("Perguntar")
                active: root.secretSource === "prompt"
                onToggled: root.fieldEdited("secretSource", "prompt")
            }
        }

        Text {
            width: parent.width
            visible: !root.arquivo
            wrapMode: Text.WordWrap
            color: Theme.textMuted
            font.pixelSize: 10
            text: {
                if (root.secretSource === "environment") {
                    return qsTr("A IDE lê a variável na hora de conectar. "
                                + "Só o NOME dela é salvo.");
                }
                if (root.secretSource === "prompt") {
                    return qsTr("A senha é pedida a cada sessão e vive só em "
                                + "memória.");
                }
                return qsTr("Não manda senha: o servidor decide. Cobre socket "
                            + "unix com peer, trust local e o ~/.pgpass.");
            }
        }

        DataSourceField {
            width: parent.width
            visible: !root.arquivo && root.secretSource === "environment"
            label: qsTr("Variável de ambiente")
            placeholder: "PGPASSWORD"
            value: root.draft ? (root.draft.secretVariable || "") : ""
            onEdited: text => root.fieldEdited("secretVariable", text)
        }
    }
}
