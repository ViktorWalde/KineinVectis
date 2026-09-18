import QtQuick

// Estado da sessao de debug e dos breakpoints na UI (fatia M2.5b).
// A UI nunca fala DAP: o core mastiga tudo em event.debug.* e este
// controller so guarda estado e emite intencoes (padrao dos controllers).
Item {
    id: root

    property string workspaceRoot: ""
    property bool sessionActive: false
    property bool starting: false
    property bool attached: false
    property bool paused: false
    property string currentFile: ""
    property int currentLine: 0
    property alias outputModel: debugOutputModel
    property alias framesModel: debugFramesModel
    property alias variablesModel: debugVariablesModel
    property alias watchesModel: debugWatchesModel
    property int currentFrameIndex: -1
    property real currentFrameId: -1
    // Breakpoints por arquivo (objeto js file -> [linhas]); a revisao força
    // os bindings do gutter a reavaliarem apos cada mutacao.
    property var breakpointsByFile: ({})
    property int breakpointsRevision: 0

    signal startRequested(string program, var connect)
    signal stopRequested()
    signal continueRequested()
    signal nextRequested()
    signal stepInRequested()
    signal stepOutRequested()
    signal pauseRequested()
    signal setBreakpointsRequested(string file, var breakpoints)
    signal evaluateRequested(string expression, real frameId)
    signal stackTraceRequested()
    signal frameVariablesRequested(real frameId)
    signal variablesByRefRequested(real ref)
    signal showTabRequested(string tab)
    signal openAtRequested(string file, int line)

    // P3: escopos/memoria/disassembly — filho, sem propriedade de repasse.
    readonly property alias inspect: inspectController

    visible: false

    DebugInspectController {
        id: inspectController

        frameId: root.currentFrameId
    }

    ListModel { id: debugOutputModel }
    ListModel { id: debugFramesModel }
    ListModel { id: debugVariablesModel }

    // Watches: { expression, value, failed }. Sobrevivem ao `continue` de
    // proposito — a expressao e' do usuario, nao da parada. O que morre e' o
    // VALOR, reavaliado na parada seguinte.
    ListModel { id: debugWatchesModel }

    function clearInspection() {
        debugFramesModel.clear();
        debugVariablesModel.clear();
        currentFrameIndex = -1;
        currentFrameId = -1;
        // O valor do watch morre com a parada; a expressao nao.
        for (let i = 0; i < debugWatchesModel.count; i++) {
            debugWatchesModel.setProperty(i, "value", "");
            debugWatchesModel.setProperty(i, "failed", false);
        }
    }

    function addWatch(expression) {
        const limpo = (expression || "").trim();
        if (limpo === "") {
            return;
        }
        debugWatchesModel.append({ expression: limpo, value: "", failed: false });
        if (paused) {
            evaluateRequested(limpo, currentFrameId);
        }
    }

    function removeWatch(index) {
        if (index >= 0 && index < debugWatchesModel.count) {
            debugWatchesModel.remove(index);
        }
    }

    // Reavalia TODOS os watches. Chamado a cada parada e a cada troca de
    // frame: o valor de uma expressao depende do frame, e mostrar o valor do
    // frame anterior seria pior que nao mostrar nada.
    function refreshWatches() {
        if (!paused) {
            return;
        }
        for (let i = 0; i < debugWatchesModel.count; i++) {
            evaluateRequested(debugWatchesModel.get(i).expression, currentFrameId);
        }
    }

    function handleEvaluated(expression, value, typeName, ref) {
        for (let i = 0; i < debugWatchesModel.count; i++) {
            if (debugWatchesModel.get(i).expression === expression) {
                debugWatchesModel.setProperty(i, "value",
                                              typeName === "" ? value : value + "  (" + typeName + ")");
                debugWatchesModel.setProperty(i, "failed", false);
            }
        }
    }

    function handleEvaluateFailed(expression, message) {
        for (let i = 0; i < debugWatchesModel.count; i++) {
            if (debugWatchesModel.get(i).expression === expression) {
                debugWatchesModel.setProperty(i, "value", message);
                debugWatchesModel.setProperty(i, "failed", true);
            }
        }
    }

    function clear() {
        clearInspection();
        inspectController.clear();
        debugOutputModel.clear();
        starting = false;
        attached = false;
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

    // A sarjeta do editor so' quer as LINHAS — este contrato nao mudou quando
    // o breakpoint ganhou condicao (protocolo 0.66.0).
    function breakpointLinesFor(file) {
        return breakpointsFor(file).map(function(bp) { return bp.line; });
    }

    // Os breakpoints inteiros: { line, condition, hitCondition }.
    function breakpointsFor(file) {
        const items = breakpointsByFile[file];
        return items === undefined ? [] : items;
    }

    function breakpointAt(file, line) {
        return breakpointsFor(file).find(function(bp) { return bp.line === line; }) || null;
    }

    // Condicao vazia REMOVE a condicao em vez de mandar string vazia: o core
    // ja' descarta so'-espacos, e guardar "" aqui faria a UI mostrar um
    // breakpoint como condicional quando ele nao e'.
    function setBreakpointCondition(file, line, condition, hitCondition) {
        const items = breakpointsFor(file).slice();
        const index = items.findIndex(function(bp) { return bp.line === line; });
        if (index < 0) {
            return;
        }
        const limpo = (condition || "").trim();
        const limpoHit = (hitCondition || "").trim();
        items[index] = {
            line: line,
            condition: limpo === "" ? undefined : limpo,
            hitCondition: limpoHit === "" ? undefined : limpoHit
        };
        breakpointsByFile[file] = items;
        breakpointsRevision++;
        setBreakpointsRequested(file, items);
    }

    function toggleBreakpoint(file, line) {
        if (file === "" || line <= 0) {
            return;
        }
        const items = breakpointsFor(file).slice();
        const index = items.findIndex(function(bp) { return bp.line === line; });
        if (index >= 0) {
            items.splice(index, 1);
        } else {
            items.push({ line: line });
            items.sort(function(a, b) { return a.line - b.line; });
        }
        if (items.length === 0) {
            delete breakpointsByFile[file];
        } else {
            breakpointsByFile[file] = items;
        }
        breakpointsRevision++;
        setBreakpointsRequested(file, items);
    }

    // One start intent for automatic targets, files and TCP attach.
    function startDebug(program, connect) {
        if (workspaceRoot === "" || sessionActive || starting) {
            return;
        }
        starting = true;
        showTabRequested("debug");
        startRequested(program || "", connect || {});
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

    function handleStarted(program, isAttached) {
        starting = false;
        attached = isAttached === true;
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
        // O valor de um watch depende do FRAME: trocar de frame sem reavaliar
        // mostraria o valor do frame anterior, que e pior que nao mostrar.
        refreshWatches();
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
        starting = false;
        attached = false;
        sessionActive = false;
        paused = false;
        currentFile = "";
        currentLine = 0;
        clearInspection();
        inspectController.clear();
        appendLine(qsTr("== debug finalizado (codigo %1) ==").arg(exitCode),
                   exitCode === 0 ? "info" : "stderr");
    }

    // `adapter` (0.111.0) e' o stderr do PROPRIO adaptador — o traceback de
    // um debugpy que morreu, o aviso do lldb-dap — e veste stderr: e' erro
    // de quem depura, nao saida do programa.
    function handleOutput(category, line) {
        appendLine(line, category === "stderr" || category === "adapter" ? "stderr"
                   : (category === "console" ? "info" : "stdout"));
    }

    function handleRequestFailed(method, message) {
        if (method === "debug.start") {
            starting = false;
        }
        if (method.startsWith("debug.")) {
            showTabRequested("debug");
            appendLine(message, "stderr");
        }
    }
}
