import QtQuick

// Resolucao central dos icones de arquivo. A ordem preserva a semantica do
// pacote visual: nomes especiais vencem extensoes e YAML generico nunca vira
// Docker Compose por acidente.
KvIcon {
    id: root

    property string fileName: ""
    property bool directory: false
    property bool expanded: false

    function hasExtension(lowerName, extensions) {
        for (let i = 0; i < extensions.length; ++i) {
            if (lowerName.endsWith(extensions[i])) {
                return true;
            }
        }
        return false;
    }

    function resolvedIconName(fileName, directory, expanded) {
        if (directory) {
            return expanded ? "tree-folder-open" : "tree-folder-closed";
        }

        const lowerName = fileName.toLowerCase();
        if (lowerName === "cmakelists.txt") {
            return "tree-file-cmakelists";
        }
        if (lowerName === "dockerfile" || lowerName.startsWith("dockerfile.")
                || lowerName.endsWith(".dockerfile")
                || lowerName === "compose.yaml" || lowerName === "compose.yml"
                || lowerName === "docker-compose.yaml"
                || lowerName === "docker-compose.yml"
                || (lowerName.startsWith("compose.")
                    && (lowerName.endsWith(".yaml") || lowerName.endsWith(".yml")))
                || (lowerName.startsWith("docker-compose.")
                    && (lowerName.endsWith(".yaml") || lowerName.endsWith(".yml")))) {
            return "tree-file-docker";
        }
        if (lowerName.endsWith(".h")) {
            return "tree-file-h";
        }
        if (root.hasExtension(lowerName, [".hh", ".hpp", ".hxx", ".h++", ".ipp"])) {
            return "tree-file-hpp";
        }
        if (root.hasExtension(lowerName, [".c", ".cc", ".cpp", ".cxx", ".c++"])) {
            return "tree-file-c";
        }
        if (lowerName.endsWith(".rs")) {
            return "tree-file-rust";
        }
        if (root.hasExtension(lowerName, [".py", ".pyi", ".pyw"])) {
            return "tree-file-python";
        }
        if (root.hasExtension(lowerName, [".yaml", ".yml"])) {
            return "tree-file-yaml";
        }
        if (lowerName.endsWith(".sql")) {
            return "tree-file-sql";
        }
        if (root.hasExtension(lowerName, [".md", ".markdown", ".mdown", ".mkdn"])) {
            return "tree-file-markdown";
        }
        return "file";
    }

    name: root.resolvedIconName(root.fileName, root.directory, root.expanded)
}
