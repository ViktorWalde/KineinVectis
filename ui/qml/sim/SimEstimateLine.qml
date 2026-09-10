pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O CUSTO NA TELA, antes de rodar.
//
// Mesmo idioma do `$sample` da etapa 27: a IDE conta e diz o que vai acontecer,
// e quem decide e' o autor. Ela nao corrige o passo nem escolhe o motor
// (arquitetura/34 §2.1, revisado em 2026-09-05).
Column {
    id: root

    property var estimate: null

    visible: root.estimate !== null
    spacing: 2

    function mb(bytes) {
        if (bytes === undefined) {
            return "";
        }
        if (bytes < 1000000) {
            return (bytes / 1000).toFixed(1) + " KB";
        }
        if (bytes < 1000000000) {
            return (bytes / 1000000).toFixed(1) + " MB";
        }
        return (bytes / 1000000000).toFixed(2) + " GB";
    }

    Text {
        width: root.width
        wrapMode: Text.WordWrap
        text: root.estimate === null ? "" :
              qsTr("%1 passos. A trilha guarda 1 a cada %2 — %3, contra %4 se guardasse tudo.")
                .arg(root.estimate.steps)
                .arg(root.estimate.sampleEvery)
                .arg(root.mb(root.estimate.sampledTrailBytes))
                .arg(root.mb(root.estimate.fullTrailBytes))
        color: Theme.textMuted
        font.family: Theme.uiFont
        font.pixelSize: Theme.fontSizeStatus
    }

    Text {
        width: root.width
        wrapMode: Text.WordWrap
        visible: root.estimate !== null && root.estimate.tooMany === true
        text: root.estimate === null ? "" :
              qsTr("Acima do limite de %1 passos: a IDE vai recusar. Aumente o passo "
                   + "ou reduza a duração.").arg(root.estimate.limit)
        color: Theme.warningSoft
        font.family: Theme.uiFont
        font.pixelSize: Theme.fontSizeStatus
    }

    Text {
        width: root.width
        wrapMode: Text.WordWrap
        visible: root.estimate !== null && root.estimate.compilingWouldPay === true
        text: qsTr("Nesta escala, compilar a fórmula passaria a compensar — o ponto de "
                   + "virada medido é 21,5 milhões de avaliações. Compilar ainda não "
                   + "existe nesta versão.")
        color: Theme.textMuted
        font.family: Theme.uiFont
        font.pixelSize: Theme.fontSizeStatus
    }
}
