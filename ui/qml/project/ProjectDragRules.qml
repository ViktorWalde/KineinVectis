pragma Singleton
import QtQuick

// Um único parser para o MIME interno da árvore, usado por seus destinos.
QtObject {
    readonly property ProjectTreeRules treeRules: ProjectTreeRules {}

    function internalPaths(event) {
        if (event.formats.indexOf("application/x-kinein-project-paths") < 0) return [];
        // Uma aplicação externa pode forjar MIME/keys. Só o Item da árvore
        // nesta instância pode autorizar a semântica de mover.
        if (event.source === null || event.source === undefined
                || !Array.isArray(event.source.dragPaths)) return [];
        let paths;
        try {
            paths = JSON.parse(event.getDataAsString("application/x-kinein-project-paths"));
        } catch (_) {
            return [];
        }
        if (!treeRules.validPaths(paths)) return [];
        if (paths.length !== event.source.dragPaths.length
                || !paths.every((path, index) => path === event.source.dragPaths[index])) return [];
        return paths;
    }
}
