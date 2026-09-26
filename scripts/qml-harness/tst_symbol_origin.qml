import QtQuick
// Pelo MODULO: o controller usa o `PathRules`, que mora em `editor/`, e um
// import de diretorio nao alcanca as duas pastas.
import KineinVectis

// A FONTE DO SIMBOLO DEIXA DE SER ANONIMA (L1, 2026-09-26).
//
// `@nome` pede simbolos do DOCUMENTO; `#nome` pede do WORKSPACE. Ate' aqui as
// duas respostas chegavam pelo MESMO sinal, sem nada que dissesse qual pedido
// tinha voltado — e o caminho de falha, no mesmo fluxo, sempre preservou o
// metodo. Era o sucesso que perdia a informacao que o fracasso mantinha.
//
// Desde o protocolo 0.135.0 a resposta carrega a identidade do pedido: `path`
// para o documento, `query` para o workspace. Este harness trava o que o
// defeito permitia.
Item {
    id: root

    property int failures: 0

    SearchEverywhereController {
        id: caixa

        workspaceRoot: "/p"
        hasActiveEditorFile: true
    }

    function check(ok, bit, message) {
        if (!ok) {
            root.failures += bit;
            console.error(message);
        }
    }

    function simbolo(nome) {
        return [{ "name": nome, "kind": "function", "path": "/p/src/a.rs",
                  "line": 1, "column": 1 }];
    }

    Component.onCompleted: {
        // Pedido de DOCUMENTO: `@nome`.
        caixa.everywhereVisible = true;
        caixa.runSymbolSearch("@um");
        // O carimbo vem de quem ENVIA — no produto, o roteador; aqui, o teste.
        caixa.noteDocumentAnchor("/p/src/a.rs");

        // A resposta do arquivo CERTO e' aceita.
        check(caixa.handleDocumentSymbols("/p/src/a.rs", simbolo("um")) === true, 1,
              "a resposta do arquivo ancorado e' aceita");

        // A resposta de OUTRO arquivo NAO. Este e' o caso que o sinal unico
        // permitia: trocar de aba e receber a resposta do arquivo anterior.
        caixa.runSymbolSearch("@dois");
        caixa.noteDocumentAnchor("/p/src/a.rs");
        check(caixa.handleDocumentSymbols("/p/src/OUTRO.rs", simbolo("dois")) === false, 2,
              "a resposta de outro arquivo e' descartada");

        // Uma resposta de WORKSPACE nao pode cair num pedido de DOCUMENTO.
        check(caixa.handleWorkspaceSymbols("dois", simbolo("dois")) === false, 4,
              "workspace nao responde a um pedido de documento");

        // Pedido de WORKSPACE: `#nome`.
        caixa.runSymbolSearch("#tres");
        check(caixa.handleWorkspaceSymbols("tres", simbolo("tres")) === true, 8,
              "a resposta da query certa e' aceita");

        // E a resposta de uma query ANTIGA nao. Digitar rapido produz varias
        // perguntas; so' a ultima vale.
        caixa.runSymbolSearch("#quatro");
        check(caixa.handleWorkspaceSymbols("tres", simbolo("tres")) === false, 16,
              "a query antiga e' descartada");

        // E o documento nao responde a um pedido de workspace.
        check(caixa.handleDocumentSymbols("/p/src/a.rs", simbolo("quatro")) === false, 32,
              "documento nao responde a um pedido de workspace");

        // SEM CARIMBO nao ha' o que comparar, e uma resposta solta nao entra:
        // e' o caso de uma resposta que sobreviveu a um pedido cancelado.
        caixa.runSymbolSearch("@cinco");
        check(caixa.handleDocumentSymbols("/p/src/a.rs", simbolo("cinco")) === false, 64,
              "sem carimbo, nada e' aceito");

        Qt.exit(root.failures === 0 ? 0 : 1);
    }
}
