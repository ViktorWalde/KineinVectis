// A FAMILIA DE ICONES da IDE (0.3.9, reformulacao pedida pelo autor: "icons
// melhores visualmente"; antes "estao muito fracos"). Um desenho por nome, em
// SVG path, no grid 24x24 — o KvIcon escala para 16, 20, 24 ou 28 px e pinta
// com a cor do Theme.
//
// As regras da familia (DocsPublic/iconografia/README.md, "Familia da casca"):
//
//   area viva   2..22 (2 px de respiro); o desenho ocupa a area, nao um canto
//   traco       2.0 no grid de 24, ponta e juncao redondas (o KvIcon fixa)
//   cantos      retangulos com raio 1.5..2.5; nada de quina viva
//   peso        uma forma CHEIA so' onde ela e' o signo: executar, parar, a
//               cabeca do martelo, os pontos do "mais", da estrutura e da placa
//   ponto       "h.01" com ponta redonda: um ponto do tamanho do traco
//   metafora    uma por acao, sem repetir entre areas vizinhas do trilho
//
// `shape(name)` devolve { stroke, fill } (fill pode ser "") ou null para quem
// nao e' desta familia (os tipos de arquivo moram no KvFileIconGlyphs.js).
// Nenhum desenho copia marca registrada nem conjunto de terceiros.
.pragma library

// Circulo como path: dois arcos de meia volta.
function circle(cx, cy, r) {
    return "M" + (cx - r) + " " + cy + "a" + r + " " + r + " 0 1 0 " + (2 * r) + " 0"
        + "a" + r + " " + r + " 0 1 0 " + (-2 * r) + " 0";
}

// Retangulo de cantos redondos.
function roundRect(x, y, w, h, r) {
    return "M" + (x + r) + " " + y + "h" + (w - 2 * r)
        + "a" + r + " " + r + " 0 0 1 " + r + " " + r + "v" + (h - 2 * r)
        + "a" + r + " " + r + " 0 0 1 " + (-r) + " " + r + "h" + (-(w - 2 * r))
        + "a" + r + " " + r + " 0 0 1 " + (-r) + " " + (-r) + "v" + (-(h - 2 * r))
        + "a" + r + " " + r + " 0 0 1 " + r + " " + (-r) + "z";
}

// Engrenagem: aro, furo e oito dentes curtos.
function gear(cx, cy) {
    let teeth = "";
    for (let i = 0; i < 8; i++) {
        const angle = (i * 45 + 22.5) * Math.PI / 180;
        teeth += "M" + (cx + 6 * Math.cos(angle)).toFixed(2) + " " + (cy + 6 * Math.sin(angle)).toFixed(2)
            + "L" + (cx + 8.6 * Math.cos(angle)).toFixed(2) + " " + (cy + 8.6 * Math.sin(angle)).toFixed(2);
    }
    return circle(cx, cy, 6) + circle(cx, cy, 2.4) + teeth;
}

const fileOutline = "M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z M14 3v5h5";
const folderOutline = "M3 7a2 2 0 0 1 2-2h4.2l2 2.2H19a2 2 0 0 1 2 2V18a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z";
const branchShape = circle(6, 5.5, 2.5) + circle(6, 18.5, 2.5) + circle(18, 8, 2.5)
    + "M6 8v8 M18 10.5C18 14.5 14 16 8.3 17.4";
const eyeOutline = "M2.5 12C4.6 7.6 8 5.2 12 5.2C16 5.2 19.4 7.6 21.5 12"
    + "C19.4 16.4 16 18.8 12 18.8C8 18.8 4.6 16.4 2.5 12Z";
const warningShape = "M10.3 4.1a2 2 0 0 1 3.4 0l7.5 13a2 2 0 0 1-1.7 3H4.5a2 2 0 0 1-1.7-3z"
    + " M12 9.5v4 M12 17h.01";
const hammerHead = "M13.73 3.91L20.09 10.27L17.27 13.09L10.91 6.73Z";
const runShape = "M7 4.8v14.4a1 1 0 0 0 1.5.86l11.3-7.2a1 1 0 0 0 0-1.72L8.5 3.94A1 1 0 0 0 7 4.8z";
const stopShape = roundRect(6, 6, 12, 12, 2.5);

const shapes = {
    // Acoes basicas
    "add": { stroke: "M12 5v14M5 12h14", fill: "" },
    "back": { stroke: "M19 12H5M11 18l-6-6 6-6", fill: "" },
    "check": { stroke: "M5 12.5l4.5 4.5L19 7.5", fill: "" },
    "chevron-down": { stroke: "M6 9l6 6 6-6", fill: "" },
    "chevron-up": { stroke: "M6 15l6-6 6 6", fill: "" },
    "close": { stroke: "M6 6l12 12M18 6L6 18", fill: "" },
    // Remover (lixeira) e abrir fora (o navegador): as acoes dos Containers.
    "trash": { stroke: "M4 7h16 M9.5 7V4.5h5V7 M6.5 7l.9 12.6A1.5 1.5 0 0 0 8.9 21h6.2a1.5 1.5 0 0 0 1.5-1.4L17.5 7"
                       + " M10.25 11v6M13.75 11v6", fill: "" },
    "external": { stroke: "M14 4h6v6M20 4l-9 9 M18 14v4.5a1.5 1.5 0 0 1-1.5 1.5h-11A1.5 1.5 0 0 1 4 18.5v-11"
                          + "A1.5 1.5 0 0 1 5.5 6H10", fill: "" },
    "expand": { stroke: "M15 4h5v5M9 20H4v-5M20 4l-6 6M4 20l6-6", fill: "" },
    "collapse": { stroke: "M4 14h6v6M20 10h-6V4M14 10l6-6M4 20l6-6", fill: "" },
    "menu": { stroke: "M4 6.5h16M4 12h16M4 17.5h16", fill: "" },
    // 2026-10-03: os itens de menu sem desenho (os menus ganharam icone).
    "save": { stroke: "M5.5 5.5A1.5 1.5 0 0 1 7 4h9l3.5 3.5V19a1.5 1.5 0 0 1-1.5 1.5H7A1.5 1.5 0 0 1 5.5 19z"
                      + " M8.5 4v4.5h6V4 M8.5 20.5v-6h7v6", fill: "" },
    "rename": { stroke: "M4 20h4L19 9a2.1 2.1 0 0 0-3-3L5 17z M14 8l3 3", fill: "" },
    "goto": { stroke: "M4 12h11 M11 8l4 4-4 4 M20 5v14", fill: "" },
    "undo": { stroke: "M9 14L4 9l5-5 M4 9h10.5a5.5 5.5 0 0 1 0 11H11", fill: "" },
    "copy": { stroke: "M9 9h10.5v10.5H9z M15 9V5.5A1.5 1.5 0 0 0 13.5 4h-8A1.5 1.5 0 0 0 4 5.5v8"
                      + "A1.5 1.5 0 0 0 5.5 15H9", fill: "" },
    "paste": { stroke: "M8 5H6.5A1.5 1.5 0 0 0 5 6.5v13A1.5 1.5 0 0 0 6.5 21h11a1.5 1.5 0 0 0 1.5-1.5v-13"
                       + "A1.5 1.5 0 0 0 17.5 5H16 M9 3.5h6V7H9z", fill: "" },
    "cut": { stroke: circle(6.5, 17.5, 2.5) + circle(17.5, 17.5, 2.5) + "M8.4 15.8L18 4 M15.6 15.8L6 4", fill: "" },
    "more": { stroke: "", fill: circle(6, 12, 1.7) + circle(12, 12, 1.7) + circle(18, 12, 1.7) },
    "refresh": { stroke: "M20 12A8 8 0 1 1 17.65 6.35L20 8.6 M20 4.5v4.1h-4.1", fill: "" },
    "search": { stroke: circle(10.5, 10.5, 6.5) + "M15.5 15.5L20 20", fill: "" },
    "pin": { stroke: "M9 3h6M10 3.5V9l-3 4v1.5h10V13l-3-4V3.5M12 14.5V21", fill: "" },
    "eye": { stroke: eyeOutline + circle(12, 12, 3), fill: "" },
    "eye-off": { stroke: eyeOutline + "M4 4l16 16", fill: "" },
    "help": { stroke: circle(12, 12, 9) + "M9.6 9.3a2.5 2.5 0 0 1 4.8 1c0 1.7-2.4 2.2-2.4 3.7 M12 17.2h.01",
              fill: "" },
    "settings": { stroke: "M4 7h8M16 7h4M4 17h4M12 17h8" + circle(14, 7, 2) + circle(10, 17, 2), fill: "" },
    "configure": { stroke: gear(12, 12), fill: "" },

    // Janela
    "minimize": { stroke: "M6 12h12", fill: "" },
    "maximize": { stroke: roundRect(5, 5, 14, 14, 1.5), fill: "" },
    "restore": { stroke: roundRect(4.5, 8, 11.5, 11.5, 1.5)
                         + "M8 8V6a1.5 1.5 0 0 1 1.5-1.5H18a1.5 1.5 0 0 1 1.5 1.5v8.5A1.5 1.5 0 0 1 18 16h-2",
                 fill: "" },

    // Projeto e arquivos
    "file": { stroke: fileOutline, fill: "" },
    "documents": { stroke: fileOutline + " M9 13h6M9 17h4", fill: "" },
    "folder": { stroke: folderOutline, fill: "" },
    "project": { stroke: folderOutline, fill: "" },
    // Os Simbolos: a estrutura do arquivo (pais e filhos), nao uma folha.
    "outline": { stroke: "M8.5 6H20M12.5 12H20M12.5 18H20",
                 fill: circle(4.5, 6, 1.5) + circle(8.5, 12, 1.5) + circle(8.5, 18, 1.5) },
    "home": { stroke: "M3.5 10.5L12 3.5l8.5 7 M5.5 9v10a1.5 1.5 0 0 0 1.5 1.5h10a1.5 1.5 0 0 0 1.5-1.5V9"
                      + " M10 20.5V15h4v5.5", fill: "" },
    "desktop": { stroke: roundRect(3, 4, 18, 12.5, 2) + "M12 16.5V20M8 20.5h8", fill: "" },
    "download": { stroke: "M12 4v11M7.5 10.5L12 15l4.5-4.5M5 20h14", fill: "" },
    "drive": { stroke: "M6.2 5h11.6a1.5 1.5 0 0 1 1.4 1l1.8 7v5a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-5l1.8-7"
                       + "a1.5 1.5 0 0 1 1.4-1z M3 13h18 M7 16.5h.01M10 16.5h.01", fill: "" },
    "recent": { stroke: circle(12, 12, 9) + "M12 7.5V12l3 2", fill: "" },

    // Git
    "git": { stroke: branchShape, fill: "" },
    "branch": { stroke: branchShape, fill: "" },
    // Receber e enviar: setas na diagonal, para nao confundir com baixar.
    "pull": { stroke: "M18 6L7 17M7 9v8h8M4 21h16", fill: "" },
    "push": { stroke: "M6 18L17 7M9 7h8v8M4 3h16", fill: "" },

    // Build, execucao e qualidade
    "build": { stroke: hammerHead + " M14.09 9.91L4.5 19.5", fill: hammerHead },
    "run": { stroke: runShape, fill: runShape },
    "stop": { stroke: stopShape, fill: stopShape },
    "debug": { stroke: "M8 13a4 4 0 0 1 8 0v3a4 4 0 0 1-8 0z M9.5 9.4a2.5 2.5 0 0 1 5 0 M12 12.5V20"
                       + " M8 13.5H4.5M16 13.5h3.5 M8.3 10.5L5.5 8.5M15.7 10.5l2.8-2"
                       + " M8.4 17.5l-2.9 2M15.6 17.5l2.9 2", fill: "" },
    "test": { stroke: "M9 3h6 M10 3v6.2L4.7 18.4A1.7 1.7 0 0 0 6.2 21h11.6a1.7 1.7 0 0 0 1.5-2.6L14 9.2V3"
                      + " M7.2 15h9.6", fill: "" },
    "terminal": { stroke: roundRect(3, 4, 18, 16, 2.5) + "M7 9.5l3 2.5-3 2.5M12.5 15H17", fill: "" },
    "problems": { stroke: circle(12, 12, 9) + "M12 7.5V13 M12 16.5h.01", fill: "" },
    "warning": { stroke: warningShape, fill: "" },
    // Ferramentas: a caixa de ferramentas (compiladores, depuradores, analise).
    "tools": { stroke: "M4 9h16v9.5a1.5 1.5 0 0 1-1.5 1.5h-13A1.5 1.5 0 0 1 4 18.5z"
                       + " M9 9V6.5A1.5 1.5 0 0 1 10.5 5h3A1.5 1.5 0 0 1 15 6.5V9 M4 13.5h16 M11 13.5v2h2v-2",
               fill: "" },

    // Ambiente do projeto
    // O navegador do Banco: o esquema (camadas) e a tabela (grade).
    "schema": { stroke: "M12 3l9 4.5-9 4.5-9-4.5z M3 12l9 4.5 9-4.5 M3 16.5l9 4.5 9-4.5", fill: "" },
    "table": { stroke: roundRect(3, 4, 18, 16, 2) + "M3 9.5h18M3 14.75h18M9.5 9.5V20", fill: "" },
    "database": { stroke: "M4 6a8 3 0 1 0 16 0a8 3 0 1 0 -16 0 M4 6v12a8 3 0 0 0 16 0V6 M4 12a8 3 0 0 0 16 0",
                  fill: "" },
    "container": { stroke: "M12 3l8 4.5v9L12 21l-8-4.5v-9z M4 7.5l8 4.5 8-4.5 M12 12v9", fill: "" },
    "observability": { stroke: "M3 12h3.5l2.5-6 4 12 2.5-6H21", fill: "" },
    "remote": { stroke: roundRect(3, 3.5, 18, 7, 2) + roundRect(3, 13.5, 18, 7, 2)
                        + "M7 7h.01M7 17h.01M11 7h6M11 17h6", fill: "" },
    "cpu": { stroke: roundRect(5, 5, 14, 14, 2.5) + roundRect(9, 9, 6, 6, 1)
                     + "M9.5 2v3M14.5 2v3M9.5 19v3M14.5 19v3M2 9.5h3M2 14.5h3M19 9.5h3M19 14.5h3", fill: "" },
    // Embarcados: a placa, com o chip e a fileira de pinos.
    "embedded": { stroke: roundRect(3, 5, 18, 14, 2.5) + roundRect(7, 9, 6, 6, 1),
                  fill: circle(17, 9, 1.1) + circle(17, 12, 1.1) + circle(17, 15, 1.1) }
};

function shape(name) {
    return shapes[name] !== undefined ? shapes[name] : null;
}

// Os nomes da familia, para a galeria e para o harness.
function names() {
    return Object.keys(shapes);
}
