import QtQuick

// Store de diagnósticos por arquivo (fatia T6). Consome o MESMO
// event.lsp.diagnostics que alimenta a aba Problemas, mas mantém o
// range completo por arquivo para o editor (sublinhado + gutter) e a
// navegação próximo/anterior. O core é a fonte; este controller só
// guarda e deriva — funções puras, testáveis por sonda Qt Quick.
Item {
    id: root

    // path absoluto -> array de diagnósticos (shape do event.lsp.*:
    // severity, message, line, column, endLine, endColumn, code).
    property var byFile: ({})
    property string activePath: ""
    // Bump força o rebind do editor (spans + gutter) quando os
    // diagnósticos do arquivo ativo mudam. Mutar byFile no lugar não
    // notifica o QML — a revisão é o gatilho de reatividade (padrão do
    // diff gutter da M3.2).
    property int revision: 0

    // Derivados do arquivo ativo, atrelados à revisão para o editor.
    readonly property var editorSpansList: revision >= 0 ? editorSpans() : []
    readonly property var gutterMap: revision >= 0 ? gutterByLine() : ({})

    visible: false

    function handleLspDiagnostics(path, diagnostics) {
        byFile[path] = diagnostics;
        if (path === activePath) {
            revision++;
        }
    }

    function setActivePath(path) {
        if (path === activePath) {
            return;
        }
        activePath = path;
        revision++;
    }

    function clear() {
        byFile = {};
        activePath = "";
        revision++;
    }

    function activeDiagnostics() {
        const list = byFile[activePath];
        return list === undefined ? [] : list;
    }

    // Ordena por (linha, coluna) — base da navegação e da gutter.
    function sortedActive() {
        const list = activeDiagnostics().slice();
        list.sort(function(a, b) {
            const lineA = a.line !== undefined ? a.line : 0;
            const lineB = b.line !== undefined ? b.line : 0;
            if (lineA !== lineB) {
                return lineA - lineB;
            }
            const colA = a.column !== undefined ? a.column : 0;
            const colB = b.column !== undefined ? b.column : 0;
            return colA - colB;
        });
        return list;
    }

    // Spans para o highlighter (0-based UTF-16). endLine/endColumn caem
    // de volta ao início quando ausentes.
    function editorSpans() {
        const list = activeDiagnostics();
        const spans = [];
        for (let i = 0; i < list.length; i++) {
            const diagnostic = list[i];
            if (diagnostic.line === undefined || diagnostic.column === undefined) {
                continue;
            }
            const endLine = diagnostic.endLine !== undefined
                    ? diagnostic.endLine : diagnostic.line;
            const endColumn = diagnostic.endColumn !== undefined
                    ? diagnostic.endColumn : diagnostic.column;
            spans.push({
                startLine: diagnostic.line - 1,
                startChar: diagnostic.column - 1,
                endLine: endLine - 1,
                endChar: endColumn - 1,
                severity: diagnostic.severity !== undefined
                          ? diagnostic.severity : "error"
            });
        }
        return spans;
    }

    function severityRank(severity) {
        if (severity === "error") {
            return 3;
        }
        if (severity === "warning") {
            return 2;
        }
        return 1;
    }

    // linha (1-based) -> { severity (a mais forte), message (juntada) }
    // para a marca e o tooltip da gutter.
    function gutterByLine() {
        const list = activeDiagnostics();
        const map = ({});
        for (let i = 0; i < list.length; i++) {
            const diagnostic = list[i];
            if (diagnostic.line === undefined) {
                continue;
            }
            const line = diagnostic.line;
            const severity = diagnostic.severity !== undefined
                    ? diagnostic.severity : "error";
            const message = diagnostic.message !== undefined
                    ? diagnostic.message : "";
            const entry = map[line];
            if (entry === undefined) {
                map[line] = { severity: severity, message: message };
            } else {
                if (severityRank(severity) > severityRank(entry.severity)) {
                    entry.severity = severity;
                }
                entry.message = entry.message + "\n" + message;
            }
        }
        return map;
    }

    function diagnosticCount() {
        return activeDiagnostics().length;
    }

    // Próximo diagnóstico depois de (line, column), com wrap. Devolve
    // { line, column } ou null quando não há nenhum.
    function nextDiagnostic(line, column) {
        const list = sortedActive();
        if (list.length === 0) {
            return null;
        }
        for (let i = 0; i < list.length; i++) {
            const diagnostic = list[i];
            const dl = diagnostic.line !== undefined ? diagnostic.line : 0;
            const dc = diagnostic.column !== undefined ? diagnostic.column : 0;
            if (dl > line || (dl === line && dc > column)) {
                return { line: dl, column: dc };
            }
        }
        return {
            line: list[0].line !== undefined ? list[0].line : 1,
            column: list[0].column !== undefined ? list[0].column : 1
        };
    }

    // Diagnóstico anterior a (line, column), com wrap.
    function prevDiagnostic(line, column) {
        const list = sortedActive();
        if (list.length === 0) {
            return null;
        }
        for (let i = list.length - 1; i >= 0; i--) {
            const diagnostic = list[i];
            const dl = diagnostic.line !== undefined ? diagnostic.line : 0;
            const dc = diagnostic.column !== undefined ? diagnostic.column : 0;
            if (dl < line || (dl === line && dc < column)) {
                return { line: dl, column: dc };
            }
        }
        const last = list[list.length - 1];
        return {
            line: last.line !== undefined ? last.line : 1,
            column: last.column !== undefined ? last.column : 1
        };
    }
}
