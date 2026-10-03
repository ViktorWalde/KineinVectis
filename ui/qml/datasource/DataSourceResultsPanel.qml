pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A secao de DADOS da janela do Banco (2026-10-03, pedido do autor: o
// resultado aparece na PROPRIA janela que o icone do Banco abre — "senao o
// contexto fica espalhado demais em paineis"). Mostra o que o console do
// editor (Ctrl+Enter) ou a arvore (clique duplo numa tabela) pediu. Estreita
// como a janela: a grade rola nos dois sentidos.
//
// A escrita (INSERT, UPDATE, DROP...) nao roda sem confirmacao: o core recusa
// a primeira vez com WRITE_CONFIRMATION_REQUIRED, e o botao daqui repete a
// mesma instrucao, confirmada.
Item {
    id: root

    property var controller: null

    signal closeRequested()

    // Quanto a secao precisa para a grade caber sem rolar.
    readonly property real naturalWidth: grid.naturalWidth

    readonly property var lastQuery: root.controller ? root.controller.lastQuery : null
    readonly property bool querying: root.controller !== null && root.controller.querying
    readonly property bool mustConfirm: root.controller !== null && root.controller.writeConfirmationRequired
                                        && root.lastQuery !== null

    Item {
        id: statusRow

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        height: 24

        KvIcon {
            id: statusIcon

            anchors.left: parent.left
            anchors.verticalCenter: parent.verticalCenter
            size: 14
            name: "table"
            active: root.querying
        }

        Text {
            anchors.left: statusIcon.right
            anchors.leftMargin: Theme.spacingXSmall
            anchors.right: closeButton.left
            anchors.rightMargin: Theme.spacingXSmall
            anchors.verticalCenter: parent.verticalCenter
            text: root.lastQuery === null ? qsTr("Dados")
                  : root.lastQuery.name + "  ·  " + (root.querying ? qsTr("executando…")
                                                      : (root.controller ? root.controller.queryStatus : ""))
            color: root.mustConfirm ? Theme.warningSoft : Theme.textSecondary
            font.pixelSize: Theme.fontSizeCaption
            elide: Text.ElideRight
        }

        KvIconButton {
            id: closeButton

            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            compact: true
            iconName: "close"
            iconSize: 12
            tooltip: qsTr("Fechar os dados")
            onClicked: root.closeRequested()
        }
    }

    KvButton {
        id: confirmButton

        anchors.top: statusRow.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        visible: root.mustConfirm
        height: visible ? implicitHeight : 0
        compact: true
        danger: true
        text: qsTr("Esta instrução escreve — executar mesmo assim")
        enabled: !root.querying
        onClicked: root.controller.runOn(root.lastQuery.name, root.lastQuery.sql, true)
    }

    // A instrucao que gerou os dados, numa linha: o que se esta' vendo.
    Text {
        id: sqlLine

        anchors.top: confirmButton.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        visible: root.lastQuery !== null
        // Altura da FONTE, nao do proprio texto: com elide, `implicitHeight`
        // depende de `height` e o QML acusava binding loop (log da UI).
        height: visible ? Theme.fontSizeMicro + 2 * Theme.spacingXSmall : 0
        verticalAlignment: Text.AlignVCenter
        text: root.lastQuery !== null ? root.lastQuery.sql.replace(/\s+/g, " ") : ""
        color: Theme.textMuted
        font.family: Theme.monoFont
        font.pixelSize: Theme.fontSizeMicro
        elide: Text.ElideRight
    }

    KvDataGrid {
        id: grid

        clip: true
        anchors.top: sqlLine.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        maxHeight: Math.max(40, root.height - statusRow.height - confirmButton.height - sqlLine.height)
        // O core manda o nome da coluna; a grade quer { key, label }.
        columns: root.controller ? root.controller.queryColumns.map(name => ({ key: name, label: name })) : []
        rows: root.controller ? root.controller.queryRows : []
        emptyText: root.querying ? "" : qsTr("Sem linhas.")
    }
}
