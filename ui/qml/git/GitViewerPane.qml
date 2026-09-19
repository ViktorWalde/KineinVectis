import QtQuick
import KineinVectis

// O diff / o commit selecionado na janela do Git, ABERTO NO EDITOR (E3-3,
// roadmaps/44 §4.1): uma aba de visualizacao sobre o EditorPane — o
// cabecalho parece a faixa de abas ("diff: caminho" / "commit abc1234 —
// resumo") com o x, e embaixo o GitInspectorPane de sempre. NAO e' uma
// aba do modelo do editor (o EditorController esta' em debito de catraca);
// fechar (x, Esc) volta ao editor como estava. Le do GitInspectorController.
Rectangle {
    id: root

    property var inspector: null
    // O caminho do workspace, para a aba dizer o relativo de uma mudanca.
    property string workspaceRoot: ""

    readonly property bool active: inspector !== null && inspector !== undefined && inspector.active
    readonly property string tabTitle: {
        if (!active) return "";
        if (inspector.isCommit) return qsTr("commit %1 — %2").arg(inspector.shortSha).arg(inspector.summary);
        const p = inspector.path;
        const rel = workspaceRoot !== "" && p.indexOf(workspaceRoot + "/") === 0
                    ? p.substring(workspaceRoot.length + 1) : p;
        return qsTr("diff: %1").arg(rel);
    }

    visible: active
    radius: Theme.radiusLarge
    color: Theme.background1
    border.color: Theme.borderSoft
    border.width: 1

    function close() {
        if (inspector) inspector.clear();
    }

    Keys.onEscapePressed: root.close()

    // A "aba": uma so', selecionada, com o x.
    Rectangle {
        id: faixa

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        height: 34
        color: Theme.background0
        radius: Theme.radiusLarge

        Rectangle {
            anchors.left: parent.left
            anchors.bottom: parent.bottom
            anchors.leftMargin: Theme.spacingSmall
            width: Math.min(parent.width - 2 * Theme.spacingSmall, tituloAba.implicitWidth + 44)
            height: 30
            radius: Theme.radius
            color: Theme.surfaceSelected
            border.color: Theme.borderSoft
            border.width: 1

            KvIcon {
                id: iconeAba

                anchors.left: parent.left
                anchors.leftMargin: Theme.spacingSmall
                anchors.verticalCenter: parent.verticalCenter
                name: "git"
                size: 13
                active: true
            }

            Text {
                id: tituloAba

                anchors.left: iconeAba.right
                anchors.right: fecharAba.left
                anchors.leftMargin: Theme.spacingXSmall
                anchors.rightMargin: Theme.spacingXSmall
                anchors.verticalCenter: parent.verticalCenter
                text: root.tabTitle
                color: Theme.textPrimary
                font.pixelSize: 11
                elide: Text.ElideMiddle
            }

            KvIconButton {
                id: fecharAba

                anchors.right: parent.right
                anchors.rightMargin: Theme.spacingXSmall
                anchors.verticalCenter: parent.verticalCenter
                compact: true
                iconName: "close"
                tooltip: qsTr("Fechar e voltar ao editor (Esc)")
                onClicked: root.close()
            }
        }
    }

    GitInspectorPane {
        anchors.top: faixa.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.margins: Theme.spacingSmall
        inspector: root.inspector
    }
}
