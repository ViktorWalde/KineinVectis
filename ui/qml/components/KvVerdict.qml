import QtQuick
import KineinVectis

// O VEREDITO comum dos paineis de ambiente (Etapa 2 F8): "medindo..." em
// cinza enquanto `busy`; depois a faixa verde (`ok`) ou vermelha com o que
// o core MEDIU (em mono, como veio). Era o mesmo retangulo escrito em
// DataSourceVerdict e RemoteVerdict; o banco e o remoto o usam agora, e
// o motor dos containers e a sonda dos embarcados tambem.
Item {
    id: root

    property bool busy: false
    property string busyText: qsTr("medindo…")
    property bool ok: false
    // Um estado que nao e' falha nem sucesso ("nenhuma sonda"): faixa cinza.
    property bool neutral: false
    // O texto da faixa: o que foi medido quando `ok`, a falha quando nao.
    property string message: ""

    readonly property bool showsBand: !busy && (ok || message !== "")

    implicitHeight: busy ? andamento.implicitHeight
                         : (showsBand ? texto.implicitHeight + 2 * Theme.spacingSmall : 0)
    visible: busy || showsBand

    Text {
        id: andamento

        width: parent.width
        visible: root.busy
        text: root.busyText
        color: Theme.textMuted
        font.pixelSize: 10
    }

    Rectangle {
        width: parent.width
        height: root.implicitHeight
        visible: root.showsBand
        radius: Theme.radius
        color: root.ok ? Theme.successSoft : (root.neutral ? Theme.textMuted : Theme.errorSoft)
        opacity: 0.18
    }

    Text {
        id: texto

        x: Theme.spacingSmall
        y: Theme.spacingSmall
        width: parent.width - 2 * Theme.spacingSmall
        visible: root.showsBand
        wrapMode: Text.WordWrap
        text: root.message
        color: root.ok ? Theme.textPrimary : Theme.textSecondary
        font.family: Theme.monoFont
        font.pixelSize: 10
    }
}
