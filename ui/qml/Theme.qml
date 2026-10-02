// Design tokens da spec canonica (fatia C1 de DocsPublic/roadmaps/20):
// paleta/estados de DocsPublic/especificacoes/sistema-de-layout.md §7 e
// sistema-de-componentes-de-ui.md §5-§10; escalas de LAYOUT §6 e
// COMPONENTS §6-§7; tipografia de LAYOUT §8.
pragma Singleton
import QtQuick

QtObject {
    readonly property color background0: "#0b0d10"
    readonly property color background1: "#111418"
    readonly property color background2: "#171b21"
    readonly property color backgroundEditor: "#0f1216"
    readonly property color currentLine: "#1a1f26"
    readonly property color surface1: "#171b21"
    readonly property color surface2: "#1a1f27"
    readonly property color surfaceSelected: "#222833"
    readonly property color borderSoft: "#2a2f37"
    readonly property color borderStrong: "#3a414a"
    readonly property color textPrimary: "#e7e2d8"
    readonly property color textSecondary: "#a9a39a"
    readonly property color textMuted: "#6f737a"
    readonly property color textDisabled: "#4e535a"
    readonly property color accent: "#ffb000"
    readonly property color accentActive: "#ffc93d"
    readonly property color accentDim: "#b97900"
    // O fundo escurecido atras de um dialogo modal.
    readonly property color scrim: "#c0000000"

    // Paleta ANSI de 16 cores do terminal (0–7 normais, 8–15 brilhantes),
    // usada pelo renderer de grade (DocsPublic/roadmaps/24 D2).
    readonly property var terminalPalette: [
        "#171b21", "#d05f5f", "#78aa98", "#d0b05f",
        "#6f93c0", "#b07fb0", "#5fb0b0", "#a9a39a",
        "#3a414a", "#ff7f7f", "#91c5b0", "#ffd77f",
        "#8fb0e0", "#d09fd0", "#7fd0d0", "#e7e2d8"
    ]
    readonly property color errorSoft: "#d45f5f"
    readonly property color warningSoft: "#e6b84a"
    readonly property color successSoft: "#7ccf6a"
    readonly property color infoSoft: "#5c8dff"
    readonly property color purpleOrbital: "#8a5cff"

    readonly property int spacingXSmall: 4
    readonly property int spacingSmall: 8
    readonly property int spacingMedium: 12
    readonly property int spacingLarge: 16
    readonly property int spacingRegion: 24
    readonly property int radiusXSmall: 3
    readonly property int radius: 5
    readonly property int radiusLarge: 8
    readonly property int radiusDialog: 12
    readonly property int panelGap: 8

    // TIPOGRAFIA POR PAPEL (0.3.9, roadmap 58 §4.3): os valores sao os que o
    // QML ja' usava a mao (9 a 12 cobrem 476 dos 498 literais medidos em
    // 2026-10-02); a migracao troca o numero pelo papel, sem mudar o tamanho.
    readonly property int fontSizeMicro: 9
    readonly property int fontSizeCaption: 10
    readonly property int fontSizeSmall: 11
    readonly property int fontSizeBody: 12
    readonly property int fontSizeLarge: 14
    readonly property int fontSizeHeadline: 20

    // MOVIMENTO POR PAPEL (0.3.9; decisao do autor de 2026-10-01: fluidez a
    // 60 Hz, 120 Hz onde der). Toda animacao tira a duracao e a curva daqui,
    // para a fluidez ser regra do sistema e nao ajuste de tela. `motionFast`
    // e' o de hover, alternar e aparecer (antes 90 e 120 ms, misturados);
    // os outros sao ritmos de atencao que ja' existiam, agora com nome.
    readonly property int motionFast: 110
    readonly property int motionCaretBlink: 520
    readonly property int motionPulse: 600
    readonly property int motionProgress: 900
    readonly property int easingStandard: Easing.OutCubic
    readonly property int easingPendulum: Easing.InOutSine

    readonly property int fontSizeStatus: 12
    readonly property int fontSizeTree: 13
    readonly property int fontSizeTerminal: 13
    readonly property int fontSizePanelTitle: 13
    // Gravável (M4.1): o SettingsController escreve o valor efetivo de
    // editorFontSize aqui; default 14 casa com o core.
    property int fontSizeEditor: 14

    readonly property string uiFont: "Inter, Noto Sans, sans-serif"
    // `font.family` do QML recebe uma família, não uma pilha CSS. A família
    // genérica deixa o Qt/fontconfig escolher uma fonte realmente monoespaçada
    // em cada desktop Linux, inclusive quando JetBrains Mono não está instalado.
    readonly property string monoFont: "monospace"
}
