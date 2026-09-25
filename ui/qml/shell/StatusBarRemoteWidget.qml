pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O REMOTE NA BARRA DE STATUS (fatia V4, 2026-09-25).
//
// A §5.2 da especificacao do Remote pede o item compacto quando o workspace e'
// espelhado; a §3 lista como defeito 6 que "sync do arquivo salvo e' pouco
// visivel fora do painel". Clicar abre o painel.
//
// Componente burro: recebe a frase pronta da regra e a desenha. QUEM decide o
// que pode ser dito e' o `RemoteHudRules`, que tem harness — e' la' que moram
// as frases que a V4 proibe.
//
// O TIMER existe por honestidade: a frase carrega a IDADE da medida, e idade
// envelhece sozinha. Sem ele, "verificado há 1 min" ficaria na tela por horas.
Row {
    id: root

    // Os FATOS. A frase sai da regra pura; este componente so' desenha.
    property bool isMirror: false
    property string targetName: ""
    property bool syncing: false
    property string syncDirection: ""
    property bool syncFailed: false
    property string syncMessage: ""
    property bool deploying: false
    property bool probed: false
    property bool probeOk: false
    property double probedAt: 0

    // Reavaliado a cada tique: a idade envelhece sozinha, e a frase depende
    // dela. Sem isto, "verificado há 1 min" ficaria na tela por horas.
    property double agora: Date.now()

    readonly property var hud: regras.hudFor({
        "isMirror": root.isMirror,
        "targetName": root.targetName,
        "syncing": root.syncing,
        "syncDirection": root.syncDirection,
        "syncFailed": root.syncFailed,
        "syncMessage": root.syncMessage,
        "deploying": root.deploying,
        "probed": root.probed,
        "probeOk": root.probeOk,
        "probeAgeMs": root.probedAt > 0 ? root.agora - root.probedAt : -1
    })

    signal panelRequested()

    RemoteHudRules {
        id: regras
    }

    spacing: Theme.spacingSmall
    visible: root.hud.visible === true

    Timer {
        // Trinta segundos: a frase mais curta fala em minutos, entao meio
        // minuto e' suficiente para nunca mostrar idade errada por muito tempo.
        interval: 30000
        running: root.visible
        repeat: true
        onTriggered: root.agora = Date.now()
    }

    Rectangle {
        anchors.verticalCenter: parent.verticalCenter
        width: 6
        height: 6
        radius: 3
        color: {
            switch (root.hud.tone) {
            case "ok": return Theme.successSoft;
            case "atencao": return Theme.warningSoft;
            case "ocupado": return Theme.accent;
            default: return Theme.textMuted;
            }
        }
    }

    Text {
        anchors.verticalCenter: parent.verticalCenter
        text: root.hud.label
        color: root.hud.tone === "atencao" ? Theme.warningSoft : Theme.textSecondary
        font.pixelSize: Theme.fontSizeStatus

        MouseArea {
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: root.panelRequested()
            onContainsMouseChanged: {
                if (containsMouse && root.hud.detail !== "") {
                    TooltipController.showFor(parent, root.hud.detail, "top");
                } else {
                    TooltipController.hideFor(parent);
                }
            }
        }
    }
}
