import QtQuick
import KineinVectis

// Os resumos do PROJETO na barra de status: o indice do projeto inteiro e o
// contexto de compilador do arquivo ativo (detalhe ao pairar). O Python
// morou aqui de 2026-09-13 a 2026-10-02; subiu para o chip do cabecalho, que
// tambem o configura (0.3.8 F3, decisao do autor de nao repetir).
// Saiu da WorkspaceStatusBar quando ela chegou a 298/300: "resumos do
// projeto" e' uma responsabilidade que a barra so' posiciona.
//
// POR PRIORIDADE, NUNCA CORTADO (53 §4.4, decisao do autor de 2026-10-01): a
// faixa esquerda da barra recorta o que passa, e a 1024 px o contexto saia
// partido ao meio ("simbolos ‹"). Agora a barra diz quanto sobra
// (`availableWidth`) e os resumos entram por ordem de importancia — o
// contexto do compilador, o indice —, cada um INTEIRO ou nenhum.
Row {
    id: root

    property string indexSummary: ""
    property string contextSummary: ""
    property string contextDetail: ""
    property real availableWidth: 100000

    // Quais cabem, na ordem de prioridade dada: um item entra se ele e o
    // espaco antes dele cabem no que sobrou; item de largura 0 (vazio) nao
    // conta. Funcao pura, testada no tst_status_bar_priority.qml.
    function fitByPriority(widths, available, gap) {
        let used = 0;
        const shown = [];
        for (let i = 0; i < widths.length; i++) {
            const extra = widths[i] + (used > 0 ? gap : 0);
            const fits = widths[i] > 0 && used + extra <= available;
            shown.push(fits);
            if (fits) {
                used += extra;
            }
        }
        return shown;
    }

    // Ordem de prioridade: contexto, indice.
    readonly property var fitting: fitByPriority(
        [root.contextSummary !== "" ? Math.min(contextMetrics.advanceWidth, 520) : 0,
         root.indexSummary !== "" ? indexText.implicitWidth : 0],
        availableWidth, spacing)

    spacing: Theme.spacingMedium

    // A largura do contexto EM REPOUSO: pairar troca o texto pelo detalhe, e
    // medir o texto exibido faria os outros resumos piscarem.
    TextMetrics {
        id: contextMetrics

        font: contextText.font
        text: qsTr("contexto: %1").arg(root.contextSummary)
    }

    Text {
        id: indexText

        anchors.verticalCenter: parent.verticalCenter
        visible: root.fitting[1]
        text: qsTr("índice: %1").arg(root.indexSummary)
        color: Theme.textMuted
        font.pixelSize: Theme.fontSizeStatus
    }

    Text {
        id: contextText

        anchors.verticalCenter: parent.verticalCenter
        visible: root.fitting[0]
        text: contextArea.containsMouse && root.contextDetail !== ""
              ? root.contextDetail
              : qsTr("contexto: %1").arg(root.contextSummary)
        color: Theme.textMuted
        font.pixelSize: Theme.fontSizeStatus
        elide: Text.ElideMiddle
        width: Math.min(implicitWidth, 520)

        MouseArea {
            id: contextArea

            anchors.fill: parent
            hoverEnabled: true
            acceptedButtons: Qt.NoButton
        }
    }
}
