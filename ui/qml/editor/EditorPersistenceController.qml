pragma ComponentBehavior: Bound
import QtQuick

// O QUE O EDITOR LEMBRA quando a IDE fecha: as abas abertas (sessão) e o
// buffer não salvo (rascunho).
//
// # Duas redes com propósitos opostos, e é por isso que moram juntas
//
// Elas parecem a mesma coisa e não são — e confundi-las é o que produz o bug
// clássico de "a IDE reabriu meu arquivo modificado sem eu ter salvado, ou
// pior, sem a modificação".
//
// ```text
// SESSÃO     quais arquivos estavam abertos, e qual estava ativo.
//            Sobrevive a um fechamento NORMAL. É conveniência.
//            → .kinein/session.json, via `workspace.saveSession`
//
// RASCUNHO   o TEXTO não salvo de um buffer sujo.
//            Sobrevive a um CRASH — e SÓ a um crash: salvar limpa o rascunho,
//            porque o disco passou a ser a verdade. É rede de segurança de
//            DADO (`DocsPublic/seguranca/23`, pilar 2).
//            → .kinein/kinein.db (SQLite/WAL), via `draft.save`
// ```
//
// O que as une é o gesto: as duas gravam **depois que o usuário para**, por
// debounce, e as duas são relidas na abertura do workspace. Por isso um dono
// só — e por isso este arquivo diz, logo no topo, qual é qual.
//
// # Por que os dois debounces têm tempos diferentes
//
// `1500 ms` para o rascunho e `1200 ms` para a sessão não são números
// arbitrários herdados: o rascunho grava o TEXTO INTEIRO num banco a cada
// disparo, e a sessão grava uma lista de caminhos. O mais caro espera mais.
//
// # O rascunho tem prazo máximo (INF-26, corrigido em 2026-11-22)
//
// Só o debounce não basta: cada tecla reinicia o temporizador, e quem digita
// sem uma pausa de 1,5 s não tinha NADA gravado. Medido em 2026-11-15 (62
// §3.4): 100 s de digitação e `kill -9` logo depois, 0 teclas recuperadas em
// 3 de 3. Por isso há um segundo temporizador, `draftMaxWait`, que a primeira
// mudança suja liga e as seguintes NÃO reiniciam. O que disparar primeiro
// grava e desliga os dois. Assim toda mudança sai para o core em até
// `draftMaxWaitMs` (2 s, o RNF-DUR-01 do core novo), digite a pessoa quanto
// digitar. O teste é o `ui/tests/tst_editor_draft_deadline.cpp`.
//
// A gravação automática no DISCO (`diskSaveDebounce`) continua só com o
// debounce, de propósito: gravar o arquivo do projeto no meio da digitação
// faz o editor reler o que gravou (INF-28), e a rede contra queda é o
// rascunho, não o disco.
//
// # A ordem do restore de sessão é carga estrutural
//
// Os arquivos são pedidos NA ORDEM DAS ABAS (desde 0.3.9: a aba se arrasta, e
// a ordem tem de voltar como estava). O core responde na ordem em que recebe
// (`arquitetura/04` §6, a única garantia de ordem que existe) e cada
// `fileLoaded` seleciona a própria aba — por isso, quando a última chega, a
// sessão SELECIONA o ativo de volta. Até 2026-10-02 o ativo era pedido por
// último e ia parar no fim da fila; selecionar antes do fim reabre o projeto
// na aba errada, e nada reclama.
Item {
    id: root

    property string workspaceRoot: ""
    property var surfaceBridge: null
    property var documentController: null
    // Os caminhos das abas abertas, para o snapshot da sessão.
    property var filesModel: null

    // AUTOSAVE (Etapa 2, F3, decisao do autor em 2026-09-18): o buffer sujo vai
    // para o DISCO sozinho — apos 2 s de pausa (aqui), ao trocar de aba e
    // quando o editor perde o foco (o EditorController pede). O rascunho
    // continua: ele grava aos 1,5 s e e' a rede ate' o disco receber; salvar
    // o limpa (a regra de sempre). Desligavel em Configuracoes.
    property bool autoSaveEnabled: true

    // A pausa que grava o rascunho e o prazo máximo que nenhuma digitação
    // adia (INF-26, ver o topo). Propriedades para o teste poder encurtar.
    property int draftPauseMs: 1500
    property int draftMaxWaitMs: 2000

    signal readFileRequested(string path)
    signal draftSaveRequested(string path, string content)
    signal saveSessionRequested(var files, string activeFile)
    signal autoSaveRequested()

    visible: false

    function ready() {
        return surfaceBridge !== null && surfaceBridge.ready()
                && documentController !== null;
    }

    function currentPath() {
        return documentController === null || documentController.currentReadOnly
               ? "" : documentController.currentFilePath();
    }

    // O ativo da sessão em restauração, e quantas abas ela abre.
    property string restoringActive: ""
    property int restoringExpected: 0

    function restoreSession(files, activeFile) {
        restoringActive = activeFile;
        restoringExpected = files.length;
        for (let index = 0; index < files.length; index++) {
            readFileRequested(files[index]);
        }
        restoreDeadline.restart();
    }

    // A última aba chegou (ou o prazo venceu: um arquivo da sessão pode ter
    // sumido do disco): volta para o ativo — ver a nota da ordem, no topo.
    function finishRestore() {
        if (restoringActive === "" || filesModel === null) return;
        for (let index = 0; index < filesModel.count; index++) {
            if (filesModel.get(index).path === restoringActive) {
                documentController.selectDocument(filesModel.get(index).docId);
                break;
            }
        }
        restoringActive = "";
        restoreDeadline.stop();
    }

    Timer {
        id: restoreDeadline

        interval: 3000
        onTriggered: root.finishRestore()
    }

    // Sem workspace não há sessão a gravar: o snapshot iria para lugar nenhum.
    function scheduleSessionSave() {
        if (workspaceRoot === "") {
            return;
        }
        sessionSaveDebounce.restart();
    }

    function scheduleDraftSave() {
        autosaveDebounce.restart();
        // `start` e não `restart`: o prazo conta da primeira mudança ainda não
        // gravada. Reiniciá-lo aqui devolveria o INF-26.
        if (!draftMaxWait.running) {
            draftMaxWait.start();
        }
        if (autoSaveEnabled) {
            diskSaveDebounce.restart();
        }
    }

    function cancelPendingSaves() {
        autosaveDebounce.stop();
        draftMaxWait.stop();
        diskSaveDebounce.stop();
        sessionSaveDebounce.stop();
    }

    // Quem disparar primeiro, a pausa ou o prazo, grava e desliga os dois.
    function saveDraftNow() {
        autosaveDebounce.stop();
        draftMaxWait.stop();
        // `currentIsModified` é a guarda que impede o rascunho de existir
        // para um buffer limpo — rascunho idêntico ao disco reabriria a
        // aba marcada como modificada sem motivo.
        const path = currentPath();
        if (path !== "" && ready() && documentController.currentIsModified()) {
            draftSaveRequested(path, surfaceBridge.text());
        }
    }

    // O gesto que nao espera a pausa: trocar de aba, perder o foco.
    function flushAutoSave() {
        if (!autoSaveEnabled) {
            return;
        }
        diskSaveDebounce.stop();
        autoSaveRequested();
    }

    Timer {
        id: diskSaveDebounce

        interval: 2000
        repeat: false
        onTriggered: {
            if (root.autoSaveEnabled && root.currentPath() !== "" && root.ready()
                    && root.documentController.currentIsModified()) {
                root.autoSaveRequested();
            }
        }
    }

    Timer {
        id: autosaveDebounce

        interval: root.draftPauseMs
        repeat: false
        onTriggered: root.saveDraftNow()
    }

    Timer {
        id: draftMaxWait

        interval: root.draftMaxWaitMs
        repeat: false
        onTriggered: root.saveDraftNow()
    }

    Timer {
        id: sessionSaveDebounce

        interval: 1200
        repeat: false
        onTriggered: {
            if (root.workspaceRoot === "" || root.filesModel === null) {
                return;
            }
            const files = [];
            for (let index = 0; index < root.filesModel.count; index++) {
                const document = root.filesModel.get(index);
                if (document.readOnly !== true) files.push(document.path);
            }
            root.saveSessionRequested(files, root.currentPath());
        }
    }

    // Sair do editor (clicar no terminal, no explorer, noutra janela) salva.
    Connections {
        target: root.surfaceBridge !== null && root.surfaceBridge.ready()
                ? root.surfaceBridge.editorSurface : null

        function onEditorActiveFocusChanged() {
            if (!root.surfaceBridge.editorSurface.editorActiveFocus && root.ready()
                    && root.documentController.currentIsModified()) {
                root.flushAutoSave();
            }
        }
    }

    // Abrir, fechar ou ARRASTAR aba (0.3.9) muda a sessão tanto quanto
    // trocar de aba: a ordem das abas volta como estava.
    Connections {
        target: root.filesModel

        function onCountChanged() {
            if (root.restoringActive !== "" && root.filesModel.count >= root.restoringExpected) {
                Qt.callLater(root.finishRestore);
            }
            root.scheduleSessionSave();
        }

        function onRowsMoved() {
            root.scheduleSessionSave();
        }
    }
}
