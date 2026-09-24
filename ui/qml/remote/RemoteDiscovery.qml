pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// "Usar o SSH que ja' funciona" (0.132.0, fatia R0.5): a entrada que NAO pede
// formulario. Os aliases concretos do `~/.ssh/config` aparecem com a origem;
// escolher um deles cria o alvo sem redigitar usuario, porta nem chave, e o
// resumo abaixo diz o que o OpenSSH FARIA — medido por `ssh -G`, nao adivinhado.
//
// Componente burro: recebe por property, pede por signal. Nao monta linha de
// ssh e nao mostra texto de ProxyCommand — o core nao manda, por desenho.
Item {
    id: root

    property string discovery: "idle"
    // Vem do controller: ele e' o dono de "esta' carregando".
    property bool discovering: false
    property var aliases: []
    property var aliasSources: []
    property string resolving: ""
    property var resolved: null

    signal refreshRequested()
    signal aliasChosen(string name)

    implicitHeight: coluna.implicitHeight

    readonly property bool vazio: root.discovery === "ready" && root.aliases.length === 0

    // O resumo em UMA linha: e' isso que dispensa o formulario.
    readonly property string resumo: {
        if (root.resolving !== "") {
            return qsTr("perguntando ao ssh sobre %1…").arg(root.resolving);
        }
        if (!root.resolved) {
            return "";
        }
        const alvo = root.resolved.hostName || root.resolved.host;
        const porta = root.resolved.port ? ":" + root.resolved.port : "";
        const quem = root.resolved.user ? root.resolved.user + "@" : "";
        return qsTr("o ssh vai em %1%2%3").arg(quem).arg(alvo).arg(porta);
    }

    Column {
        id: coluna

        anchors.left: parent.left
        anchors.right: parent.right
        spacing: 4

        Row {
            width: parent.width
            spacing: Theme.spacingSmall

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: qsTr("Usar o SSH que já funciona")
                color: Theme.textPrimary
                font.pixelSize: 11
            }

            KvButton {
                compact: true
                text: root.discovery === "ready" || root.discovery === "failed"
                      ? qsTr("Procurar de novo") : qsTr("Procurar")
                enabled: !root.discovering
                onClicked: root.refreshRequested()
            }
        }

        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            text: {
                if (root.discovering) {
                    return qsTr("lendo ~/.ssh/config…");
                }
                if (root.discovery === "failed") {
                    return qsTr("não deu para ler a configuração do ssh; "
                                + "preencha o alvo abaixo.");
                }
                if (root.vazio) {
                    return qsTr("nenhum alias concreto em ~/.ssh/config — "
                                + "configure o servidor no formulário abaixo.");
                }
                // Dizer QUAIS arquivos foram lidos importa quando ha' `Include`:
                // e' a diferenca entre "a IDE nao achou" e "o alias esta' num
                // arquivo que ninguem incluiu".
                const lidos = root.aliasSources.length > 1
                            ? qsTr(" Lidos: %1.").arg(root.aliasSources.join(", "))
                            : "";
                return qsTr("aliases do seu ~/.ssh/config; escolher um cria o alvo "
                            + "sem repetir usuário, porta nem chave.") + lidos;
            }
            color: Theme.textMuted
            font.pixelSize: 10
        }

        Repeater {
            model: root.aliases

            delegate: Rectangle {
                id: linha

                required property var modelData

                width: coluna.width
                height: 30
                radius: Theme.radius
                color: area.containsMouse ? Theme.surface2 : "transparent"

                Column {
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.leftMargin: Theme.spacingSmall
                    anchors.rightMargin: Theme.spacingSmall

                    Text {
                        width: parent.width
                        text: linha.modelData.name
                        color: Theme.textPrimary
                        font.family: Theme.monoFont
                        font.pixelSize: 11
                        elide: Text.ElideRight
                    }

                    // A ORIGEM importa: um alias de config.d/ nao e' a mesma
                    // coisa que um do arquivo principal quando algo destoa.
                    Text {
                        width: parent.width
                        text: linha.modelData.source
                        color: Theme.textMuted
                        font.family: Theme.monoFont
                        font.pixelSize: 9
                        elide: Text.ElideMiddle
                    }
                }

                MouseArea {
                    id: area

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.aliasChosen(linha.modelData.name)
                }
            }
        }

        Text {
            width: parent.width
            visible: root.resumo !== ""
            wrapMode: Text.WordWrap
            text: root.resumo
            color: Theme.textPrimary
            font.family: Theme.monoFont
            font.pixelSize: 10
        }

        Text {
            width: parent.width
            visible: root.resolved !== null && root.resolving === ""
            wrapMode: Text.WordWrap
            text: {
                const partes = [];
                const chaves = root.resolved && root.resolved.identities
                             ? root.resolved.identities.length : 0;
                // O `ssh -G` lista as chaves PADRAO mesmo quando nenhuma
                // existe no disco (cinco, numa maquina sem ~/.ssh/config).
                // Dizer "candidata" sugeria que ha' chave configurada.
                if (chaves > 0) {
                    partes.push(qsTr("%n chave(s) que o ssh tentaria", "", chaves));
                }
                if (root.resolved && root.resolved.proxyJump) {
                    partes.push(qsTr("salto por %1").arg(root.resolved.proxyJump));
                }
                if (root.resolved && root.resolved.proxyCommand) {
                    partes.push(qsTr("passa por um ProxyCommand do seu config"));
                }
                return partes.join(" · ");
            }
            color: Theme.textMuted
            font.pixelSize: 9
        }
    }
}
