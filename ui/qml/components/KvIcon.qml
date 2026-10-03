import QtQuick
import "KvIconGlyphs.js" as Glyphs
import "KvFileIconGlyphs.js" as FileGlyphs

// Iconografia vetorial central da Kinein. Os desenhos (KvIconGlyphs.js) usam
// o grid 24x24, traco 2.0 e cores de estado do Theme. Canvas evita glifos
// dependentes da fonte e permite que a cor siga o Theme em tempo real.
Item {
    id: root

    property string name: "file"
    property real size: 24
    property bool active: false
    property bool disabled: false
    property bool warning: false
    property bool error: false
    property bool success: false
    property color iconColor: disabled ? Theme.textDisabled
                              : error ? Theme.errorSoft
                              : warning ? Theme.warningSoft
                              : success ? Theme.successSoft
                              : active ? Theme.accent : Theme.iconDefault
    readonly property url assetSource: {
        if (root.name === "tree-folder-closed") {
            return "qrc:/KineinVectis/assets/icons/tree/folder-closed.svg";
        }
        if (root.name === "tree-folder-open") {
            return "qrc:/KineinVectis/assets/icons/tree/folder-open.svg";
        }
        return "";
    }

    implicitWidth: size
    implicitHeight: size

    onNameChanged: iconCanvas.requestPaint()
    onIconColorChanged: iconCanvas.requestPaint()
    onWidthChanged: iconCanvas.requestPaint()
    onHeightChanged: iconCanvas.requestPaint()

    Canvas {
        id: iconCanvas

        anchors.fill: parent
        visible: String(root.assetSource) === ""
        antialiasing: true

        onPaint: {
            const context = getContext("2d");
            context.reset();
            context.scale(width / 24, height / 24);
            context.strokeStyle = root.iconColor;
            context.fillStyle = root.iconColor;
            // 2.0 no grid de 24 (era 1.75): a 20 px o traco tinha ~1,4 px e
            // sumia sobre a moldura cinza (0.3.9).
            context.lineWidth = 2.0;
            context.lineCap = "round";
            context.lineJoin = "round";

            // A familia da casca (KvIconGlyphs.js): um SVG path por nome, com
            // a parte cheia quando o signo pede peso.
            const shape = Glyphs.shape(root.name);
            if (shape !== null) {
                if (shape.stroke !== "") {
                    context.path = shape.stroke;
                    context.stroke();
                }
                if (shape.fill !== "") {
                    context.path = shape.fill;
                    context.fill();
                }
                return;
            }
            // Tipos de arquivo: geometria otica propria para 16/20 px; quem
            // nao e' tipo conhecido vira a folha de sempre.
            context.beginPath();
            if (!FileGlyphs.draw(root.name, context)) {
                context.path = Glyphs.shape("file").stroke;
            }
            context.stroke();
        }
    }

    // Os assets da arvore preservam sua paleta e nao recebem a tintura dos
    // icones simbolicos do Canvas.
    Image {
        anchors.fill: parent
        visible: String(root.assetSource) !== ""
        source: root.assetSource
        fillMode: Image.PreserveAspectFit
        asynchronous: false
        cache: true
        smooth: true
    }
}
