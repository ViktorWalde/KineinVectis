pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// C4: a trilha "src › lsp › manager.rs" acima do texto.
//
// POR QUE ESTE ARQUIVO EXISTE (2026-09-03). Era um Row com Repeater e delegate
// proprio dentro do EditorPane, que ja compunha abas, faixa de conflito,
// superficie, estrutura, popups e dialogos. E' um widget de verdade, com
// desenho proprio — nao fiacao — e por isso sai inteiro.
//
// O GANHO NAO E' SO' TAMANHO: o delegate chamava `path.split("/")` TRES vezes
// (uma para o modelo e duas para descobrir se o segmento era o ultimo). Agora a
// quebra e' derivada UMA vez em `segments`, e "ser o ultimo" e' uma propriedade
// com nome, nao uma comparacao repetida.
//
// Caminho VAZIO nao produz segmento nenhum — `"".split("/")` devolveria [""],
// que desenharia uma trilha de um item invisivel.
Row {
    id: root

    property string path: ""

    readonly property var segments: path === "" ? [] : path.split("/")

    height: visible ? 18 : 0
    spacing: Theme.spacingXSmall

    Repeater {
        model: root.segments

        delegate: Row {
            id: segment

            required property int index
            required property string modelData

            readonly property bool isLast:
                segment.index === root.segments.length - 1

            spacing: Theme.spacingXSmall

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: segment.modelData
                color: segment.isLast ? Theme.textSecondary : Theme.textMuted
                font.pixelSize: 11
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                visible: !segment.isLast
                text: "›"
                color: Theme.textMuted
                font.pixelSize: 11
            }
        }
    }
}
