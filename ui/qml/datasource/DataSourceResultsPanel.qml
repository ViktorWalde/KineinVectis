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
    // O teto seguinte de uma leitura cortada (0 = nada a carregar).
    readonly property int nextCeiling: DataSourceKinds.nextRowCeiling(root.lastQuery)
    // Ordenada E cortada: a ordem vale so' para o que ja' veio (59 §5.4.1).
    // Fica na barra de baixo: no fim da linha de status, o corte a escondia.
    readonly property string sortNote: grid.sorting.order !== null && root.lastQuery !== null
                                       && root.lastQuery.truncated === true
                                       ? qsTr("Ordem só nas %1 carregadas").arg(root.lastQuery.rowCount) : ""
    // COPIAR e EXPORTAR (passo 14b): o resultado da ultima acao fica na barra
    // de baixo, no lugar da nota da ordem, ate' as linhas mudarem.
    property string actionNote: ""
    property bool actionFailed: false
    // A linha escolhida como linha de `rows`: ela segue a linha quando a
    // ordem muda (a grade fala da linha como aparece).
    property int selectedSource: -1
    readonly property bool hasRows: grid.rows.length > 0 && grid.columns.length > 0
                                    && !root.querying && !root.failed

    GridRules {
        id: delimited
    }

    Connections {
        target: root.controller ? root.controller.exports : null
        function onFinished(message, ok) { root.note(message, !ok); }
    }

    function note(message, failure) {
        root.actionNote = message;
        root.actionFailed = failure === true;
    }

    // Tudo na ORDEM DA TELA: o que se copia e' o que se ve.
    function copyRow(index) {
        Clipboard.setText(delimited.delimitedRow(grid.columns, grid.sorting.shownRows[index], "\t"));
        root.note(qsTr("Linha copiada (TSV)."));
    }

    function copyAll() {
        Clipboard.setText(delimited.delimitedText(grid.columns, grid.sorting.shownRows, ","));
        root.note(qsTr("%1 linha(s) copiada(s) em CSV.").arg(grid.rows.length));
    }

    function exportAll() {
        root.note(qsTr("Exportando…"));
        root.controller.exports.begin(root.lastQuery.name, delimited.delimitedText(grid.columns, grid.sorting.shownRows, ","),
                                      grid.rows.length, new Date());
    }

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
            anchors.right: actions.visible ? actions.left : closeButton.left
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

        // A janela e' estreita: copiar e exportar sao icones com dica.
        Row {
            id: actions

            anchors.right: closeButton.left
            anchors.verticalCenter: parent.verticalCenter
            visible: root.hasRows

            KvIconButton {
                compact: true
                iconName: "copy"
                iconSize: 12
                tooltip: qsTr("Copiar as linhas carregadas em CSV, na ordem da tela")
                onClicked: root.copyAll()
            }

            KvIconButton {
                compact: true
                iconName: "download"
                iconSize: 12
                enabled: root.controller !== null && !root.controller.exports.busy
                tooltip: qsTr("Exportar as linhas carregadas em CSV para %1/ no projeto").arg(DataSourceKinds.exportDirectory)
                onClicked: root.exportAll()
            }
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
        maxHeight: Math.max(40, root.height - statusRow.height - sqlLine.height
                                - (moreRow.visible ? moreRow.height + Theme.spacingXSmall : 0))
        // Clique no cabecalho ordena as linhas carregadas (passo 14).
        sortable: true
        // O core manda o nome da coluna; a grade quer { key, label }.
        columns: root.controller ? root.controller.queryColumns.map(name => ({ key: name, label: name })) : []
        rows: root.controller ? root.controller.queryRows : []
        // Uma escrita nao devolve linhas — ela as afeta (o status diz quantas).
        emptyText: root.querying || root.mustConfirm ? ""
                   : (root.lastQuery !== null && root.lastQuery.wrote === true ? qsTr("Instrução executada.") : qsTr("Sem linhas."))
        // Clique escolhe a linha (e da' o teclado a' grade); Ctrl+C a copia.
        selectable: true
        selectedIndex: grid.sorting.displayIndex(root.selectedSource)
        onRowClicked: index => {
            root.selectedSource = grid.sorting.sourceIndex(index);
            grid.forceActiveFocus();
        }
        onCopyRequested: index => root.copyRow(index)
        onRowsChanged: {
            root.selectedSource = -1;
            root.note("");
        }
    }

    // A BARRA DE BAIXO (passo 14): o resultado de copiar/exportar, ou o aviso
    // de que a ordem vale so' para as linhas carregadas, e o CARREGAR MAIS,
    // que repete a mesma leitura com o teto seguinte. So' aparece quando ha'
    // o que dizer ou carregar.
    Item {
        id: moreRow

        anchors.top: grid.bottom
        anchors.topMargin: Theme.spacingXSmall
        anchors.left: parent.left
        anchors.right: parent.right
        height: moreButton.implicitHeight
        visible: (root.nextCeiling > 0 || root.sortNote !== "" || root.actionNote !== "")
                 && !root.querying && !root.failed

        Text {
            id: noteText

            anchors.left: parent.left
            anchors.right: moreButton.visible ? moreButton.left : parent.right
            anchors.rightMargin: Theme.spacingXSmall
            anchors.verticalCenter: parent.verticalCenter
            text: root.actionNote !== "" ? root.actionNote : root.sortNote
            textFormat: Text.PlainText
            color: root.actionFailed ? Theme.errorSoft : Theme.textMuted
            font.pixelSize: Theme.fontSizeCaption
            elide: Text.ElideRight

            // Cortada (o caminho exportado e' longo): o texto inteiro ao pairar.
            MouseArea {
                anchors.fill: parent
                hoverEnabled: true
                onContainsMouseChanged: {
                    if (containsMouse && noteText.truncated) TooltipController.showFor(noteText, noteText.text, "right");
                    else TooltipController.hideFor(noteText);
                }
            }
        }

        KvButton {
            id: moreButton

            anchors.right: parent.right
            visible: root.nextCeiling > 0
            compact: true
            // Curto: a janela do Banco e' estreita e a nota divide a barra.
            text: qsTr("Carregar mais")
            tooltip: qsTr("Repete a mesma leitura com teto de %1 linhas; a ordem e as larguras ficam.")
                     .arg(Number(root.nextCeiling).toLocaleString(Qt.locale(), "f", 0))
            onClicked: root.controller.queries.loadMore()
        }
    }
}
