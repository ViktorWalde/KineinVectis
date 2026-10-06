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
    // A consulta falhou: a mensagem do motor ganha lugar proprio, quebrada em
    // linhas e em vermelho. Na linha de status (uma linha so') ela quebrava o
    // layout: a mensagem do SQLite traz `\n` e o elide nao corta texto com
    // quebra (2026-10-03, achado na tela real).
    readonly property bool failed: root.lastQuery !== null && root.lastQuery.failed === true && !root.querying

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
                  : root.lastQuery.name + "  ·  " + (DataSourceKinds.policyLabel(root.lastQuery.expectedContext.profile) ? DataSourceKinds.policyLabel(root.lastQuery.expectedContext.profile) + " · " : "") + (root.querying ? qsTr("executando…")
                     : (root.failed ? qsTr("falhou")
                        // A escrita espera o painel do impacto; fechado sem
                        // executar, diz que NADA rodou (e nao "confirme").
                        : (root.mustConfirm ? (root.controller.impact.open ? qsTr("aguardando a confirmação…")
                                                                           : qsTr("cancelada — nada foi executado"))
                           : (root.controller ? root.controller.queryStatus.replace(/\s+/g, " ") : ""))))
            textFormat: Text.PlainText
            maximumLineCount: 1
            color: root.failed ? Theme.errorSoft : (root.mustConfirm ? Theme.warningSoft : Theme.textSecondary)
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

    // A escrita nao confirmada abre o painel do impacto (SqlImpactDialog):
    // aqui fica so' o status, em ambar, enquanto ele pergunta.
    // A instrucao que gerou os dados, numa linha: o que se esta' vendo.
    Text {
        id: sqlLine

        anchors.top: statusRow.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        visible: root.lastQuery !== null
        // Altura da FONTE, nao do proprio texto: com elide, `implicitHeight`
        // depende de `height` e o QML acusava binding loop (log da UI).
        height: visible ? Theme.fontSizeMicro + 2 * Theme.spacingXSmall : 0
        verticalAlignment: Text.AlignVCenter
        text: root.lastQuery !== null ? root.lastQuery.sql.replace(/\s+/g, " ") : ""
        textFormat: Text.PlainText
        color: Theme.textMuted
        font.family: Theme.monoFont
        font.pixelSize: Theme.fontSizeMicro
        elide: Text.ElideRight
    }

    // A mensagem do motor, inteira, no lugar da grade.
    Text {
        anchors.top: sqlLine.bottom
        anchors.topMargin: Theme.spacingXSmall
        anchors.left: parent.left
        anchors.right: parent.right
        visible: root.failed
        wrapMode: Text.Wrap
        text: root.controller ? root.controller.queryStatus : ""
        textFormat: Text.PlainText
        color: Theme.errorSoft
        font.family: Theme.monoFont
        font.pixelSize: Theme.fontSizeCaption
    }

    KvDataGrid {
        id: grid

        visible: !root.failed
        clip: true
        anchors.top: sqlLine.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        maxHeight: Math.max(40, root.height - statusRow.height - sqlLine.height)
        // O core manda o nome da coluna; a grade quer { key, label }.
        columns: root.controller ? root.controller.queryColumns.map(name => ({ key: name, label: name })) : []
        rows: root.controller ? root.controller.queryRows : []
        // Uma escrita nao devolve linhas — ela as afeta (o status diz quantas).
        emptyText: root.querying || root.mustConfirm ? ""
                   : (root.lastQuery !== null && root.lastQuery.wrote === true ? qsTr("Instrução executada.") : qsTr("Sem linhas."))
    }
}
