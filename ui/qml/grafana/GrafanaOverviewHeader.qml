pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O CABECALHO DO USO DIARIO (§5.2 da especificacao): nome, estado numa frase,
// a acao primaria e o caminho de volta para a configuracao.
//
// O painel antigo nao tinha cabecalho: tinha formulario. Endereco, politica de
// token e tres botoes de mesmo peso ficavam na tela para sempre, e quem so'
// queria abrir um dashboard atravessava tudo isso todo dia. Aqui a
// configuracao vira um gesto — `configurar…` — e o resto do painel fala do
// Grafana, nao do cadastro dele.
Item {
    id: root

    property var controller: null
    property var acao: ({ "kind": "", "label": "", "hint": "" })
    // O autor pediu para ver a configuracao, mesmo com tudo em ordem.
    property bool setupPinned: false

    signal primaryActivated()
    signal setupToggled()

    readonly property string frase: root.controller
            ? root.controller.statusPhrase : ""

    implicitHeight: linha.implicitHeight
                    + (estado.visible ? estado.implicitHeight + Theme.spacingXSmall : 0)
    height: implicitHeight

    Row {
        id: linha

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        spacing: Theme.spacingSmall

        Text {
            anchors.verticalCenter: parent.verticalCenter
            width: parent.width - acoes.width - Theme.spacingSmall
            elide: Text.ElideRight
            text: qsTr("Observabilidade")
            color: Theme.textPrimary
            font.pixelSize: 13
            font.bold: true
        }

        Row {
            id: acoes

            spacing: Theme.spacingSmall

            KvBarButton {
                objectName: "grafanaPrimaria"

                // Quando o gesto primario E' o campo do token, quem o desenha
                // e' o `GrafanaTokenPrompt`: um botao aqui seria o segundo.
                visible: root.acao.kind !== "provideToken"
                labelText: root.acao.label
                enabled: root.acao.kind !== "waiting"
                onActivated: root.primaryActivated()
            }

            KvBarButton {
                // O caminho de volta existe SEMPRE, ate' com tudo em ordem: o
                // endereco muda, o Grafana muda de maquina, e esconder a
                // configuracao atras de "esquecer a instancia" seria obrigar a
                // desfazer para reconfigurar.
                labelText: root.setupPinned ? qsTr("ocultar ajustes")
                                            : qsTr("configurar…")
                onActivated: root.setupToggled()
            }
        }
    }

    Text {
        id: estado

        anchors.top: linha.bottom
        anchors.topMargin: Theme.spacingXSmall
        anchors.left: parent.left
        anchors.right: parent.right
        wrapMode: Text.WordWrap
        visible: root.frase !== ""
        text: root.frase
        color: Theme.textMuted
        font.pixelSize: 10
    }
}
