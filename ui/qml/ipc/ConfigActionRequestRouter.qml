import QtQuick

// Espelho do ConfigActionEventRouter: leva ao core o que o controller pede.
//
// As tres requisicoes do ciclo (list -> preview -> apply) passam por aqui e
// so por aqui; o controller nao conhece o CoreClient.
Item {
    id: root

    property var coreClient: null
    property var configActionController: null

    visible: false

    Connections {
        target: root.configActionController

        function onListRequested(includeHiddenByScope) {
            root.coreClient.configActionList(includeHiddenByScope);
        }

        function onPreviewRequested(id, params) {
            root.coreClient.configActionPreview(id, params);
        }

        function onApplyRequested(id, params, expected) {
            root.coreClient.configActionApply(id, params, expected);
        }
    }
}
