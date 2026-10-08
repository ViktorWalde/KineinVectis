import QtQuick

// A ORDEM da grade comum (passo 14 da 0.3.9, 59 §5.4.1): qual coluna, em que
// sentido, e a traducao entre a linha como ela APARECE e a linha em `rows`.
// Sem tela, para o harness medir; a comparacao e' do GridRules.
QtObject {
    id: root

    property bool enabled: false
    property var rows: []
    property var columns: []
    // Coluna de numeros por indice (a da grade): compara como numero.
    property var numeric: []
    property GridRules gridRules: null
    property int column: -1
    property string direction: ""

    // Indices de `rows` na ordem da tela; `null` e' a ordem original.
    readonly property var order: root.enabled && root.gridRules !== null
                                 && root.column >= 0 && root.column < root.columns.length
                                 ? root.gridRules.sortedOrder(root.rows, root.columns[root.column], root.column,
                                                          root.direction, root.numeric[root.column] === true)
                                 : null
    readonly property var shownRows: root.order === null ? root.rows : root.order.map(index => root.rows[index])

    function toggle(index) {
        const next = root.gridRules.nextSort(root.column, root.direction, index);
        root.column = next.column;
        root.direction = next.direction;
    }

    function reset() {
        root.column = -1;
        root.direction = "";
    }

    function sourceIndex(shown) {
        return root.order === null || shown < 0 ? shown : root.order[shown];
    }

    function displayIndex(source) {
        return root.order === null || source < 0 ? source : root.order.indexOf(source);
    }
}
