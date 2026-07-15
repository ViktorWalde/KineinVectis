import QtQuick

// Iconografia vetorial central da Kinein. Os desenhos usam o grid 24x24,
// stroke 1.75px e cores de estado definidos pela spec visual. Canvas evita
// glifos dependentes da fonte e permite que a cor siga Theme em tempo real.
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
                              : active ? Theme.accent : Theme.textSecondary

    implicitWidth: size
    implicitHeight: size

    onNameChanged: iconCanvas.requestPaint()
    onIconColorChanged: iconCanvas.requestPaint()
    onWidthChanged: iconCanvas.requestPaint()
    onHeightChanged: iconCanvas.requestPaint()

    Canvas {
        id: iconCanvas

        anchors.fill: parent
        antialiasing: true

        function line(context, x1, y1, x2, y2) {
            context.moveTo(x1, y1);
            context.lineTo(x2, y2);
        }

        function node(context, x, y, radius) {
            context.moveTo(x + radius, y);
            context.arc(x, y, radius, 0, Math.PI * 2, false);
        }

        onPaint: {
            const context = getContext("2d");
            context.reset();
            context.scale(width / 24, height / 24);
            context.strokeStyle = root.iconColor;
            context.fillStyle = root.iconColor;
            context.lineWidth = 1.75;
            context.lineCap = "round";
            context.lineJoin = "round";
            context.beginPath();

            switch (root.name) {
            case "project":
            case "folder":
                context.moveTo(3, 7);
                context.lineTo(8.5, 7);
                context.lineTo(10.5, 9);
                context.lineTo(21, 9);
                context.lineTo(20, 19);
                context.lineTo(3, 19);
                context.closePath();
                line(context, 5, 12, 13, 12);
                break;
            case "search":
                context.arc(10.5, 10.5, 5.5, 0.25, Math.PI * 2 - 0.25, false);
                line(context, 14.8, 14.8, 20, 20);
                break;
            case "git":
            case "branch":
                line(context, 7, 5, 7, 19);
                line(context, 7, 9, 16, 9);
                line(context, 16, 9, 16, 15);
                node(context, 7, 5, 2);
                node(context, 7, 19, 2);
                node(context, 16, 17, 2);
                break;
            case "build":
                context.moveTo(5, 5);
                context.lineTo(13, 12);
                context.lineTo(5, 19);
                line(context, 12, 19, 20, 19);
                break;
            case "debug":
                context.moveTo(5, 5);
                context.lineTo(14, 12);
                context.lineTo(5, 19);
                node(context, 10, 12, 2);
                line(context, 17, 7, 20, 4);
                line(context, 17, 17, 20, 20);
                break;
            case "run":
                context.moveTo(6, 4.5);
                context.lineTo(18.5, 12);
                context.lineTo(6, 19.5);
                context.closePath();
                break;
            case "stop":
                context.moveTo(6, 5);
                context.lineTo(18, 5);
                context.lineTo(19, 17);
                context.lineTo(17, 19);
                context.lineTo(5, 18);
                context.lineTo(5, 6);
                context.closePath();
                break;
            case "test":
                context.moveTo(4, 12);
                context.lineTo(9, 17);
                context.lineTo(20, 6);
                node(context, 19, 18, 1.25);
                line(context, 7, 20, 14, 20);
                break;
            case "configure":
                context.moveTo(12, 4);
                context.lineTo(20, 19);
                context.lineTo(4, 19);
                context.lineTo(10.5, 7);
                node(context, 6, 18, 1.5);
                break;
            case "tools":
                line(context, 5, 7, 12, 12);
                line(context, 12, 12, 19, 6);
                line(context, 12, 12, 18, 19);
                node(context, 5, 7, 2);
                node(context, 19, 6, 2);
                node(context, 18, 19, 2);
                node(context, 12, 12, 2.2);
                break;
            case "terminal":
                context.rect(3, 5, 18, 14);
                context.moveTo(6, 9);
                context.lineTo(9, 12);
                context.lineTo(6, 15);
                line(context, 12, 15, 17, 15);
                break;
            case "problems":
            case "warning":
                context.moveTo(12, 3.5);
                context.lineTo(21, 19.5);
                context.lineTo(3, 19.5);
                context.closePath();
                line(context, 12, 9, 12, 14);
                node(context, 12, 17, 0.7);
                break;
            case "settings":
                line(context, 5, 7, 19, 7);
                line(context, 5, 12, 19, 12);
                line(context, 5, 17, 19, 17);
                node(context, 9, 7, 2);
                node(context, 15, 12, 2);
                node(context, 11, 17, 2);
                break;
            case "context":
                context.moveTo(12, 3.5);
                context.lineTo(14, 9.5);
                context.lineTo(20.5, 12);
                context.lineTo(14, 14.5);
                context.lineTo(12, 20.5);
                context.lineTo(10, 14.5);
                context.lineTo(3.5, 12);
                context.lineTo(10, 9.5);
                context.closePath();
                break;
            case "close":
                line(context, 6, 6, 18, 18);
                line(context, 18, 6, 6, 18);
                break;
            case "refresh":
                context.arc(12, 12, 7, -0.4, Math.PI * 1.45, false);
                context.moveTo(5, 6);
                context.lineTo(5, 11);
                context.lineTo(10, 10);
                break;
            case "expand":
                line(context, 4, 9, 4, 4);
                line(context, 4, 4, 9, 4);
                line(context, 15, 4, 20, 4);
                line(context, 20, 4, 20, 9);
                line(context, 20, 15, 20, 20);
                line(context, 20, 20, 15, 20);
                line(context, 9, 20, 4, 20);
                line(context, 4, 20, 4, 15);
                break;
            case "collapse":
                line(context, 9, 4, 9, 9);
                line(context, 9, 9, 4, 9);
                line(context, 15, 4, 15, 9);
                line(context, 15, 9, 20, 9);
                line(context, 15, 20, 15, 15);
                line(context, 15, 15, 20, 15);
                line(context, 9, 20, 9, 15);
                line(context, 9, 15, 4, 15);
                break;
            case "help":
                context.arc(12, 12, 9, 0, Math.PI * 2, false);
                context.moveTo(9, 9.5);
                context.bezierCurveTo(9.5, 6.5, 14.8, 6.5, 15, 9.5);
                context.bezierCurveTo(15.1, 11.2, 12, 11.8, 12, 14);
                node(context, 12, 17.5, 0.7);
                break;
            case "back":
                context.moveTo(10, 5);
                context.lineTo(3, 12);
                context.lineTo(10, 19);
                line(context, 3.5, 12, 21, 12);
                break;
            case "chevron-down":
                context.moveTo(5, 9);
                context.lineTo(12, 16);
                context.lineTo(19, 9);
                break;
            case "chevron-up":
                context.moveTo(5, 15);
                context.lineTo(12, 8);
                context.lineTo(19, 15);
                break;
            case "check":
                context.moveTo(4.5, 12.5);
                context.lineTo(9.5, 17.5);
                context.lineTo(19.5, 6.5);
                break;
            case "pull":
                line(context, 12, 4, 12, 18);
                context.moveTo(6, 12);
                context.lineTo(12, 18);
                context.lineTo(18, 12);
                line(context, 5, 21, 19, 21);
                break;
            case "push":
                line(context, 12, 20, 12, 6);
                context.moveTo(6, 12);
                context.lineTo(12, 6);
                context.lineTo(18, 12);
                line(context, 5, 3, 19, 3);
                break;
            case "file":
            default:
                context.moveTo(6, 3);
                context.lineTo(14, 3);
                context.lineTo(19, 8);
                context.lineTo(19, 21);
                context.lineTo(6, 21);
                context.closePath();
                line(context, 14, 3, 14, 8);
                line(context, 14, 8, 19, 8);
                break;
            }

            context.stroke();
        }
    }
}
