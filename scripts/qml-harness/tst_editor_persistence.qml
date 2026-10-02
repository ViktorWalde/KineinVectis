// O que o editor LEMBRA entre sessões, depois do corte de 2026-09-02
// (roadmap 30, etapa 6).
//
// Por que existe: sessão e rascunho pareciam a mesma coisa e não são. Ao sair
// do `EditorController` para um dono próprio, as duas invariantes que as
// separam ficaram sem teste:
//
//   1. a ORDEM do restore — as abas são pedidas na ordem salva (desde 0.3.9,
//      quando a aba passou a se arrastar) e, como cada resposta seleciona a
//      própria aba, a sessão SELECIONA o ativo quando a última chega. Errar
//      isso reabre o projeto na aba errada, e NADA reclama: o build passa, o
//      qmllint passa, e o usuário só nota que "a IDE nunca lembra onde eu
//      estava".
//
//   2. o rascunho só existe para buffer SUJO. Persistir um buffer limpo faria a
//      próxima abertura "recuperar" um texto idêntico ao disco e marcar a aba
//      como modificada sem motivo — foi exatamente esse o defeito que a etapa 1
//      de 2026-08-30 encontrou na primeira `sonda_drafts.py`.
import QtQuick
import "../../ui/qml/editor"

Item {
    id: root

    property var lidos: []
    property var rascunhos: []
    property var sessoes: []
    property bool sujo: false
    property string caminhoAtual: "/tmp/projeto/a.cpp"

    QtObject {
        id: pontefalsa

        // O `Connections` do foco mira `editorSurface`; sem ele no falso, o
        // alvo vira `undefined` e o Qt avisa "Unable to assign".
        readonly property QtObject editorSurface: QtObject {
            property bool editorActiveFocus: true
        }

        function ready() { return true; }
        function text() { return "conteudo do buffer"; }
    }

    ListModel {
        id: openTabs
    }

    property int selectedDocId: 0

    // As abas de um restore ja' carregadas, num modelo a parte: mexer no
    // `abas` agendaria sessao e estragaria a prova de tempo mais abaixo.
    ListModel {
        id: restoredTabs

        ListElement { path: "/tmp/projeto/a.cpp"; docId: 1 }
        ListElement { path: "/tmp/projeto/b.cpp"; docId: 2 }
        ListElement { path: "/tmp/projeto/c.cpp"; docId: 3 }
    }

    QtObject {
        id: documentosFalsos

        function currentFilePath() { return root.caminhoAtual; }
        function currentIsModified() { return root.sujo; }
        function selectDocument(docId) { root.selectedDocId = docId; }
    }

    EditorPersistenceController {
        id: persistence

        workspaceRoot: "/tmp/projeto"
        surfaceBridge: pontefalsa
        documentController: documentosFalsos
        filesModel: openTabs
        onReadFileRequested: function (path) { root.lidos.push(path); }
        onDraftSaveRequested: function (path, content) {
            root.rascunhos.push({ path: path, content: content });
        }
        onSaveSessionRequested: function (files, activeFile) {
            root.sessoes.push({ files: files, activeFile: activeFile });
        }
    }

    // Os debounces são 1500 ms (rascunho) e 1200 ms (sessão). Os prazos abaixo
    // ficam DEPOIS do maior deles, e essa folga é o ponto: a primeira versão
    // deste harness checava aos 700 ms — antes de o rascunho poder disparar —,
    // e por isso NÃO reprovava a mutação "persiste buffer limpo". Checagem que
    // acontece cedo demais é checagem que não pode falhar.
    Timer {
        id: veredito

        interval: 3800
        repeat: false
        onTriggered: root.concluir()
    }

    property int falhasAcumuladas: 0

    function concluir() {
        let failures = root.falhasAcumuladas;

        // O rascunho do buffer SUJO foi persistido...
        if (root.rascunhos.length !== 1) failures += 4096;
        else if (root.rascunhos[0].content !== "conteudo do buffer") failures += 8192;

        // ...e a sessão gravou as abas com a ativa nomeada.
        if (root.sessoes.length === 0) {
            failures += 16384;
        } else {
            const ultima = root.sessoes[root.sessoes.length - 1];
            if (ultima.files.length !== 2) failures += 32768;
            if (ultima.activeFile !== "/tmp/projeto/a.cpp") failures += 65536;
        }

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }

    Component.onCompleted: {
        let failures = 0;

        // ---- A ordem do restore: a das abas, e o ativo no fim -------------
        persistence.restoreSession(
            ["/tmp/projeto/a.cpp", "/tmp/projeto/b.cpp", "/tmp/projeto/c.cpp"],
            "/tmp/projeto/b.cpp");
        if (root.lidos.join() !== "/tmp/projeto/a.cpp,/tmp/projeto/b.cpp,/tmp/projeto/c.cpp") {
            failures += 1;
        }
        // O ativo e' lembrado ate' a ultima aba chegar, e entao selecionado.
        if (persistence.restoringActive !== "/tmp/projeto/b.cpp" || persistence.restoringExpected !== 3) {
            failures += 2;
        }
        persistence.filesModel = restoredTabs;
        persistence.finishRestore();
        persistence.filesModel = openTabs;
        if (root.selectedDocId !== 2) failures += 4;
        if (persistence.restoringActive !== "") failures += 8;

        // Restore sem ativo conhecido não deixa ninguém de fora.
        root.lidos = [];
        persistence.restoreSession(["/tmp/projeto/x.cpp"], "");
        if (root.lidos.length !== 1) failures += 16;

        // ---- Rascunho: buffer LIMPO não vira rascunho ----------------------
        root.sujo = false;
        persistence.scheduleDraftSave();

        // ---- Sessão sem workspace não agenda nada -------------------------
        const sessoesAntes = root.sessoes.length;
        persistence.workspaceRoot = "";
        persistence.scheduleSessionSave();
        persistence.workspaceRoot = "/tmp/projeto";

        root.falhasAcumuladas = failures;

        // Agora sim: buffer sujo e abas abertas. O veredito confere depois do
        // debounce mais longo (1500 ms) mais folga.
        limparEDisparar.start();
        veredito.start();
    }

    Timer {
        id: limparEDisparar

        interval: 1900
        repeat: false
        onTriggered: {
            // 1900 ms > 1500 ms do debounce: se o buffer LIMPO tivesse gerado
            // rascunho, ele ja teria disparado. Este e o instante em que a
            // ausencia vira prova.
            if (root.rascunhos.length !== 0) {
                root.falhasAcumuladas += 1024;
            }
            if (root.sessoes.length !== 0) {
                root.falhasAcumuladas += 2048;
            }
            root.sujo = true;
            openTabs.append({ path: "/tmp/projeto/a.cpp" });
            openTabs.append({ path: "/tmp/projeto/b.cpp" });
            persistence.scheduleDraftSave();
            persistence.scheduleSessionSave();
        }
    }
}
