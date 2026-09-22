.pragma library

// Glifos de arquivo desenhados diretamente na grade 24x24. A arte e'
// deliberadamente simples: uma metafora principal, contraste alto e nenhum
// microtexto, sombra ou detalhe que desapareca quando exibido a 16/20 px.

function path(context, points, closePath) {
    context.beginPath();
    context.moveTo(points[0][0], points[0][1]);
    for (let i = 1; i < points.length; ++i) {
        context.lineTo(points[i][0], points[i][1]);
    }
    if (closePath) {
        context.closePath();
    }
}

function strokeLine(context, x1, y1, x2, y2, color, width) {
    context.beginPath();
    context.moveTo(x1, y1);
    context.lineTo(x2, y2);
    context.strokeStyle = color;
    context.lineWidth = width;
    context.stroke();
}

function document(context, accent) {
    context.fillStyle = "#202832";
    context.strokeStyle = "#91a0af";
    context.lineWidth = 1.35;
    path(context, [[3, 1.5], [15.5, 1.5], [21, 7], [21, 22.5],
                   [3, 22.5]], true);
    context.fill();
    context.stroke();

    context.fillStyle = accent;
    path(context, [[15.5, 1.8], [20.7, 7], [15.5, 7]], true);
    context.fill();
}

function plus(context, x, y, color) {
    strokeLine(context, x - 1.7, y, x + 1.7, y, color, 1.7);
    strokeLine(context, x, y - 1.7, x, y + 1.7, color, 1.7);
}

function letterC(context, centerX, centerY, radius, color, width) {
    context.beginPath();
    context.arc(centerX, centerY, radius, 0.65, Math.PI * 2 - 0.65, false);
    context.strokeStyle = color;
    context.lineWidth = width;
    context.stroke();
}

function letterH(context, centerX, centerY, color) {
    strokeLine(context, centerX - 3, centerY - 4, centerX - 3,
               centerY + 4, color, 2.1);
    strokeLine(context, centerX + 3, centerY - 4, centerX + 3,
               centerY + 4, color, 2.1);
    strokeLine(context, centerX - 3, centerY, centerX + 3,
               centerY, color, 2.1);
}

function drawCMake(context) {
    context.fillStyle = "#e05252";
    path(context, [[7, 17.5], [11.5, 7], [13.5, 12]], true);
    context.fill();
    context.fillStyle = "#55aaff";
    path(context, [[12.2, 6.5], [18, 15], [14, 13]], true);
    context.fill();
    context.fillStyle = "#63c174";
    path(context, [[7.5, 18], [18, 16], [13.5, 13.5]], true);
    context.fill();
}

function drawCargo(context) {
    context.strokeStyle = "#e99b54";
    context.lineWidth = 1.7;
    path(context, [[7, 10], [12, 7], [17, 10], [17, 16], [12, 19], [7, 16]], true);
    context.stroke();
    strokeLine(context, 7.5, 10.2, 12, 13, "#e99b54", 1.7);
    strokeLine(context, 16.5, 10.2, 12, 13, "#e99b54", 1.7);
    strokeLine(context, 12, 13, 12, 18.5, "#e99b54", 1.7);
}

function drawMake(context) {
    context.strokeStyle = "#54c8d8";
    context.fillStyle = "#54c8d8";
    context.lineWidth = 2;
    path(context, [[7, 7], [15, 7], [17.5, 9.5], [15.5, 11.5],
                   [13.5, 9.5], [12.5, 10.5]], true);
    context.fill();
    strokeLine(context, 12.5, 10, 7.5, 17.5, "#54c8d8", 2.5);
}

function drawPyProject(context) {
    context.fillStyle = "#55aaff";
    context.fillRect(7, 7, 6, 5);
    context.fillRect(7, 12, 4, 5);
    context.fillStyle = "#f2c94c";
    context.fillRect(13, 12, 4, 5);
    context.fillRect(15, 9, 2, 3);
}

function drawRos(context) {
    context.fillStyle = "#68b7f7";
    const points = [[8, 8], [12, 8], [16, 8], [8, 12], [12, 12],
                    [16, 12], [8, 16], [12, 16], [16, 16]];
    for (let i = 0; i < points.length; ++i) {
        context.beginPath();
        context.arc(points[i][0], points[i][1], 1.35, 0, Math.PI * 2, false);
        context.fill();
    }
}

function drawRust(context) {
    context.strokeStyle = "#e98652";
    context.lineWidth = 1.8;
    context.beginPath();
    context.arc(12, 13, 4.3, 0, Math.PI * 2, false);
    context.stroke();
    context.beginPath();
    context.arc(12, 13, 1.5, 0, Math.PI * 2, false);
    context.stroke();
    for (let i = 0; i < 8; ++i) {
        const angle = i * Math.PI / 4;
        strokeLine(context, 12 + Math.cos(angle) * 4.8,
                   13 + Math.sin(angle) * 4.8,
                   12 + Math.cos(angle) * 6,
                   13 + Math.sin(angle) * 6, "#e98652", 1.8);
    }
}

function drawPython(context) {
    context.strokeStyle = "#5db0e6";
    context.lineWidth = 2;
    path(context, [[7, 9], [10, 12], [7, 15]], false);
    context.stroke();
    strokeLine(context, 12, 15, 17, 15, "#f2c94c", 2);
}

function drawYaml(context) {
    context.strokeStyle = "#c58cf2";
    context.lineWidth = 1.5;
    strokeLine(context, 8, 9, 12, 13, "#c58cf2", 1.5);
    strokeLine(context, 16, 9, 12, 13, "#c58cf2", 1.5);
    strokeLine(context, 12, 13, 12, 18, "#c58cf2", 1.5);
    context.fillStyle = "#c58cf2";
    const points = [[8, 9], [16, 9], [12, 13], [12, 18]];
    for (let i = 0; i < points.length; ++i) {
        context.beginPath();
        context.arc(points[i][0], points[i][1], 1.7, 0, Math.PI * 2, false);
        context.fill();
    }
}

function drawSql(context) {
    context.strokeStyle = "#61cf91";
    context.fillStyle = "#28463a";
    context.lineWidth = 1.6;
    context.beginPath();
    context.ellipse(12, 8.5, 5, 2.3, 0, 0, Math.PI * 2, false);
    context.fill();
    context.stroke();
    strokeLine(context, 7, 8.5, 7, 16.5, "#61cf91", 1.6);
    strokeLine(context, 17, 8.5, 17, 16.5, "#61cf91", 1.6);
    context.beginPath();
    context.ellipse(12, 16.5, 5, 2.3, 0, 0, Math.PI, false);
    context.stroke();
    context.beginPath();
    context.ellipse(12, 12.5, 5, 2.1, 0, 0, Math.PI, false);
    context.stroke();
}

function drawMarkdown(context) {
    context.strokeStyle = "#d6dde5";
    context.lineWidth = 1.9;
    path(context, [[6.5, 16], [6.5, 9], [9.5, 12.5], [12.5, 9],
                   [12.5, 16]], false);
    context.stroke();
    strokeLine(context, 16, 9, 16, 16, "#d6dde5", 1.9);
    path(context, [[13.8, 13.8], [16, 16.2], [18.2, 13.8]], false);
    context.stroke();
}

function drawDocker(context) {
    context.fillStyle = "#52bce8";
    context.fillRect(7, 9, 3, 3);
    context.fillRect(11, 9, 3, 3);
    context.fillRect(15, 9, 3, 3);
    context.fillRect(11, 5, 3, 3);
    context.fillRect(15, 5, 3, 3);
    path(context, [[6, 13], [18.5, 13], [17, 17], [9, 18], [6, 16]], true);
    context.fill();
}

function draw(name, context) {
    if (!name.startsWith("tree-file-")) {
        return false;
    }

    const kind = name.substring(10);
    let accent = "#91a0af";
    if (kind === "c" || kind === "cpp" || kind === "h" || kind === "hpp"
            || kind === "cmakelists" || kind === "ros") accent = "#55aaff";
    else if (kind === "cargo" || kind === "rust") accent = "#e98652";
    else if (kind === "makefile" || kind === "docker") accent = "#52bce8";
    else if (kind === "python" || kind === "pyproject") accent = "#f2c94c";
    else if (kind === "yaml") accent = "#c58cf2";
    else if (kind === "sql") accent = "#61cf91";

    document(context, accent);

    switch (kind) {
    case "c":
        letterC(context, 12, 13, 4.3, "#62b4f5", 2.2);
        break;
    case "cpp":
        letterC(context, 10, 13, 3.7, "#62b4f5", 2.1);
        plus(context, 15.4, 10.7, "#d6dde5");
        plus(context, 15.4, 15.2, "#d6dde5");
        break;
    case "h":
        letterH(context, 12, 13, "#62b4f5");
        break;
    case "hpp":
        letterH(context, 10.5, 13, "#62b4f5");
        plus(context, 16, 10.7, "#d6dde5");
        plus(context, 16, 15.2, "#d6dde5");
        break;
    case "cmakelists": drawCMake(context); break;
    case "cargo": drawCargo(context); break;
    case "makefile": drawMake(context); break;
    case "pyproject": drawPyProject(context); break;
    case "ros": drawRos(context); break;
    case "rust": drawRust(context); break;
    case "python": drawPython(context); break;
    case "yaml": drawYaml(context); break;
    case "sql": drawSql(context); break;
    case "markdown": drawMarkdown(context); break;
    case "docker": drawDocker(context); break;
    default:
        return false;
    }

    context.beginPath();
    return true;
}
