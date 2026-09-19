pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Um patch unificado, linha a linha, com a classe de cada uma pela regra
// pura (GitRules.lineKind). Saiu do antigo GitDiffDialog para servir ao
// painel da direita da HUD do Git (2026-09-18).
ListView {
    id: view

    property string patch: ""
    property bool loading: false
    property string emptyText: qsTr("Sem mudanças.")

    GitRules { id: rules }

    clip: true
    model: patch === "" ? [] : patch.split("\n")

    function colorFor(kind) {
        if (kind === "add") return Theme.successSoft;
        if (kind === "del") return Theme.errorSoft;
        if (kind === "hunk") return Theme.accent;
        if (kind === "meta") return Theme.textMuted;
        return Theme.textSecondary;
    }

    Text {
        anchors.centerIn: parent
        visible: view.patch === ""
        text: view.loading ? qsTr("Carregando…") : view.emptyText
        color: Theme.textMuted
        font.pixelSize: 11
    }

    delegate: Rectangle {
        id: linha

        required property string modelData

        readonly property string kind: rules.lineKind(modelData)

        width: view.width
        height: texto.implicitHeight + 1
        color: kind === "add" ? Qt.rgba(0.49, 0.81, 0.42, 0.10)
               : kind === "del" ? Qt.rgba(0.83, 0.37, 0.37, 0.10)
               : kind === "hunk" ? Theme.surface2 : "transparent"

        Text {
            id: texto

            x: Theme.spacingSmall
            width: parent.width - 2 * Theme.spacingSmall
            text: linha.modelData === "" ? " " : linha.modelData
            color: view.colorFor(linha.kind)
            font.family: Theme.monoFont
            font.pixelSize: 11
            wrapMode: Text.WrapAnywhere
        }
    }
}
