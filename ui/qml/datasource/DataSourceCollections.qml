pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A SEGUNDA FORMA: coleção → campo, para os motores sem esquema fixo.
//
// POR QUE ESTE ARQUIVO EXISTE (decisão do autor, 2026-09-04). O
// `DataSourceStructure` ao lado desenha `esquema → tabela → coluna`, e a coluna
// carrega três garantias: ela existe em toda linha, tem UM tipo, e não aninha.
// Nenhuma das três vale num documento. Desenhar o MongoDB naquela árvore faria
// a tela AFIRMAR as três — e tela que mente é o defeito que nenhum gate pega.
//
// As três coisas que só esta forma consegue dizer:
//
//   presença    `firmware` está em 26% dos documentos amostrados
//   tipo plural `firmware` é `int` num documento e `string` noutro
//   aninhamento `carga.origem` é campo dentro de campo
//
// DIMENSIONAMENTO: cada linha mede o próprio conteúdo e tem piso, como o resto
// da IDE desde 2026-09-04.
Item {
    id: root

    property var collections: []
    property bool loading: false

    // Nomes de coleção abertos.
    property var expanded: ({})

    readonly property int alturaMinima: 20

    implicitHeight: coluna.implicitHeight

    function toggle(nome) {
        const atual = Object.assign({}, root.expanded);
        if (atual[nome] === true) {
            delete atual[nome];
        } else {
            atual[nome] = true;
        }
        root.expanded = atual;
    }

    // A LINHA DE PROCEDÊNCIA. Ela é o que separa fato de frequência, e por isso
    // é a primeira coisa escrita embaixo do nome da coleção.
    function procedencia(colecao) {
        if (colecao.declared === true) {
            return qsTr("esquema declarado no validador — não é amostra");
        }
        const total = colecao.documentCount !== undefined
                    ? qsTr("%L1 documentos").arg(colecao.documentCount)
                    : qsTr("total desconhecido");
        const modo = colecao.fullScan === true
                   ? qsTr("leitura completa da coleção")
                   : qsTr("cursor aleatório");
        return qsTr("amostra de %1 em %2 · %3").arg(colecao.sampled).arg(total).arg(modo);
    }

    // Presença em texto. `<1%` em vez de `0%`: um campo que apareceu uma vez
    // em duzentas NÃO apareceu zero vezes, e arredondar para zero apagaria
    // justamente o campo raro que se foi procurar.
    function presenca(campo) {
        if (campo.presence === undefined || campo.presence === null) {
            return "";
        }
        const porcento = campo.presence * 100;
        if (porcento > 0 && porcento < 1) {
            return qsTr("<1%");
        }
        return Math.round(porcento) + "%";
    }

    Column {
        id: coluna

        anchors.left: parent.left
        anchors.right: parent.right
        spacing: 2

        Text {
            width: parent.width
            visible: root.loading
            text: qsTr("Lendo as coleções...")
            color: Theme.textMuted
            font.pixelSize: 10
        }

        Text {
            width: parent.width
            visible: !root.loading && root.collections.length === 0
            wrapMode: Text.WordWrap
            text: qsTr("Clique em \"Ler estrutura\" para ver as coleções e os campos.")
            color: Theme.textDisabled
            font.pixelSize: 10
        }

        Repeater {
            model: root.collections

            delegate: Column {
                id: linhaColecao

                required property var modelData

                readonly property bool aberta:
                    root.expanded[linhaColecao.modelData.name] === true

                width: coluna.width
                spacing: 1

                Rectangle {
                    width: parent.width
                    height: Math.max(root.alturaMinima, tituloColecao.implicitHeight + 4)
                    radius: Theme.radiusXSmall
                    color: areaColecao.containsMouse ? Theme.surface2 : "transparent"

                    Text {
                        id: tituloColecao

                        anchors.left: parent.left
                        anchors.right: parent.right
                        anchors.leftMargin: Theme.spacingXSmall
                        anchors.rightMargin: Theme.spacingXSmall
                        anchors.verticalCenter: parent.verticalCenter
                        elide: Text.ElideRight
                        text: (linhaColecao.aberta ? "▾ " : "▸ ")
                              + linhaColecao.modelData.name
                              + "  " + linhaColecao.modelData.kind
                        color: Theme.textPrimary
                        font.pixelSize: 11
                    }

                    MouseArea {
                        id: areaColecao

                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: root.toggle(linhaColecao.modelData.name)
                    }
                }

                // COLEÇÃO TEMPORAL SE ANUNCIA SOZINHA: isto vem do catálogo do
                // servidor, sem ler um documento — por isso não aparece como
                // campo comum no meio dos outros.
                Text {
                    x: Theme.spacingMedium
                    width: parent.width - Theme.spacingMedium
                    visible: linhaColecao.modelData.timeField !== undefined
                             && linhaColecao.modelData.timeField !== ""
                    wrapMode: Text.WordWrap
                    text: qsTr("temporal · tempo em %1, metadados em %2, granularidade %3")
                            .arg(linhaColecao.modelData.timeField)
                            .arg(linhaColecao.modelData.metaField)
                            .arg(linhaColecao.modelData.granularity)
                    color: Theme.infoSoft
                    font.pixelSize: 9
                }

                Text {
                    x: Theme.spacingMedium
                    width: parent.width - Theme.spacingMedium
                    wrapMode: Text.WordWrap
                    text: root.procedencia(linhaColecao.modelData)
                    color: linhaColecao.modelData.declared === true
                           ? Theme.successSoft : Theme.textMuted
                    font.pixelSize: 9
                }

                // O QUE FOI CORTADO, dito. Uma árvore que parece completa e não
                // é seria pior que uma árvore vazia.
                Text {
                    x: Theme.spacingMedium
                    width: parent.width - Theme.spacingMedium
                    visible: linhaColecao.modelData.truncated !== undefined
                             && linhaColecao.modelData.truncated !== ""
                    wrapMode: Text.WordWrap
                    text: "⚠ " + linhaColecao.modelData.truncated
                    color: Theme.warningSoft
                    font.pixelSize: 9
                }

                Repeater {
                    model: linhaColecao.aberta ? linhaColecao.modelData.fields : []

                    delegate: Item {
                        id: linhaCampo

                        required property var modelData

                        width: linhaColecao.width
                        height: Math.max(root.alturaMinima - 4,
                                         nomeCampo.implicitHeight + 2)

                        Text {
                            id: nomeCampo

                            anchors.left: parent.left
                            anchors.leftMargin: Theme.spacingMedium
                                                + linhaCampo.modelData.depth * Theme.spacingMedium
                            anchors.right: tiposCampo.left
                            anchors.rightMargin: Theme.spacingSmall
                            anchors.verticalCenter: parent.verticalCenter
                            elide: Text.ElideRight
                            // O caminho já traz os pais; a folha basta na linha.
                            text: String(linhaCampo.modelData.path).split(".").pop()
                                  + (linhaCampo.modelData.required === true ? " *" : "")
                            color: Theme.textSecondary
                            font.family: Theme.monoFont
                            font.pixelSize: 10
                        }

                        // MAIS DE UM TIPO NÃO É ERRO — é o que uma coluna não
                        // consegue dizer, e o que quebra o código de quem
                        // assumiu um só. Por isso ganha destaque.
                        Text {
                            id: tiposCampo

                            anchors.right: presencaCampo.left
                            anchors.rightMargin: Theme.spacingSmall
                            anchors.verticalCenter: parent.verticalCenter
                            text: linhaCampo.modelData.types.join(" | ")
                            color: linhaCampo.modelData.types.length > 1
                                   ? Theme.warningSoft : Theme.infoSoft
                            font.family: Theme.monoFont
                            font.pixelSize: 9
                        }

                        Text {
                            id: presencaCampo

                            anchors.right: parent.right
                            anchors.rightMargin: Theme.spacingXSmall
                            anchors.verticalCenter: parent.verticalCenter
                            width: 34
                            horizontalAlignment: Text.AlignRight
                            text: root.presenca(linhaCampo.modelData)
                            color: Theme.textMuted
                            font.family: Theme.monoFont
                            font.pixelSize: 9
                        }
                    }
                }
            }
        }
    }
}
