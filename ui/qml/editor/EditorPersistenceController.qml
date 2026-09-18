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
// # A ordem do restore de sessão é carga estrutural
//
// Os arquivos são pedidos com o ATIVO POR ÚLTIMO. O core responde na ordem em
// que recebe (`arquitetura/04` §6, a única garantia de ordem que existe) e cada
// `fileLoaded` seleciona a própria aba — então quem carrega por último fica
// ativo. Inverter isso reabre o projeto na aba errada, e nada reclama.
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
        return documentController === null ? "" : documentController.currentFilePath();
    }

    function restoreSession(files, activeFile) {
        for (let index = 0; index < files.length; index++) {
            if (files[index] !== activeFile) {
                readFileRequested(files[index]);
            }
        }
        // O ATIVO por último — ver a nota da ordem, no topo do arquivo.
        for (let index = 0; index < files.length; index++) {
            if (files[index] === activeFile) {
                readFileRequested(files[index]);
                break;
            }
        }
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
        if (autoSaveEnabled) {
            diskSaveDebounce.restart();
        }
    }

    function cancelPendingSaves() {
        autosaveDebounce.stop();
        diskSaveDebounce.stop();
        sessionSaveDebounce.stop();
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

        interval: 1500
        repeat: false
        onTriggered: {
            // `currentIsModified` é a guarda que impede o rascunho de existir
            // para um buffer limpo — rascunho idêntico ao disco reabriria a
            // aba marcada como modificada sem motivo.
            const path = root.currentPath();
            if (path !== "" && root.ready() && root.documentController.currentIsModified()) {
                root.draftSaveRequested(path, root.surfaceBridge.text());
            }
        }
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
                files.push(root.filesModel.get(index).path);
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

    // Abrir ou fechar aba muda a sessão tanto quanto trocar de aba.
    Connections {
        target: root.filesModel

        function onCountChanged() {
            root.scheduleSessionSave();
        }
    }
}
