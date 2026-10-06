pragma ComponentBehavior: Bound
import QtQuick

// Guarda somente a prévia pública da consulta atual. A transação pertence ao core.
QtObject {
    id: root
    property var dataSourceController: null
    property var active: null
    property bool deciding: false
    property string message: ""
    property double deadline: 0
    property int remaining: 0
    readonly property bool open: root.active !== null
    readonly property bool canCommit: root.open && root.current() && !root.deciding && root.remaining > 0

    signal decisionRequested(var operation)
    signal cancelExecutionRequested(string jobId)

    readonly property Timer countdown: Timer {
        interval: 1000
        repeat: true
        running: root.open && !root.deciding
        onTriggered: root.remaining = Math.max(0, Math.ceil((root.deadline - Date.now()) / 1000))
    }

    function current() {
        return root.active !== null && root.dataSourceController !== null
            && root.dataSourceController.queries.matches(root.active);
    }

    function prepared(event) {
        if (root.dataSourceController === null) return;
        const query = root.dataSourceController.lastQuery;
        if (!root.dataSourceController.queries.matches(event) || query.preview !== true) {
            root.cancelExecutionRequested(event.jobId);
            return;
        }
        root.active = Object.assign({}, event, { expectedContext: query.expectedContext });
        root.deciding = false;
        root.message = "";
        root.remaining = event.expiresInSeconds;
        root.deadline = Date.now() + event.expiresInSeconds * 1000;
    }

    function decide(decision) {
        if (!root.current() || root.deciding || decision === "commit" && !root.canCommit) return;
        root.deciding = true;
        root.message = decision === "commit" ? qsTr("Confirmando no PostgreSQL…") : qsTr("Desfazendo no PostgreSQL…");
        root.decisionRequested({ previewId: root.active.previewId, decision: decision, name: root.active.name,
            clientContext: root.active.clientContext, expectedContext: root.active.expectedContext });
    }

    function discard() {
        const active = root.active;
        const deciding = root.deciding;
        root.active = null;
        root.deciding = false;
        root.message = "";
        root.remaining = 0;
        if (active !== null && !deciding) {
            root.decisionRequested({ previewId: active.previewId, decision: "rollback", name: active.name,
                clientContext: active.clientContext, expectedContext: active.expectedContext });
        } else if (active === null && root.dataSourceController !== null) {
            const query = root.dataSourceController.lastQuery;
            if (query !== null && query.preview === true && query.jobId) root.cancelExecutionRequested(query.jobId);
        }
    }

    function finished(event) {
        if (root.active === null || event.clientContext !== root.active.clientContext || event.name !== root.active.name) return;
        root.active = null;
        root.deciding = false;
        root.remaining = 0;
    }

    function failed(message, operation) {
        if (root.active === null || !operation || operation.previewId !== root.active.previewId
                || operation.clientContext !== root.active.clientContext) return;
        root.deciding = false;
        root.message = message;
    }
}
