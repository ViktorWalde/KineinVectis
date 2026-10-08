// ORDENAR NA GRADE COMUM (passo 14 da 0.3.9, roadmaps/59 §5.4.1).
//
// O que este teste guarda: a ordem so' existe onde a grade a liga; as linhas
// DESENHADAS seguem a ordem; a selecao continua falando da linha como ela
// aparece e a traducao para `rows` acerta; e a mesma consulta de novo (o
// "Carregar mais") mantem a ordem, enquanto outra consulta a desfaz.
//
// MUTACOES QUE PROVAM O GATE: o `Repeater` da grade voltar a `root.rows`; o
// `onColumnsChanged` resetar sem comparar as chaves; o `enabled` do
// KvDataGridSort ignorado.
import QtQuick
import KineinVectis

Item {
    id: root

    width: 400
    height: 300

    property int failures: 0

    KvDataGrid {
        id: grid

        width: parent.width
        height: 200
        maxHeight: 200
        selectable: true
        columns: [{ key: "id", label: "id" }, { key: "name", label: "name" }]
        rows: [{ id: "10", name: "b" }, { id: "9", name: null }, { id: "2", name: "a" }]
    }

    function check(condition, message) {
        if (!condition) {
            console.error("FALHOU: " + message);
            root.failures += 1;
        }
    }

    // As linhas que a grade DESENHOU, na ordem do Repeater.
    function drawnIds() {
        const found = [];
        const visit = item => {
            for (let i = 0; i < item.children.length; i++) {
                const child = item.children[i];
                if (child.selected !== undefined && child.modelData !== undefined && child.index !== undefined) {
                    found.push({ index: child.index, id: child.modelData.id });
                }
                visit(child);
            }
        };
        visit(grid);
        return found.sort((a, b) => a.index - b.index).map(row => row.id).join(",");
    }

    Component.onCompleted: {
        // DESLIGADA, a grade nao ordena: e' o caso de containers, git e Grafana.
        grid.sorting.toggle(0);
        root.check(grid.sorting.order === null && root.drawnIds() === "10,9,2",
                   "grade sem `sortable` ordenou: " + root.drawnIds());
        grid.sorting.reset();

        grid.sortable = true;
        grid.sorting.toggle(0);
        root.check(root.drawnIds() === "2,9,10", "crescente numerico nao desenhou: " + root.drawnIds());
        // A linha na tela e a linha em `rows`: o "10" esta' em `rows[0]` e
        // aparece por ultimo.
        root.check(grid.sorting.displayIndex(0) === 2 && grid.sorting.sourceIndex(2) === 0
                   && grid.sorting.displayIndex(-1) === -1,
                   "a traducao da selecao errou");

        grid.sorting.toggle(0);
        root.check(root.drawnIds() === "10,9,2", "decrescente nao desenhou: " + root.drawnIds());
        grid.sorting.toggle(0);
        root.check(grid.sorting.order === null && root.drawnIds() === "10,9,2",
                   "o terceiro clique nao voltou a ordem original");

        // A MESMA consulta de novo (colunas iguais, linhas novas): a ordem fica.
        grid.sorting.toggle(0);
        grid.columns = [{ key: "id", label: "id" }, { key: "name", label: "name" }];
        grid.rows = [{ id: "7", name: "c" }, { id: "1", name: "d" }];
        root.check(grid.sorting.column === 0 && root.drawnIds() === "1,7",
                   "carregar mais perdeu a ordem: " + root.drawnIds());

        // OUTRA consulta: a ordem original volta.
        grid.columns = [{ key: "id", label: "id" }];
        root.check(grid.sorting.column === -1 && grid.sorting.order === null,
                   "outra consulta herdou a ordem");

        Qt.exit(root.failures === 0 ? 0 : 1);
    }
}
