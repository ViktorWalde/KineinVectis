pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: root

    property var evidence: null
    readonly property string status: evidence === null ? "" : evidence.status
    readonly property bool newError: status === "newError"
    readonly property bool unverified: status === "notVerified"
    readonly property bool trustRequired: evidence !== null && evidence.reasonCode === "untrustedWorkspace"
    readonly property string heading: newError ? qsTr("Erro novo observado")
        : status === "noNewError" ? qsTr("Sem erro novo observado") : qsTr("Não verificado")

    implicitHeight: 126
    visible: root.evidence !== null

    function details() {
        if (evidence === null) return "";
        let result = evidence.reason || "";
        if (evidence.compiler !== undefined) {
            result += "\n" + evidence.compiler + " · " + (evidence.durationMs || 0) + " ms";
            result += "\n" + qsTr("Erros antes/depois: %1 / %2; versão: %3")
                .arg(evidence.beforeErrors === undefined ? "?" : evidence.beforeErrors)
                .arg(evidence.afterErrors === undefined ? "?" : evidence.afterErrors)
                .arg(evidence.documentVersion === undefined ? "?" : evidence.documentVersion);
            result += "\n" + qsTr("Contexto: %1").arg(evidence.contextId || "?");
            result += "\n" + (evidence.directory || "") + "\n" + (evidence.arguments || []).join(" ");
        }
        const errors = evidence.newErrors || [];
        for (let i = 0; i < errors.length; ++i) {
            result += "\n" + (errors[i].file || "") + ":" + (errors[i].line || "?")
                + ": " + errors[i].message;
        }
        return result;
    }

    Text {
        id: title
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        text: root.heading
        color: root.newError ? Theme.errorSoft : Theme.textPrimary
        font.pixelSize: Theme.fontSizeSmall
        font.bold: true
    }

    Flickable {
        anchors.top: title.bottom
        anchors.topMargin: Theme.spacingXSmall
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        contentWidth: width
        contentHeight: detailText.implicitHeight
        boundsBehavior: Flickable.StopAtBounds
        clip: true

        TextEdit {
            id: detailText
            width: parent.width
            text: root.details()
            readOnly: true
            selectByMouse: true
            wrapMode: TextEdit.Wrap
            color: Theme.textSecondary
            font.pixelSize: Theme.fontSizeCaption
        }
    }
}
