import QtQuick

// OS FATOS que tornam uma area contextual do trilho relevante (0.3.7 F1,
// roadmap 53 §4.1), lidos de quem ja' sabe — nunca recalculados aqui.
// `undefined` = ainda nao chegou. O estado de fato vem do core e nunca e'
// persistido pela UI (53 §4.2).
QtObject {
    id: root

    property var embeddedController: null
    property var dataSourceController: null
    property var remoteController: null
    property var grafanaController: null
    // tools.detect: o fato dos containers e' a MAQUINA ter podman ou docker,
    // porque o status do motor so' chega quando o painel abre.
    property var toolsList: []
    // O dono de "esta ferramenta foi detectada" (hasAnyTool): aqui so' se
    // pergunta, sem repetir a leitura dos status.
    property var projectHealthController: null
    property bool workspaceOpen: false

    // Lista ausente: o fato ainda nao chegou (-1). A lista que vem do C++ e'
    // QVariantList; quem sabe le-la e' o `listOf` do codec, dono unico disso.
    readonly property ShellLayoutCodec lists: ShellLayoutCodec {}

    function listLength(list) {
        return list === undefined || list === null ? -1 : lists.listOf(list).length;
    }

    function engineDetected(list) {
        if (listLength(list) <= 0 || !root.projectHealthController) {
            return undefined;
        }
        return root.projectHealthController.hasAnyTool(["podman", "docker"]);
    }

    readonly property var facts: ({
        "project.embedded": root.embeddedController ? root.embeddedController.projectEmbedded === true
                                                    : undefined,
        "datasource.any": root.dataSourceController && root.workspaceOpen
                          && root.listLength(root.dataSourceController.profiles) >= 0
                          ? root.dataSourceController.profiles.length > 0 : undefined,
        "remote.any": root.remoteController && root.workspaceOpen
                      && root.listLength(root.remoteController.targets) >= 0
                      ? root.remoteController.targets.length > 0 : undefined,
        "grafana.instance": root.grafanaController && root.workspaceOpen
                            ? root.grafanaController.hasInstance === true : undefined,
        "container.engine": root.engineDetected(root.toolsList)
    })

    // Fato ainda desconhecido mantem o ULTIMO conhecido (53 §5.4): uma area
    // nao some e volta enquanto o core carrega.
    property var knownFacts: ({})

    function absorb() {
        const merged = Object.assign({}, knownFacts);
        for (const key in facts) {
            if (facts[key] !== undefined) {
                merged[key] = facts[key];
            }
        }
        knownFacts = merged;
    }

    onFactsChanged: absorb()
    Component.onCompleted: absorb()
}
