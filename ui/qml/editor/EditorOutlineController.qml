import QtQuick

Item {
    id: root

    property var items: []
    property var collapsed: ({})
    property alias model: flatModel
    readonly property int count: flatModel.count

    visible: false

    function nodeKey(parentKey, node, index) {
        return parentKey + "/" + index + ":" + String(node.name)
                + ":" + String(node.line);
    }

    function appendNodes(nodes, depth, parentKey) {
        if (nodes === undefined || nodes === null) {
            return;
        }
        for (let i = 0; i < nodes.length; i++) {
            const node = nodes[i];
            const children = node.children || [];
            const key = nodeKey(parentKey, node, i);
            const isCollapsed = collapsed[key] === true;
            flatModel.append({
                nodeKey: key,
                displayName: String(node.name || ""),
                symbolKind: String(node.kind || "symbol"),
                targetLine: Number(node.line || 1),
                targetColumn: Number(node.column || 1),
                depth: depth,
                hasChildren: children.length > 0,
                expanded: !isCollapsed
            });
            if (!isCollapsed) {
                appendNodes(children, depth + 1, key);
            }
        }
    }

    function rebuild() {
        flatModel.clear();
        appendNodes(items, 0, "root");
    }

    function toggle(key) {
        const next = collapsed;
        next[key] = next[key] !== true;
        collapsed = next;
        rebuild();
    }

    onItemsChanged: rebuild()

    ListModel {
        id: flatModel
    }
}
