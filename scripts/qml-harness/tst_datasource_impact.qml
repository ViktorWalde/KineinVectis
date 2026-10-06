import QtQuick
import KineinVectis

// A confirmacao de escrita com o impacto medido (0.150.0). O que se prova:
// abrir pede a medida; evento de outra pergunta nao vale; a escrita comum
// libera o Executar com a medida; a DESTRUTIVA so' com o nome digitado (sem
// aspas nem esquema); sem medida (falha) vira destrutiva com o nome da
// conexao; confirmar emite o texto exato; cancelar nao emite nada; e as
// frases dizem o numero.
Item {
    id: root
    width: 800
    height: 800

    property var asked: []
    property var ran: []

    DataSourceImpactController {
        id: impactController

        onImpactRequested: (name, sql) => root.asked.push(name + "|" + sql)
        onRunConfirmed: (name, sql) => root.ran.push(name + "|" + sql)
    }

    // O diálogo real também avalia as ligações dos delegados do impacto.
    SqlImpactDialog {
        anchors.fill: parent
        impact: impactController
        visible: impactController.open
    }

    function check(condition, message) {
        if (!condition) {
            console.error("FALHOU: " + message);
            return 1;
        }
        return 0;
    }

    Component.onCompleted: {
        let failures = 0;
        impactController.begin("loja", "DELETE FROM clientes");
        failures += check(impactController.open && impactController.measuring && root.asked[0] === "loja|DELETE FROM clientes", "abrir mede");
        failures += check(!impactController.canRun, "medindo: Executar desligado");
        impactController.handleMeasured({ clientContext: impactController.clientContext, name: "loja", sql: "outra coisa", severity: "write", statements: [] });
        failures += check(impactController.measuring, "evento de outra pergunta nao vale");

        impactController.handleMeasured({ clientContext: impactController.clientContext, name: "loja", sql: "DELETE FROM clientes", severity: "destructive", confirmationTarget: "clientes",
                                statements: [{ kind: "delete", targets: ["\"public\".\"clientes\""], severity: "destructive", rows: 3 }] });
        failures += check(impactController.destructive && impactController.confirmName === "clientes", "nome a digitar: " + impactController.confirmName);
        failures += check(!impactController.canRun, "destrutiva sem o nome: desligado");
        impactController.typed = "client";
        failures += check(!impactController.canRun, "nome errado: desligado");
        impactController.typed = "clientes";
        failures += check(impactController.canRun, "nome certo: ligado");
        failures += check(impactController.describe(impactController.statements[0]).indexOf("TODAS") >= 0
                          && impactController.describe(impactController.statements[0]).indexOf("3 linhas") >= 0,
                          "frase: " + impactController.describe(impactController.statements[0]));
        impactController.confirm();
        failures += check(!impactController.open && root.ran[0] === "loja|DELETE FROM clientes", "confirmar roda o texto exato");

        // Escrita comum: um clique, depois da medida.
        impactController.begin("loja", "DELETE FROM clientes WHERE id = 2");
        impactController.handleMeasured({ clientContext: impactController.clientContext, name: "loja", sql: "DELETE FROM clientes WHERE id = 2", severity: "write",
                                statements: [{ kind: "delete", targets: ["clientes"], filter: "id = 2", severity: "write",
                                               rows: 1, totalRows: 3 }] });
        failures += check(!impactController.destructive && impactController.canRun, "escrita comum libera");
        failures += check(impactController.describe(impactController.statements[0]) === "Apaga 1 linha de clientes (a tabela tem 3)",
                          "frase filtrada: " + impactController.describe(impactController.statements[0]));
        impactController.cancel();
        failures += check(!impactController.open && root.ran.length === 1, "cancelar nao roda");

        // Sem medida: destrutiva, com o nome da conexao.
        impactController.begin("loja", "DROP TABLE x");
        impactController.handleFailed("senha necessaria", { name: impactController.name, clientContext: impactController.clientContext });
        failures += check(impactController.destructive && impactController.confirmName === "loja" && !impactController.canRun, "falha vira destrutiva");
        impactController.typed = "loja";
        failures += check(impactController.canRun, "com o nome da conexao, libera");

        // As frases dos outros tipos.
        failures += check(impactController.describe({ kind: "dropTable", targets: ["pedidos"], rows: 1 }) === "Remove a tabela pedidos e 1 linha dela",
                          impactController.describe({ kind: "dropTable", targets: ["pedidos"], rows: 1 }));
        failures += check(impactController.describe({ kind: "dropColumn", targets: ["t"], column: "email", rows: 2 }).indexOf("2 valores") >= 0, "coluna");
        failures += check(impactController.describe({ kind: "dropDatabase", targets: ["loja"] }).indexOf("INTEIRO") >= 0, "banco");
        failures += check(impactController.describe({ kind: "delete", targets: ["sumiu"], note: "no such table" }).indexOf("não deu para contar") >= 0, "sem contagem");
        failures += check(impactController.describe({ kind: "update", targets: ["t"], filter: "1=1", rows: 3, totalRows: 3 }).indexOf("tabela inteira") >= 0,
                          "WHERE que pega tudo");

        failures += check(impactController.describe({ kind: "mongoDelete", targets: ["s"], filter: "",
                                            rows: 1, totalRows: 3 }).indexOf("TODOS") < 0,
                          "deleteOne sem filtro nao apaga a colecao inteira");
        // Colecao com ponto exige o nome inteiro; SQL continua sem esquema.
        for (const command of [["dropCollection", "drop()"], ["mongoDelete", "deleteMany({})"],
                               ["mongoUpdate", 'updateMany({}, {"$set":{"v":2}})']]) {
            const kind = command[0];
            const sql = "telemetria.sensores." + command[1];
            impactController.begin("mongo", sql);
            impactController.handleMeasured({ clientContext: impactController.clientContext, name: "mongo", sql: sql, severity: "destructive", confirmationTarget: "telemetria.sensores",
                                    statements: [{ kind: kind, targets: ["telemetria.sensores"], severity: "destructive" }] });
            impactController.typed = "sensores";
            failures += check(!impactController.canRun && impactController.confirmName === "telemetria.sensores", "nome Mongo completo");
            impactController.typed = "telemetria.sensores";
            failures += check(impactController.canRun, "nome Mongo completo libera");
        }
        if (failures !== 0) console.error("FALHAS " + failures);
        Qt.callLater(() => Qt.exit(failures === 0 ? 0 : 1));
    }
}
