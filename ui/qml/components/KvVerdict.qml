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
    property string heading: ""
    property bool codeFont: true
    property bool animateChanges: false
    onMessageChanged: if (animateChanges && message !== "") reveal.restart()

    NumberAnimation {
        id: reveal
        target: root
        property: "opacity"
        from: 0.65
        to: 1
        duration: Theme.motionMedium
    }

    readonly property bool showsBand: !busy && (ok || message !== "")

    implicitHeight: busy ? andamento.implicitHeight
                         : (showsBand ? body.implicitHeight + 2 * Theme.spacingSmall : 0)
    visible: busy || showsBand

    Text {
        id: andamento

        width: parent.width
        visible: root.busy
        textFormat: Text.PlainText
        text: root.busyText
        color: Theme.textMuted
        font.pixelSize: Theme.fontSizeCaption
    }

    Rectangle {
        width: parent.width
        height: root.implicitHeight
        visible: root.showsBand
        radius: Theme.radius
        color: root.ok ? Theme.successSoft : (root.neutral ? Theme.textMuted : Theme.errorSoft)
        opacity: 0.18
    }

    Column {
        id: body
        x: Theme.spacingSmall
        y: Theme.spacingSmall
        width: parent.width - 2 * Theme.spacingSmall
        visible: root.showsBand
        spacing: Theme.spacingXSmall
        Text {
            width: parent.width
            visible: root.heading !== ""
            textFormat: Text.PlainText
            text: root.heading
            wrapMode: Text.WordWrap
            color: root.ok ? Theme.successSoft : root.neutral ? Theme.textPrimary : Theme.errorSoft
            font.pixelSize: Theme.fontSizeSmall
            font.weight: Font.DemiBold
        }
        Text {
            width: parent.width
            wrapMode: Text.Wrap
            textFormat: Text.PlainText
            text: root.message
            color: root.ok ? Theme.textPrimary : Theme.textSecondary
            font.family: root.codeFont ? Theme.monoFont : Theme.uiFont
            font.pixelSize: root.codeFont ? Theme.fontSizeCaption : Theme.fontSizeBody
        }
    }
}
