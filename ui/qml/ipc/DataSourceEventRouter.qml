import QtQuick

// Roteia as respostas de `datasource.*` do CoreClient para o controller.
//
// Inclui o `requestFailed` porque neste dominio a recusa e' informacao de
// produto: "informe o host" e "nao ha perfil chamado X" sao coisas que o autor
// precisa ler, nao erros internos.
//
// O veredito do teste chega por EVENTO, nao por resposta: conectar espera a
// rede, entao o pedido devolve um job e o resultado vem depois
// (`arquitetura/04` §5).
Item {
    id: root

    property var coreClient: null
    property var dataSourceController: null

    visible: false

    Connections {
        target: root.coreClient

        function onDataSourceListResolved(profiles) {
            root.dataSourceController.handleList(profiles);
        }

        function onDataSourceTested(name, ok, serverVersion, message, secretRequired) {
            root.dataSourceController.handleTested(name, ok, serverVersion, message,
                                                   secretRequired);
        }

        function onRequestFailed(method, message) {
            root.dataSourceController.handleFailed(method, message);
        }
    }
}
