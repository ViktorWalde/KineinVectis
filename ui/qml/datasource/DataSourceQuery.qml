pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Executar o que o autor escreve (0.121.0): o editor da instrucao, a
// confirmacao quando ela ESCREVE, e o resultado como grade de texto.
//
// Burro: recebe por property, pede por signal. A confirmacao aparece por
// `writeConfirmationRequired` — o codigo do core —, nunca lendo a mensagem.
// Ctrl+Enter executa; e' o gesto de todo cliente de banco.
Item {
    id: root

    property string sql: ""
    // Vem do controller (o dono da derivacao "e' documento?"), como no painel.
    property bool documentEngine: false
    property bool querying: false
    property bool writeConfirmationRequired: false
    property var columns: []
    property var rows: []
    property string status: ""
    property bool canRun: false

    signal sqlEdited(string text)
    signal runRequested(bool confirmWrite)

    implicitHeight: coluna.implicitHeight

    readonly property string placeholder: root.documentEngine
        ? qsTr("colecao {\"campo\": \"valor\"}   — find com limite; so' leitura")
        : qsTr("SELECT ... — Ctrl+Enter executa; escrita pede confirmacao")

    Column {
        id: coluna

        anchors.left: parent.left
        anchors.right: parent.right
        spacing: Theme.spacingXSmall

        Text {
            width: parent.width
            text: qsTr("Consulta")
            color: Theme.textMuted
            font.pixelSize: 10
        }

        Rectangle {
            width: parent.width
            height: 72
            radius: Theme.radius
            color: Theme.background0
            border.width: 1
            border.color: entrada.activeFocus ? Theme.accent : Theme.borderSoft

            Flickable {
                anchors.fill: parent
                anchors.margins: 4
                clip: true
                contentWidth: width
                contentHeight: entrada.contentHeight
                boundsBehavior: Flickable.StopAtBounds

                TextEdit {
                    id: entrada

                    width: parent.width
                    text: root.sql
                    color: Theme.textPrimary
                    selectionColor: Theme.accentDim
                    selectedTextColor: Theme.textPrimary
                    font.family: Theme.monoFont
                    font.pixelSize: 11
                    wrapMode: TextEdit.Wrap
                    selectByMouse: true
                    onTextChanged: if (text !== root.sql) root.sqlEdited(text)
                    Keys.onPressed: function(event) {
                        if ((event.key === Qt.Key_Return || event.key === Qt.Key_Enter)
                                && (event.modifiers & Qt.ControlModifier)) {
                            root.runRequested(false);
                            event.accepted = true;
                        }
                    }

                    Text {
                        visible: entrada.text === ""
                        text: root.placeholder
                        color: Theme.textMuted
                        font.pixelSize: 10
                    }
                }
            }
        }

        Row {
            width: parent.width
            spacing: Theme.spacingSmall

            KvButton {
                text: root.querying ? qsTr("Executando...") : qsTr("Executar")
                primary: true
                compact: true
                enabled: root.canRun && !root.querying && root.sql.trim() !== ""
                onClicked: root.runRequested(false)
            }

            // A escrita so' roda depois deste clique: o core recusou a
            // primeira vez com WRITE_CONFIRMATION_REQUIRED.
            KvButton {
                visible: root.writeConfirmationRequired
                text: qsTr("Esta instrução ESCREVE — executar mesmo assim")
                compact: true
                enabled: !root.querying
                onClicked: root.runRequested(true)
            }
        }

        Text {
            width: parent.width
            visible: root.status !== ""
            wrapMode: Text.WordWrap
            text: root.status
            color: root.writeConfirmationRequired ? Theme.textSecondary : Theme.textMuted
            font.family: Theme.monoFont
            font.pixelSize: 10
        }

        // A grade comum (F8): cabecalho fixo, largura por conteudo, `null`
        // em italico, rolagem quando nao cabe. As colunas vem do core como
        // nomes; as linhas como arrays na ordem delas.
        KvDataGrid {
            width: parent.width
            visible: root.columns.length > 0
            columns: root.columns.map(name => ({ key: name, label: name }))
            rows: root.rows
            maxHeight: 220
        }
    }
}
