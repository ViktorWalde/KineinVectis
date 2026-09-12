pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A PREVIA: o texto que vai ser gravado no arquivo do usuario.
//
// POR QUE ESTE ARQUIVO EXISTE (2026-09-04). Ele saiu do ConfigActionPreview
// por RESPONSABILIDADE, nao por tamanho: aquele painel responde "o que esta
// selecionado e o que voce precisa preencher"; este responde "como o arquivo
// vai ficar". Sao duas perguntas, e a segunda e' a unica que justifica o
// consentimento — ela merece dono, rolagem e espaco proprios.
//
// O realce usa prefixo/sufixo comum de linhas — exato para o que estas acoes
// fazem (inserir e acrescentar bloco) e honesto quando nao e: uma reescrita do
// arquivo inteiro aparece como um bloco grande alterado, que e a verdade. A UI
// nao inventa um diff mais bonito do que o efeito real; o texto mostrado E o
// que sera gravado, porque veio do mesmo plano do core.
Rectangle {
    id: root

    // O arquivo do plano (`path`, `after`, e `before` quando ja existe), ou
    // null quando a acao nao altera arquivo nenhum.
    property var file: null
    // Acoes de efeito `report` nao escrevem: elas devolvem linhas de texto.
    property var reportLines: []
    property bool loading: false
    // Nome do campo obrigatorio que ainda falta, ou "" quando nada falta.
    property string missingParam: ""

    radius: Theme.radius
    color: Theme.backgroundEditor
    border.color: Theme.borderSoft
    border.width: 1
    clip: true

    // Linhas do resultado, marcadas como alteradas ou nao.
    function diffLines() {
        if (root.file === null) {
            return [];
        }
        const after = root.file.after.split("\n");
        const before = root.file.before !== undefined ? root.file.before.split("\n") : [];
        let head = 0;
        while (head < before.length && head < after.length && before[head] === after[head]) {
            ++head;
        }
        let tail = 0;
        while (tail < before.length - head && tail < after.length - head
               && before[before.length - 1 - tail] === after[after.length - 1 - tail]) {
            ++tail;
        }
        const lines = [];
        for (let index = 0; index < after.length; ++index) {
            lines.push({
                text: after[index],
                added: index >= head && index < after.length - tail
            });
        }
        return lines;
    }

    Text {
        id: diffPath

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.spacingSmall
        text: root.file !== null
              ? (root.file.before === undefined
                 ? qsTr("%1 (arquivo novo)").arg(root.file.path)
                 : root.file.path)
              : ""
        color: Theme.textMuted
        font.pixelSize: 10
        visible: root.file !== null
    }

    ListView {
        id: linhas

        anchors.top: root.file !== null ? diffPath.bottom : parent.top
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.spacingSmall
        model: root.file !== null ? root.diffLines() : root.reportLines
        boundsBehavior: Flickable.StopAtBounds
        clip: true

        delegate: Text {
            required property var modelData

            text: typeof modelData === "string" ? modelData : modelData.text
            color: (typeof modelData !== "string" && modelData.added)
                   ? Theme.successSoft : Theme.textSecondary
            font.family: Theme.monoFont
            font.pixelSize: 11
        }

        // B2 (DocsPublic/roadmaps/24): sem ela, um CMakeLists que nao cabe na caixa
        // nao AVISA que continua abaixo — e o usuario consente com o que viu.
        // `parent: linhas` e obrigatorio: filho declarado dentro de um ListView
        // vira filho do contentItem e rolaria junto.
        VerticalScrollBar {
            parent: linhas
            anchors.right: linhas.right
            anchors.top: linhas.top
            anchors.bottom: linhas.bottom

            contentSize: linhas.contentHeight
            viewportSize: linhas.height
            position: linhas.contentY

            onMoveRequested: function (position) {
                linhas.contentY = position;
            }
        }
    }

    Text {
        anchors.centerIn: parent
        width: parent.width - 2 * Theme.spacingMedium
        horizontalAlignment: Text.AlignHCenter
        wrapMode: Text.WordWrap
        text: root.loading
              ? qsTr("Calculando o plano...")
              : (root.missingParam !== ""
                 ? qsTr("Informe: %1").arg(root.missingParam)
                 : qsTr("Esta ação não altera arquivo nenhum."))
        color: Theme.textMuted
        font.pixelSize: 11
        visible: root.file === null && root.reportLines.length === 0
    }
}
