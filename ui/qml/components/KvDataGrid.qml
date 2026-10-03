pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A GRADE comum dos paineis de ambiente (Etapa 2 F8): cabecalho fixo,
// colunas com a largura da regra pura (GridRules), `null` em italico,
// rolagem nos dois sentidos quando nao cabe. Nasceu da grade do banco
// (a secao de dados da janela do Banco) e serve a containers, portas seriais e sondas.
Item {
    id: root

    // [{ key, label }] — `label` opcional (cai no `key`).
    property var columns: []
    // Linhas: objetos (por `key`) ou arrays (por posicao).
    property var rows: []
    property string emptyText: ""
    property bool mono: true
    property int maxHeight: 220
    property int rowHeight: 20
    // Selecao por linha (E3-6): -1 = nenhuma; quem escuta `rowClicked`
    // decide o que a linha escolhida permite (a barra de acoes).
    property int selectedIndex: -1
    property bool selectable: false

    signal rowClicked(int index)
    // ABRIR o que a linha representa — Enter no teclado, e o que cada painel
    // entender por "abrir". Separado de escolher: escolher e' de graca,
    // abrir pode lancar um navegador.
    signal rowActivated(int index)

    // TECLADO (§5.2 da especificacao do Grafana: "selecao e teclado").
    //
    // A grade NAO escreve em `selectedIndex`. Ela emite `rowClicked`, igual ao
    // mouse, e quem e' dono da selecao continua sendo quem sempre foi — no
    // antigo `ContainerListView` ela era um binding para o controller, e escrever aqui
    // quebraria esse binding em silencio, que e' a forma de estrago que o QML
    // nao acusa.
    activeFocusOnTab: root.selectable && root.rows.length > 0

    function stepSelection(passo) {
        if (!root.selectable || root.rows.length === 0) {
            return;
        }
        // Sem selecao ainda: a primeira tecla escolhe a ponta de onde ela vem.
        const atual = root.selectedIndex;
        const bruto = atual < 0 ? (passo > 0 ? 0 : root.rows.length - 1) : atual + passo;
        const alvo = Math.max(0, Math.min(root.rows.length - 1, bruto));
        if (alvo !== atual) {
            root.rowClicked(alvo);
        }
    }

    // A LINHA ESCOLHIDA PRECISA ESTAR A' VISTA: navegar por teclado ate' uma
    // linha fora da area rolada e' o mesmo que nao navegar.
    function ensureVisible(indice) {
        if (indice < 0 || root.rows.length === 0) {
            return;
        }
        const passo = root.rowHeight + 1;
        const topo = indice * passo;
        const fundo = topo + passo;
        if (topo < corpo.contentY) {
            corpo.contentY = topo;
        } else if (fundo > corpo.contentY + corpo.height) {
            corpo.contentY = fundo - corpo.height;
        }
    }

    onSelectedIndexChanged: root.ensureVisible(root.selectedIndex)

    Keys.onUpPressed: root.stepSelection(-1)
    Keys.onDownPressed: root.stepSelection(1)
    Keys.onPressed: evento => {
        if (!root.selectable || root.rows.length === 0) {
            return;
        }
        if (evento.key === Qt.Key_Home) {
            root.stepSelection(-root.rows.length);
            evento.accepted = true;
        } else if (evento.key === Qt.Key_End) {
            root.stepSelection(root.rows.length);
            evento.accepted = true;
        } else if (evento.key === Qt.Key_Return || evento.key === Qt.Key_Enter) {
            if (root.selectedIndex >= 0) {
                root.rowActivated(root.selectedIndex);
                evento.accepted = true;
            }
        }
    }

    // A largura que a pessoa ARRASTOU em cada coluna ({indice: px}); clique
    // duplo na alca volta a coluna ao natural. Colunas novas (outra consulta)
    // comecam do natural.
    property var overrides: ({})
    onColumnsChanged: root.overrides = ({})

    function resizeColumn(index, width) {
        const next = Object.assign({}, root.overrides);
        next[index] = Math.max(rules.minWidth, Math.round(width));
        root.overrides = next;
    }

    function autoFitColumn(index) {
        const next = Object.assign({}, root.overrides);
        delete next[index];
        root.overrides = next;
    }

    readonly property var widths: rules.columnWidths(columns, rows, width, overrides)
    // Colunas de numero alinham a direita.
    readonly property var numeric: columns.map((column, index) => rules.isNumericColumn(rows, column, index))
    readonly property int contentWidth: widths.reduce((sum, w) => sum + w + 1, 0)
    // A largura em que tudo cabe sem rolar (a janela do Banco se alarga ate' ela).
    readonly property int naturalWidth: rules.naturalTotal(columns, rows)

    implicitHeight: columns.length === 0
                    ? (emptyText === "" ? 0 : vazio.implicitHeight)
                    : Math.min(maxHeight, rowHeight + 1 + rows.length * (rowHeight + 1))

    // A largura natural sai da FONTE que desenha (o cabecalho, em negrito, e'
    // o mais largo), e nao de um palpite de 7 px por caractere. A medida e'
    // um Text igual ao do cabecalho, fora da tela: o FontMetrics devolvia o
    // dobro (outra fonte), e a coluna `cliente_id` saia com 170 px.
    Text {
        id: probe

        visible: false
        text: "MMMMMMMMMM"
        font.family: root.mono ? Theme.monoFont : ""
        font.pixelSize: Theme.fontSizeCaption
        font.weight: Font.DemiBold
    }

    GridRules {
        id: rules

        charWidth: probe.implicitWidth / 10
    }

    Text {
        id: vazio

        width: parent.width
        visible: root.columns.length === 0 && root.emptyText !== ""
        text: root.emptyText
        color: Theme.textMuted
        font.pixelSize: Theme.fontSizeSmall
        wrapMode: Text.WordWrap
    }

    // O cabecalho, com a alca de largura de cada coluna.
    KvDataGridHeader {
        id: cabecalho

        visible: root.columns.length > 0
        x: -corpo.contentX
        grid: root
    }

    // ONDE O TECLADO ESTA' FALANDO. Sem isto, a grade responde a setas sem
    // dizer que e' ela quem responde.
    Rectangle {
        anchors.fill: corpo
        anchors.margins: -1
        visible: root.activeFocus
        color: "transparent"
        radius: Theme.radiusXSmall
        border.width: 1
        border.color: Theme.accent
    }

    Flickable {
        id: corpo

        anchors.top: cabecalho.bottom
        anchors.topMargin: 1
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        visible: root.columns.length > 0
        clip: true
        contentWidth: Math.max(width, root.contentWidth)
        contentHeight: linhas.implicitHeight
        boundsBehavior: Flickable.StopAtBounds

        Column {
            id: linhas

            spacing: 1

            Repeater {
                model: root.rows

                delegate: Row {
                    id: linha

                    required property var modelData
                    required property int index

                    readonly property bool selected: root.selectable && root.selectedIndex === index

                    spacing: 1

                    Repeater {
                        model: root.columns

                        delegate: Rectangle {
                            id: celula

                            required property var modelData
                            required property int index

                            readonly property var value: rules.cellOf(linha.modelData, modelData, index)

                            width: root.widths[index]
                            height: root.rowHeight
                            color: linha.selected ? Theme.surfaceSelected
                                   : (area.containsMouse ? Theme.surface2 : Theme.background1)

                            Text {
                                id: cellText

                                anchors.fill: parent
                                anchors.leftMargin: 5
                                anchors.rightMargin: 5
                                verticalAlignment: Text.AlignVCenter
                                horizontalAlignment: root.numeric[celula.index] ? Text.AlignRight : Text.AlignLeft
                                text: rules.cellText(celula.value)
                                font.italic: rules.isNull(celula.value)
                                color: rules.isNull(celula.value) ? Theme.textMuted
                                       : (linha.selected ? Theme.textPrimary : Theme.textSecondary)
                                font.family: root.mono ? Theme.monoFont : ""
                                font.pixelSize: Theme.fontSizeCaption
                                elide: Text.ElideRight
                            }

                            // O realce ao pairar (a linha inteira acende) e, se a
                            // grade e' selecionavel, o clique escolhe a linha.
                            MouseArea {
                                id: area

                                anchors.fill: parent
                                hoverEnabled: true
                                acceptedButtons: root.selectable ? Qt.LeftButton : Qt.NoButton
                                cursorShape: root.selectable ? Qt.PointingHandCursor : Qt.ArrowCursor
                                onClicked: root.rowClicked(linha.index)
                                // Valor cortado: o texto inteiro ao pairar.
                                onContainsMouseChanged: {
                                    if (containsMouse && cellText.truncated) TooltipController.showFor(celula, cellText.text, "bottom");
                                    else TooltipController.hideFor(celula);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
