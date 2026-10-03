// Design tokens da spec canonica (fatia C1 de DocsPublic/roadmaps/20):
// paleta/estados de DocsPublic/especificacoes/sistema-de-layout.md §7 e
// sistema-de-componentes-de-ui.md §5-§10; escalas de LAYOUT §6 e
// COMPONENTS §6-§7; tipografia de LAYOUT §8.
pragma Singleton
import QtQuick

QtObject {
    // Paleta "ilhas" (0.3.9, pedido do autor com o tema Islands da JetBrains
    // como inspiracao, o verde trocado pelo ambar): a MOLDURA (janela, topo,
    // status, trilho) e' cinza; as ILHAS (explorador, editor, painel de
    // baixo) sao mais escuras e sem borda marcada; o codigo tem a cor da
    // ilha. Medida no print de referencia: moldura #26282c, ilha #191a1c.
    readonly property color frame: "#26282c"
    // O veu ambar no canto do topo: a moldura com ~8% de ambar.
    readonly property color frameAccentTint: "#373328"
    readonly property color background0: "#141517"
    readonly property color background1: "#191a1c"
    readonly property color background2: "#1f2023"
    readonly property color backgroundEditor: "#191a1c"
    readonly property color currentLine: "#1f2024"
    readonly property color surface1: "#1e1f22"
    readonly property color surface2: "#2b2d30"
    readonly property color surfaceSelected: "#33353b"
    readonly property color borderSoft: "#2b2d30"
    readonly property color borderStrong: "#3c3f44"
    // Texto confortavel (0.3.9, pedido do autor: "branco muito puro"): 10,7:1
    // sobre a area, 7,3:1 o secundario (WCAG AAA); o fraco fica em 4,6:1.
    readonly property color textPrimary: "#c8cbd0"
    readonly property color textSecondary: "#a4a8af"
    // Icones de traco (0.3.9, retorno do autor: "muito fracos"): em repouso um
    // pouco mais claros que o texto secundario; no hover, quase o texto
    // principal claro. O ativo continua ambar.
    readonly property color iconDefault: "#b9bdc4"
    readonly property color iconHover: "#e2e4e8"
    readonly property color textMuted: "#80848b"
    readonly property color textDisabled: "#5a5d63"
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
    // O vao entre a moldura e a ilha, e dentro dela (0.3.9: 8 -> 6, a
    // proporcao da referencia do autor).
    readonly property int panelGap: 6

    // TIPOGRAFIA POR PAPEL (0.3.9, roadmap 58 §4.3): os valores sao os que o
    // QML ja' usava a mao (9 a 12 cobrem 476 dos 498 literais medidos em
    // 2026-10-02); a migracao troca o numero pelo papel, sem mudar o tamanho.
    readonly property int fontSizeMicro: 9
    readonly property int fontSizeCaption: 10
    readonly property int fontSizeSmall: 11
    readonly property int fontSizeBody: 12
    readonly property int fontSizeMedium: 13
    readonly property int fontSizeLarge: 14
    readonly property int fontSizeSubtitle: 15
    readonly property int fontSizeHeadline: 20
    readonly property int fontSizeDisplay: 22

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
