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

    property var asked: []
    property var ran: []

    DataSourceImpactController {
        id: impact

        onImpactRequested: (name, sql) => root.asked.push(name + "|" + sql)
        onRunConfirmed: (name, sql) => root.ran.push(name + "|" + sql)
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
        impact.begin("loja", "DELETE FROM clientes");
        failures += check(impact.open && impact.measuring && root.asked[0] === "loja|DELETE FROM clientes", "abrir mede");
        failures += check(!impact.canRun, "medindo: Executar desligado");
        impact.handleMeasured({ name: "loja", sql: "outra coisa", severity: "write", statements: [] });
        failures += check(impact.measuring, "evento de outra pergunta nao vale");

        impact.handleMeasured({ name: "loja", sql: "DELETE FROM clientes", severity: "destructive",
                                statements: [{ kind: "delete", targets: ["\"public\".\"clientes\""], severity: "destructive", rows: 3 }] });
        failures += check(impact.destructive && impact.confirmName === "clientes", "nome a digitar: " + impact.confirmName);
        failures += check(!impact.canRun, "destrutiva sem o nome: desligado");
        impact.typed = "client";
        failures += check(!impact.canRun, "nome errado: desligado");
        impact.typed = "clientes";
        failures += check(impact.canRun, "nome certo: ligado");
        failures += check(impact.describe(impact.statements[0]).indexOf("TODAS") >= 0
                          && impact.describe(impact.statements[0]).indexOf("3 linhas") >= 0,
                          "frase: " + impact.describe(impact.statements[0]));
        impact.confirm();
        failures += check(!impact.open && root.ran[0] === "loja|DELETE FROM clientes", "confirmar roda o texto exato");

        // Escrita comum: um clique, depois da medida.
        impact.begin("loja", "DELETE FROM clientes WHERE id = 2");
        impact.handleMeasured({ name: "loja", sql: "DELETE FROM clientes WHERE id = 2", severity: "write",
                                statements: [{ kind: "delete", targets: ["clientes"], filter: "id = 2", severity: "write",
                                               rows: 1, totalRows: 3 }] });
        failures += check(!impact.destructive && impact.canRun, "escrita comum libera");
        failures += check(impact.describe(impact.statements[0]) === "Apaga 1 linha de clientes (a tabela tem 3)",
                          "frase filtrada: " + impact.describe(impact.statements[0]));
        impact.cancel();
        failures += check(!impact.open && root.ran.length === 1, "cancelar nao roda");

        // Sem medida: destrutiva, com o nome da conexao.
        impact.begin("loja", "DROP TABLE x");
        impact.handleFailed("senha necessaria");
        failures += check(impact.destructive && impact.confirmName === "loja" && !impact.canRun, "falha vira destrutiva");
        impact.typed = "loja";
        failures += check(impact.canRun, "com o nome da conexao, libera");

        // As frases dos outros tipos.
        failures += check(impact.describe({ kind: "dropTable", targets: ["pedidos"], rows: 1 }) === "Remove a tabela pedidos e 1 linha dela",
                          impact.describe({ kind: "dropTable", targets: ["pedidos"], rows: 1 }));
        failures += check(impact.describe({ kind: "dropColumn", targets: ["t"], column: "email", rows: 2 }).indexOf("2 valores") >= 0, "coluna");
        failures += check(impact.describe({ kind: "dropDatabase", targets: ["loja"] }).indexOf("INTEIRO") >= 0, "banco");
        failures += check(impact.describe({ kind: "delete", targets: ["sumiu"], note: "no such table" }).indexOf("não deu para contar") >= 0, "sem contagem");
        failures += check(impact.describe({ kind: "update", targets: ["t"], filter: "1=1", rows: 3, totalRows: 3 }).indexOf("tabela inteira") >= 0,
                          "WHERE que pega tudo");

        failures += check(impact.describe({ kind: "mongoDelete", targets: ["s"], filter: "",
                                            rows: 1, totalRows: 3 }).indexOf("TODOS") < 0,
                          "deleteOne sem filtro nao apaga a colecao inteira");
        // Colecao com ponto exige o nome inteiro; SQL continua sem esquema.
        for (const command of [["dropCollection", "drop()"], ["mongoDelete", "deleteMany({})"],
                               ["mongoUpdate", 'updateMany({}, {"$set":{"v":2}})']]) {
            const kind = command[0];
            const sql = "telemetria.sensores." + command[1];
            impact.begin("mongo", sql);
            impact.handleMeasured({ name: "mongo", sql: sql, severity: "destructive",
                                    statements: [{ kind: kind, targets: ["telemetria.sensores"], severity: "destructive" }] });
            impact.typed = "sensores";
            failures += check(!impact.canRun && impact.confirmName === "telemetria.sensores", "nome Mongo completo");
            impact.typed = "telemetria.sensores";
            failures += check(impact.canRun, "nome Mongo completo libera");
        }
        if (failures !== 0) console.error("FALHAS " + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
