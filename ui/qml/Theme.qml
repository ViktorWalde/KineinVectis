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
