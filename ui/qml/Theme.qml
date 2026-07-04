// Design tokens from docs/05-design-system.md and imagens/layout-mockup.png.
pragma Singleton
import QtQuick

QtObject {
    readonly property color background0: "#0d0e0e"
    readonly property color background1: "#121313"
    readonly property color background2: "#191a18"
    readonly property color surface1: "#1f201d"
    readonly property color surface2: "#25261f"
    readonly property color borderSoft: "#2a2922"
    readonly property color textPrimary: "#eae6e1"
    readonly property color textSecondary: "#b9b3a5"
    readonly property color textMuted: "#8f8a7c"
    readonly property color accent: "#ffbb00"
    readonly property color accentDim: "#6e5c01"
    readonly property color neutralOlive: "#6e6c58"
    readonly property color errorSoft: "#d16d6d"
    readonly property color warningSoft: "#ffbb00"
    readonly property color successSoft: "#7fbf7f"
    readonly property color infoSoft: "#7aa2d8"

    readonly property int spacingSmall: 6
    readonly property int spacingMedium: 12
    readonly property int spacingLarge: 20
    readonly property int radius: 6
    readonly property int radiusLarge: 10
    readonly property int panelGap: 8

    readonly property string uiFont: "Inter, Noto Sans, sans-serif"
    readonly property string monoFont: "JetBrains Mono, Fira Code, monospace"
}
