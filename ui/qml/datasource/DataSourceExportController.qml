pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// EXPORTAR o resultado em CSV (passo 14b, roadmaps/59 §5.4.1): a pasta
// `exportacoes/` e o arquivo, pelo canal proprio da ponte (`exportDirectory`/
// `exportFile`), que a arvore do projeto nao ve. A pasta pode ja' existir e o
// core nao distingue esse caso por codigo: a falha dela nao e' relatada, e o
// pedido do arquivo diz a verdade. Um pedido por vez.
QtObject {
    id: root

    property string workspaceRoot: ""
    // O pedido em curso: { path, relative, content, rows }; null = nenhum.
    property var pending: null
    readonly property bool busy: root.pending !== null

    signal directoryRequested(string path)
    signal fileRequested(string path, string content)
    signal finished(string message, bool ok)

    onWorkspaceRootChanged: root.pending = null

    function directoryPath() {
        return root.workspaceRoot + "/" + DataSourceKinds.exportDirectory;
    }

    function begin(connection, content, rows, now) {
        if (root.busy || root.workspaceRoot === "") return;
        const relative = DataSourceKinds.exportPath(connection, now);
        root.pending = { path: root.workspaceRoot + "/" + relative, relative: relative, content: content, rows: rows };
        root.directoryRequested(root.directoryPath());
    }

    // A pasta respondeu, criada agora ou nao: o arquivo segue.
    function directoryDone(path) {
        if (!root.busy || path !== root.directoryPath()) return;
        root.fileRequested(root.pending.path, root.pending.content);
    }

    function fileDone(path) {
        if (!root.busy || path !== root.pending.path) return;
        const done = root.pending;
        root.pending = null;
        root.finished(qsTr("Exportado: %1 (%2 linha(s)).").arg(done.relative).arg(done.rows), true);
    }

    function fileFailed(path, message) {
        if (!root.busy || path !== root.pending.path) return;
        root.pending = null;
        root.finished(qsTr("A exportação falhou: %1").arg(message), false);
    }

    // As respostas da ponte, pelo metodo de cada passo.
    function handleSucceeded(method, path) {
        if (method === "fs.createDirectory") root.directoryDone(path);
        else root.fileDone(path);
    }

    function handleFailed(method, path, message) {
        if (method === "fs.createDirectory") root.directoryDone(path);
        else root.fileFailed(path, message);
    }
}
