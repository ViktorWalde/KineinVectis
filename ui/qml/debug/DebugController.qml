import QtQuick

// Estado da sessao de debug e dos breakpoints na UI (fatia M2.5b).
// A UI nunca fala DAP: o core mastiga tudo em event.debug.* e este
// controller so guarda estado e emite intencoes (padrao dos controllers).
Item {
    id: root

    property string workspaceRoot: ""
    property bool sessionActive: false
    property bool paused: false
    property string currentFile: ""
    property int currentLine: 0
    property alias outputModel: debugOutputModel
    property alias framesModel: debugFramesModel
    property alias variablesModel: debugVariablesModel
    property int currentFrameIndex: -1
    property real currentFrameId: -1
    // Breakpoints por arquivo (objeto js file -> [linhas]); a revisao força
    // os bindings do gutter a reavaliarem apos cada mutacao.
    property var breakpointsByFile: ({})
    property int breakpointsRevision: 0

    signal startRequested(string program)
    signal stopRequested()
    signal continueRequested()
    signal nextRequested()
    signal stepInRequested()
    signal stepOutRequested()
    signal pauseRequested()
    signal setBreakpointsRequested(string file, var lines)
    signal stackTraceRequested()
    signal frameVariablesRequested(real frameId)
    signal variablesByRefRequested(real ref)
    signal showTabRequested(string tab)
    signal openAtRequested(string file, int line)

    visible: false

    ListModel {
        id: debugOutputModel
    }

    ListModel {
        id: debugFramesModel
    }

    ListModel {
        id: debugVariablesModel
    }

    function clearInspection() {
        debugFramesModel.clear();
        debugVariablesModel.clear();
        currentFrameIndex = -1;
        currentFrameId = -1;
    }

    function clear() {
        clearInspection();
        debugOutputModel.clear();
        sessionActive = false;
        paused = false;
        currentFile = "";
        currentLine = 0;
        breakpointsByFile = {};
        breakpointsRevision++;
    }

    function appendLine(line, kind) {
        debugOutputModel.append({ line: line, kind: kind });
        if (debugOutputModel.count > 2000) {
            debugOutputModel.remove(0);
        }
    }

    function breakpointLinesFor(file) {
        const lines = breakpointsByFile[file];
        return lines === undefined ? [] : lines;
    }

    function toggleBreakpoint(file, line) {
        if (file === "" || line <= 0) {
            return;
        }
        const lines = breakpointLinesFor(file).slice();
        const index = lines.indexOf(line);
        if (index >= 0) {
            lines.splice(index, 1);
        } else {
            lines.push(line);
            lines.sort(function(a, b) { return a - b; });
        }
        if (lines.length === 0) {
            delete breakpointsByFile[file];
        } else {
            breakpointsByFile[file] = lines;
        }
        breakpointsRevision++;
        setBreakpointsRequested(file, lines);
    }

    function startDebug() {
        if (workspaceRoot === "" || sessionActive) {
            return;
        }
        showTabRequested("debug");
        startRequested("");
    }

    function stopDebug() {
        if (sessionActive) {
            stopRequested();
        }
    }

    function continueDebug() {
        if (sessionActive && paused) {
            continueRequested();
        }
    }

    function stepOver() {
        if (sessionActive && paused) {
            nextRequested();
        }
    }

    function stepInto() {
        if (sessionActive && paused) {
            stepInRequested();
        }
    }

    function stepOutOf() {
        if (sessionActive && paused) {
            stepOutRequested();
        }
    }

    function pauseDebug() {
        if (sessionActive && !paused) {
            pauseRequested();
        }
    }

    function handleStarted(program) {
        sessionActive = true;
        paused = false;
        currentFile = "";
        currentLine = 0;
        appendLine(qsTr("== debug iniciado: %1 ==").arg(program), "command");
    }

    // Parou → stack automatico; o frame do topo puxa as variaveis.
    function handleStackTrace(frames) {
        debugFramesModel.clear();
        for (let i = 0; i < frames.length; i++) {
            debugFramesModel.append({
                frameId: frames[i].id,
                name: frames[i].name,
                file: frames[i].file !== undefined ? frames[i].file : "",
                line: frames[i].line !== undefined ? frames[i].line : 0
            });
        }
        if (debugFramesModel.count > 0) {
            selectFrame(0, false);
        }
    }

    function selectFrame(index, navigate) {
        if (index < 0 || index >= debugFramesModel.count) {
            return;
        }
        currentFrameIndex = index;
        const frame = debugFramesModel.get(index);
        currentFrameId = frame.frameId;
        debugVariablesModel.clear();
        frameVariablesRequested(frame.frameId);
        if (navigate && frame.file !== "" && frame.line > 0) {
            openAtRequested(frame.file, frame.line);
        }
    }

    function handleVariables(frameId, ref, variables) {
        if (frameId >= 0) {
            if (frameId !== currentFrameId) {
                return;
            }
            debugVariablesModel.clear();
            for (let i = 0; i < variables.length; i++) {
                debugVariablesModel.append(variableRow(variables[i], 0));
            }
            return;
        }
        if (ref <= 0) {
            return;
        }
        for (let j = 0; j < debugVariablesModel.count; j++) {
            const item = debugVariablesModel.get(j);
            if (item.reference === ref && !item.expanded) {
                debugVariablesModel.setProperty(j, "expanded", true);
                for (let k = 0; k < variables.length; k++) {
                    debugVariablesModel.insert(
                        j + 1 + k, variableRow(variables[k], item.depth + 1));
                }
                return;
            }
        }
    }

    function variableRow(variable, depth) {
        return {
            name: variable.name,
            value: variable.value,
            typeName: variable.type !== undefined ? variable.type : "",
            reference: variable.ref,
            depth: depth,
            expanded: false
        };
    }

    function toggleVariable(index) {
        if (index < 0 || index >= debugVariablesModel.count) {
            return;
        }
        const item = debugVariablesModel.get(index);
        if (item.reference <= 0) {
            return;
        }
        if (item.expanded) {
            while (index + 1 < debugVariablesModel.count
                   && debugVariablesModel.get(index + 1).depth > item.depth) {
                debugVariablesModel.remove(index + 1);
            }
            debugVariablesModel.setProperty(index, "expanded", false);
            return;
        }
        variablesByRefRequested(item.reference);
    }

    function handleStopped(reason, file, line) {
        paused = true;
        currentFile = file !== undefined ? file : "";
        currentLine = line !== undefined ? line : 0;
        if (currentFile !== "" && currentLine > 0) {
            openAtRequested(currentFile, currentLine);
            appendLine(qsTr("== pausado (%1) em %2:%3 ==")
                           .arg(reason).arg(currentFile).arg(currentLine),
                       "info");
        } else {
            appendLine(qsTr("== pausado (%1) ==").arg(reason), "info");
        }
        stackTraceRequested();
    }

    function handleContinued() {
        paused = false;
        currentFile = "";
        currentLine = 0;
        clearInspection();
    }

    function handleFinished(exitCode) {
        sessionActive = false;
        paused = false;
        currentFile = "";
        currentLine = 0;
        clearInspection();
        appendLine(qsTr("== debug finalizado (codigo %1) ==").arg(exitCode),
                   exitCode === 0 ? "info" : "stderr");
    }

    function handleOutput(category, line) {
        appendLine(line, category === "stderr" ? "stderr"
                   : (category === "console" ? "info" : "stdout"));
    }

    function handleRequestFailed(method, message) {
        if (method.startsWith("debug.")) {
            showTabRequested("debug");
            appendLine(message, "stderr");
        }
    }
}
