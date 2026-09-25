import QtQuick
import "../../ui/qml/command"

// O CommandDispatcher REAL (fatia V3, 2026-09-24).
//
// O item 1 da V3 e' "dispatcher atual recebe resultado observavel para ID
// desconhecido". Ate' aqui o `execute` terminava a cadeia em silencio: um id
// errado na paleta, no menu ou no `KINEIN_STARTUP_COMMANDS` simplesmente NAO
// ACONTECIA, e nada dizia por que — o pior tipo de falha, a que parece
// funcionamento.
Item {
    id: root

    property int failures: 0
    property var desconhecidos: []
    property var chamadas: []
    property string abaPedida: ""
    property string simbolos: ""
    property int aberturas: 0

    function check(ok, mensagem) {
        if (!ok) {
            failures++;
            console.error(mensagem);
        }
    }

    // Falsos so' do que os comandos exercitados tocam.
    QtObject {
        id: coreFalso
        function closeWorkspace() { root.chamadas.push("closeWorkspace"); }
        function detectTools() { root.chamadas.push("detectTools"); }
        function cargoCheck() { root.chamadas.push("cargoCheck"); }
    }

    QtObject {
        id: editorFalso
        function openFind() { root.chamadas.push("openFind"); }
    }

    CommandDispatcher {
        id: d

        coreClient: coreFalso
        editorController: editorFalso
        onUnknownCommand: function(id) { root.desconhecidos.push(id); }
        onOpenWorkspaceRequested: root.aberturas++
        onShowTabRequested: function(tab) { root.abaPedida = tab; }
        onSymbolsRequested: function(q) { root.simbolos = q; }
    }

    Component.onCompleted: {
        // ID desconhecido: resultado OBSERVAVEL nas duas formas — o retorno e
        // o sinal. Sem uma das duas, quem chama nao tem como saber.
        check(d.execute("nao.existe") === false, "id desconhecido devolve false");
        check(desconhecidos.length === 1 && desconhecidos[0] === "nao.existe",
              "e anuncia QUAL id ninguem tratou");
        check(chamadas.length === 0, "id desconhecido nao pode tocar em nada");

        // Um id conhecido devolve true e faz o que promete.
        check(d.execute("workspace.close") === true, "id conhecido devolve true");
        check(chamadas[0] === "closeWorkspace", "e chamou o dono");
        check(desconhecidos.length === 1, "id conhecido nao vira desconhecido");

        check(d.execute("workspace.open") === true && aberturas === 1, "workspace.open");

        // Dois ids no mesmo braco continuam valendo os dois.
        check(d.execute("tools.detect") === true && abaPedida === "tools", "tools.detect");
        check(d.execute("tools.status") === true, "tools.status compartilha o braco");

        // `id=arg`: so' a medicao headless usa, e a paleta nunca manda `=`.
        check(d.execute("index.symbols=Widget") === true && simbolos === "Widget",
              "o argumento depois do `=` chega");
        check(d.execute("index.symbols") === true && simbolos === "",
              "sem `=`, o argumento e' vazio");

        // Um `=` num id DESCONHECIDO ainda e' desconhecido, e o anuncio traz o
        // id sem o argumento — senao a mensagem viraria lixo.
        check(d.execute("nao.existe=x") === false, "desconhecido com argumento");
        check(desconhecidos[desconhecidos.length - 1] === "nao.existe",
              "anuncia o id, nao o id=arg");

        // Comando que o core nao conhece, mas a UI trata sozinha.
        check(d.execute("editor.find") === true, "editor.find e' da UI");

        Qt.exit(failures === 0 ? 0 : 1);
    }
}
