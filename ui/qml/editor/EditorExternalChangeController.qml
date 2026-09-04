import QtQuick

// O QUE O DISCO FEZ PELAS COSTAS DA IDE.
//
// POR QUE ESTE ARQUIVO EXISTE (2026-09-03). O EditorDocumentController tinha
// 548 linhas e QUATRO vocabularios misturados: abas, buffers, salvamento e
// conflito externo. Este e' o quarto, e e' o unico com uma MAQUINA DE ESTADO
// propria — "mudou no disco / foi apagado / o local ainda e' o salvo / o local
// divergiu" — com estado proprio (`pendingExternalReads` e as tres
// propriedades `current*`).
//
// O corte e' por RESPONSABILIDADE (ARCHITECTURE.md §4 regra 9): depois dele,
// `grep -ci "external" ui/qml/editor/EditorDocumentController.qml` so' encontra
// as DELEGACOES, nenhuma regra.
//
// A REGRA CENTRAL, que e' facil de errar e por isso esta escrita aqui:
// mudanca no disco NAO e' conflito. So' vira conflito quando o buffer local
// tambem mudou. As tres saidas de `applyExternalRead` sao, nesta ordem:
//
//   1. o conteudo do disco e' o que a IDE acabou de gravar (ou ja e' o salvo)
//      -> nao houve mudanca de verdade, limpa o estado e sai;
//   2. o buffer local ainda e' identico ao salvo -> ninguem perde nada,
//      recarrega em silencio preservando a posicao do cursor;
//   3. os dois lados mudaram -> CONFLITO; guarda o conteudo externo e deixa a
//      decisao para o autor (recarregar ou manter o local).
//
// COMO CONVERSA. Recebe o `filesModel` e o `surfaceBridge` do dono; devolve
// pedidos por SINAL (`readFileRequested`, `documentChanged`). Nao chama de
// volta para o EditorDocumentController — nao ha ciclo entre os dois.
Item {
    id: root

    visible: false

    // O ListModel de abas abertas, do EditorDocumentController.
    property var filesModel: null
    property int currentTab: -1
    property var surfaceBridge: null
    // Snapshots das gravacoes em voo, do dono. Sem isto, a notificacao de
    // mudanca disparada pela PROPRIA gravacao da IDE viraria conflito falso.
    property var pendingSaves: ({})

    // Leituras pedidas por causa de mudanca externa (path -> mensagem de
    // falha, ou "" quando a leitura e' so' para comparar).
    property var pendingExternalReads: ({})

    // Estado externo da ABA ATUAL, espelhado para a UI desenhar a faixa.
    property bool currentConflict: false
    property bool currentDeleted: false
    property string currentMessage: ""

    signal readFileRequested(string path)
    signal documentChanged()

    function surfaceReady() {
        return surfaceBridge !== null && surfaceBridge.ready();
    }

    function reset() {
        pendingExternalReads = {};
        syncCurrent();
    }

    function syncCurrent() {
        if (filesModel === null || currentTab < 0
                || currentTab >= filesModel.count) {
            currentConflict = false;
            currentDeleted = false;
            currentMessage = "";
            return;
        }
        const document = filesModel.get(currentTab);
        currentConflict = document.externalConflict === true;
        currentDeleted = document.externalDeleted === true;
        currentMessage = document.externalMessage || "";
    }

    function clearAt(index) {
        filesModel.setProperty(index, "externalContent", "");
        filesModel.setProperty(index, "externalConflict", false);
        filesModel.setProperty(index, "externalDeleted", false);
        filesModel.setProperty(index, "externalMessage", "");
        if (index === currentTab) {
            syncCurrent();
        }
    }

    // Marca conflito com uma mensagem propria (usado tambem quando a
    // GRAVACAO falha) e pede a releitura do disco para comparar.
    function markConflictAt(index, message) {
        filesModel.setProperty(index, "externalConflict", true);
        filesModel.setProperty(index, "externalMessage", message);
        if (index === currentTab) {
            syncCurrent();
        }
    }

    function queueRead(path, message) {
        const reads = pendingExternalReads;
        reads[path] = message || "";
        pendingExternalReads = reads;
        readFileRequested(path);
    }

    function isPendingRead(path) {
        return pendingExternalReads[path] !== undefined;
    }

    // Notificacao do watcher: para cada arquivo aberto que mudou, ou marca
    // apagado, ou pede a leitura que vai decidir se houve conflito.
    function noteChanges(changes) {
        for (let changeIndex = 0; changeIndex < changes.length; changeIndex++) {
            const change = changes[changeIndex];
            for (let i = 0; i < filesModel.count; i++) {
                if (filesModel.get(i).path !== change.path) {
                    continue;
                }
                if (change.kind === "deleted") {
                    filesModel.setProperty(i, "externalConflict", true);
                    filesModel.setProperty(i, "externalDeleted", true);
                    filesModel.setProperty(i, "externalMessage",
                            qsTr("O arquivo foi removido fora da IDE; o buffer local foi preservado."));
                    filesModel.setProperty(i, "modified", true);
                    if (i === currentTab) {
                        syncCurrent();
                    }
                } else {
                    queueRead(change.path, "");
                }
                break;
            }
        }
    }

    // Chegou o conteudo de uma leitura que ESTE controller pediu. As tres
    // saidas estao descritas no cabecalho do arquivo.
    function applyExternalRead(path, content) {
        const reads = pendingExternalReads;
        const failureMessage = reads[path];
        delete reads[path];
        pendingExternalReads = reads;
        for (let i = 0; i < filesModel.count; i++) {
            if (filesModel.get(i).path !== path) {
                continue;
            }
            const document = filesModel.get(i);
            const pendingSave = pendingSaves[path];
            if ((pendingSave !== undefined && content === pendingSave)
                    || content === document.savedContent) {
                clearAt(i);
                return;
            }
            const localContent = i === currentTab && surfaceReady()
                    ? surfaceBridge.text() : document.content;
            if (localContent === document.savedContent) {
                filesModel.setProperty(i, "content", content);
                filesModel.setProperty(i, "savedContent", content);
                filesModel.setProperty(i, "modified", false);
                clearAt(i);
                if (i === currentTab && surfaceReady()) {
                    replaceCurrentText(content);
                    documentChanged();
                }
                return;
            }
            filesModel.setProperty(i, "externalContent", content);
            filesModel.setProperty(i, "externalConflict", true);
            filesModel.setProperty(i, "externalDeleted", false);
            filesModel.setProperty(i, "externalMessage",
                    failureMessage !== "" ? failureMessage
                    : qsTr("O arquivo mudou no disco enquanto havia alterações locais."));
            if (i === currentTab) {
                syncCurrent();
            }
            return;
        }
    }

    // Troca o texto da superficie preservando a posicao do cursor (clampada
    // ao novo tamanho). Recarregar um arquivo grande sem isso joga o autor
    // para o inicio do documento.
    function replaceCurrentText(content) {
        const cursor = Math.min(surfaceBridge.editorSurface.cursorPosition,
                                content.length);
        surfaceBridge.setText(content);
        surfaceBridge.editorSurface.cursorPosition = cursor;
    }

    // O autor escolheu FICAR COM O DISCO. Devolve o path recarregado (ou ""),
    // que o dono usa para reavisar quem acompanha o arquivo.
    function reloadCurrent() {
        if (currentTab < 0 || currentDeleted) {
            return "";
        }
        const document = filesModel.get(currentTab);
        const content = document.externalContent;
        const path = document.path;
        filesModel.setProperty(currentTab, "content", content);
        filesModel.setProperty(currentTab, "savedContent", content);
        filesModel.setProperty(currentTab, "modified", false);
        clearAt(currentTab);
        if (surfaceReady()) {
            replaceCurrentText(content);
        }
        documentChanged();
        return path;
    }

    // O autor escolheu FICAR COM O LOCAL. O conteudo do disco vira o novo
    // "salvo" (a aba segue modificada em relacao a ele), exceto quando o
    // arquivo foi APAGADO — ai nao ha disco com que comparar.
    function keepLocal() {
        if (currentTab < 0) {
            return;
        }
        const document = filesModel.get(currentTab);
        if (!currentDeleted) {
            filesModel.setProperty(currentTab, "savedContent",
                                   document.externalContent);
        }
        const localContent = surfaceReady()
                ? surfaceBridge.text() : document.content;
        filesModel.setProperty(currentTab, "content", localContent);
        filesModel.setProperty(currentTab, "modified", true);
        clearAt(currentTab);
    }
}
