// O que o editor LEMBRA entre sessões, depois do corte de 2026-09-02
// (roadmap 30, etapa 6).
//
// Por que existe: sessão e rascunho pareciam a mesma coisa e não são. Ao sair
// do `EditorController` para um dono próprio, as duas invariantes que as
// separam ficaram sem teste:
//
//   1. a ORDEM do restore — o arquivo ATIVO é pedido por ÚLTIMO, porque o core
//      responde na ordem em que recebe (`arquitetura/04` §6) e cada resposta
//      seleciona a própria aba. Inverter isso reabre o projeto na aba errada, e
//      NADA reclama: o build passa, o qmllint passa, e o usuário só nota que
//      "a IDE nunca lembra onde eu estava".
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

        function ready() { return true; }
        function text() { return "conteudo do buffer"; }
    }

    ListModel {
        id: abas
    }

    QtObject {
        id: documentosFalsos

        function currentFilePath() { return root.caminhoAtual; }
        function currentIsModified() { return root.sujo; }
    }

    EditorPersistenceController {
        id: persistence

        workspaceRoot: "/tmp/projeto"
        surfaceBridge: pontefalsa
        documentController: documentosFalsos
        filesModel: abas
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

        // ---- A ordem do restore: o ATIVO por ÚLTIMO ------------------------
        persistence.restoreSession(
            ["/tmp/projeto/a.cpp", "/tmp/projeto/b.cpp", "/tmp/projeto/c.cpp"],
            "/tmp/projeto/b.cpp");
        if (root.lidos.length !== 3) {
            failures += 1;
        } else {
            if (root.lidos[2] !== "/tmp/projeto/b.cpp") failures += 2;
            // Os outros dois vão antes, em ordem, e o ativo não é repetido.
            if (root.lidos[0] !== "/tmp/projeto/a.cpp") failures += 4;
            if (root.lidos[1] !== "/tmp/projeto/c.cpp") failures += 8;
        }

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
            abas.append({ path: "/tmp/projeto/a.cpp" });
            abas.append({ path: "/tmp/projeto/b.cpp" });
            persistence.scheduleDraftSave();
            persistence.scheduleSessionSave();
        }
    }
}
