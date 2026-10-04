pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A JANELA DO REMOTO (2026-10-04, roadmap 59 §4): o alvo Linux por SSH
// acoplado ao layout como o Banco e os Containers — abre no slot do lado do
// icone, fica aberto enquanto se edita, e lembra a largura por projeto. Antes
// era um pop-up de 680 px no meio da tela, por cima do codigo, que fechava ao
// clicar fora.
//
//   Remoto  rpi · aarch64                          ×
//   ALVOS · 2
//   ┌ ● rpi            pi@192.168.0.20 ┐        o escolhido e' um cartao:
//   │   aarch64 · Linux 6.6            │        o ponto e' a ultima sonda, e
//   │ [          Sondar           ]    │        a acao principal (Remote-
//   └ mede arquitetura, kernel e…      ┘        ActionRules) mora COM o alvo
//     ○ servidor       deploy@srv:2222
//   Visão  Projeto  Executar  Sistema  ⚙         abas numa linha
//   ━━━━━
//   a seccao aberta (RemoteSectionPages)
//
// A coluna estreita trocou as duas colunas do pop-up (alvos | secoes): os
// alvos em cima, e as abas quebram a linha em vez de sumir.
Rectangle {
    id: root

    property var controller: null

    signal closeRequested()

    radius: Theme.radiusLarge
    color: Theme.background1

    readonly property real minimumWidth: 300
    readonly property var c: root.controller
    readonly property bool draftNamed: root.c !== null && root.c.draft !== null && root.c.draft.name.trim() !== ""

    RemoteActionRules {
        id: rules
    }

    // O proximo gesto, um so', derivado do estado medido.
    readonly property var action: rules.primaryFor({
        "savedTarget": root.c !== null && root.c.selectedSaved,
        "namedDraft": root.draftNamed,
        "probing": root.c !== null && root.c.probing,
        "probed": root.c !== null && root.c.probedName !== "",
        "probeOk": root.c !== null && root.c.probeOk,
        "failure": root.c ? root.c.probeFailure : "",
        "isMirror": root.c !== null && root.c.workspace.isMirror,
        "syncing": root.c !== null && root.c.workspace.syncing,
        "hasRemoteFolder": root.c !== null && root.c.workspace.openPath.trim() !== "",
        "armedLine": root.c !== null && root.c.armedCommand !== "",
        "serverKeyRead": root.c !== null && root.c.trust.keys.length > 0,
        "trusting": root.c !== null && root.c.trust.trusting,
        "keySent": root.c !== null && root.c.keySent,
        "firstContact": root.c !== null && root.c.trust.isFirstContact(root.c.probeFailure)
    })

    // A acao primaria vira gesto. Quando o gesto vive noutra seccao, LEVA a
    // pessoa ate' la' em vez de deixar um botao que ela nao acha.
    function runPrimary(kind) {
        const c = root.c;
        switch (kind) {
        case "configurar":
            c.selectSection("configurar");
            break;
        case "salvar":
            c.selectSection("configurar");
            c.save();
            break;
        case "sondar":
            c.probe();
            break;
        case "copiarChave":
            c.copyId();
            break;
        case "rodarArmada":
            c.runArmed();
            break;
        case "confiar":
            c.trust.trust();
            break;
        case "abrirPasta":
            c.selectSection("workspace");
            if (c.workspace.openPath.trim() === "") c.workspace.startBrowse();
            else c.workspace.openFolder();
            break;
        case "puxar":
            c.selectSection("workspace");
            c.workspace.sync("pull");
            break;
        default:
            break;
        }
    }

    // ---- cabecalho: titulo, o alvo escolhido, novo, fechar -----------------

    Item {
        id: headerRow

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.spacingSmall
        height: 24

        Text {
            id: title

            anchors.left: parent.left
            anchors.verticalCenter: parent.verticalCenter
            text: qsTr("Remoto")
            color: Theme.textPrimary
            font.pixelSize: Theme.fontSizeBody
            font.weight: Font.DemiBold
        }

        Text {
            anchors.left: title.right
            anchors.leftMargin: Theme.spacingSmall
            anchors.right: headerActions.left
            anchors.rightMargin: Theme.spacingXSmall
            anchors.verticalCenter: parent.verticalCenter
            text: !root.c || root.c.selectedName === "" ? qsTr("Linux por SSH")
                  : root.c.selectedName + (root.c.probedName === root.c.selectedName && root.c.probeOk && root.c.probeArch !== ""
                                           ? " · " + root.c.probeArch : "")
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeCaption
            elide: Text.ElideRight
        }

        Row {
            id: headerActions

            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            spacing: 2

            KvIconButton {
                compact: true
                iconName: "close"
                tooltip: qsTr("Fechar a janela do Remoto")
                onClicked: root.closeRequested()
            }
        }
    }

    // ---- os alvos, a acao primaria e as secoes ------------------------------

    Column {
        id: top

        anchors.top: headerRow.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.leftMargin: Theme.spacingSmall
        anchors.rightMargin: Theme.spacingSmall
        spacing: Theme.spacingSmall

        Text {
            leftPadding: Theme.spacingXSmall
            text: qsTr("ALVOS") + (root.c && root.c.targets.length > 0 ? "  ·  " + root.c.targets.length : "")
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeMicro
            font.weight: Font.DemiBold
            font.letterSpacing: 0.6
        }

        // Muitos alvos rolam aqui, sem empurrar as secoes para fora. O
        // escolhido e' um cartao com a acao principal dentro.
        Flickable {
            width: parent.width
            height: Math.min(targetRows.implicitHeight, 300)
            contentHeight: targetRows.implicitHeight
            clip: true
            boundsBehavior: Flickable.StopAtBounds

            RemoteTargetRows {
                id: targetRows

                width: parent.width
                controller: root.c
                action: root.action
                onPrimaryTriggered: kind => root.runPrimary(kind)
            }
        }

        RemoteSections {
            width: parent.width
            current: root.c ? root.c.section : "visao"
            sections: [
                { "id": "visao", "label": qsTr("Visão") },
                { "id": "workspace", "label": qsTr("Projeto") },
                { "id": "executar", "label": qsTr("Executar") },
                { "id": "sistema", "label": qsTr("Sistema") },
                { "id": "configurar", "icon": "settings", "tooltip": qsTr("Configurar o alvo") }
            ]
            onSelected: id => root.c.selectSection(id)
        }

    }

    RemoteSectionPages {
        anchors.top: top.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: footer.visible ? footer.top : parent.bottom
        anchors.margins: Theme.spacingSmall
        controller: root.c
    }

    // O perfil se salva e se remove onde se edita: em Configurar.
    Row {
        id: footer

        anchors.bottom: parent.bottom
        anchors.right: parent.right
        anchors.margins: Theme.spacingSmall
        visible: root.c !== null && root.c.configuring
        spacing: Theme.spacingSmall

        KvButton {
            compact: true
            danger: true
            text: qsTr("Remover")
            enabled: root.c !== null && root.c.selectedName !== ""
            onClicked: root.c.remove()
        }

        KvButton {
            compact: true
            primary: true
            text: qsTr("Salvar")
            enabled: root.draftNamed
            onClicked: root.c.save()
        }
    }
}
