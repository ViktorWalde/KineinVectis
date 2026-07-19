import QtQuick

// Modo de visualizacao dos .md ("imagem" = markdown renderizado, como o
// Manual da IDE): guarda POR ARQUIVO se a aba esta em preview. Pedido do
// autor em 2026-07-18 para melhorar a leitura da documentacao do projeto.
//
// E' estado de APRESENTACAO (que vista mostrar), nao regra de negocio — o
// core nunca fica sabendo; salvar, buffers e conflitos continuam nos donos.
// Vive num controller separado (e nao no EditorPane) para ser testavel no
// harness headless, que nao carrega componente visual.
Item {
    id: root

    // path -> true. Mapa mutado in-place: `revision` forca o rebind, o mesmo
    // idioma de gitKinds/diffLineKinds no resto do repositorio.
    property var previewPaths: ({})
    property int revision: 0

    visible: false

    function isMarkdownPath(path) {
        return path.toLowerCase().endsWith(".md");
    }

    // `rev` existe so para o binding reavaliar quando o mapa muda.
    function isPreview(path, rev) {
        return isMarkdownPath(path) && previewPaths[path] === true;
    }

    function toggle(path) {
        if (!isMarkdownPath(path)) {
            return;
        }
        previewPaths[path] = previewPaths[path] !== true;
        revision += 1;
    }

    function clear() {
        previewPaths = {};
        revision += 1;
    }
}
