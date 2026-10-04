pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A JANELA DE CONTAINERS (2026-10-03, pedido do autor: a mesma base da janela
// do Banco, com "inspiracao e criatividade" no Docker Desktop, no estilo
// JetBrains da IDE). Acoplada ao layout como o Banco: abre no slot do lado do
// icone (ShellDocks) e, diferente do Banco, tambem SEM projeto — o motor e'
// da maquina.
//
//   Containers  ● podman · rootless     ⟳ ×     motor e o que ele responde
//   [filtrar por nome, imagem ou id ]  parados
//   ▾ EM EXECUCAO · 2
//     ● web    nginx:1.27      8080        com o mouse em cima:
//   ▾ PARADOS · 1                              ■/▶  logs  shell  remover
//     ○ db     postgres:16
//   ▸ IMAGENS · 3
//   ─────────────────────────────────
//   ● web · rodando                   ×     o escolhido (ContainerDetail)
//   imagem nginx:1.27 · id · status · portas com link · acoes
//   compose.yaml   [up] [down]                o compose do projeto, se houver
//
// Burro: o estado e' do ContainerController; as linhas, do ContainerRows.
Rectangle {
    id: root

    property var controller: null

    signal closeRequested()

    radius: Theme.radiusLarge
    color: Theme.background1

    readonly property real minimumWidth: 280

    ContainerRows {
        id: rowsModel

        containers: root.controller ? root.controller.containers : []
        images: root.controller ? root.controller.images : []
        filter: root.controller ? root.controller.filter : ""
    }

    readonly property var selectedRow: {
        const id = root.controller ? root.controller.selectedId : "";
        return id === "" ? null : (rowsModel.rows.find(r => rowsModel.isContainer(r) && r.id === id) || null);
    }

    // ---- cabecalho: titulo, o motor, atualizar, fechar ---------------------

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
            text: qsTr("Containers")
            color: Theme.textPrimary
            font.pixelSize: Theme.fontSizeBody
            font.weight: Font.DemiBold
        }

        Rectangle {
            id: engineDot

            anchors.left: title.right
            anchors.leftMargin: Theme.spacingSmall
            anchors.verticalCenter: parent.verticalCenter
            width: 7
            height: 7
            radius: width / 2
            color: root.controller && root.controller.reachable ? Theme.successSoft
                   : (root.controller && root.controller.statusBusy ? Theme.textMuted : Theme.errorSoft)
        }

        Text {
            anchors.left: engineDot.right
            anchors.leftMargin: Theme.spacingXSmall
            anchors.right: actions.left
            anchors.rightMargin: Theme.spacingXSmall
            anchors.verticalCenter: parent.verticalCenter
            text: !root.controller ? "" : (root.controller.statusBusy ? qsTr("procurando o motor…")
                  : (!root.controller.engineFound ? qsTr("sem motor")
                     : root.controller.engineLabel + (root.controller.status.rootless === true ? " · rootless" : "")
                       + (root.controller.reachable ? "" : qsTr(" · não responde"))))
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeCaption
            elide: Text.ElideRight
        }

        Row {
            id: actions

            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            spacing: 2

            KvIconButton {
                compact: true
                iconName: "refresh"
                tooltip: qsTr("Atualizar (o motor, os containers e as imagens)")
                enabled: root.controller !== null && !root.controller.listBusy
                onClicked: root.controller.refresh()
            }

            KvIconButton {
                compact: true
                iconName: "close"
                tooltip: qsTr("Fechar a janela de Containers")
                onClicked: root.closeRequested()
            }
        }
    }

    // ---- filtro e "parados tambem" -----------------------------------------

    Item {
        id: filterRow

        anchors.top: headerRow.bottom
        anchors.topMargin: Theme.spacingXSmall
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.leftMargin: Theme.spacingSmall
        anchors.rightMargin: Theme.spacingSmall
        height: 24
        visible: root.controller !== null && root.controller.engineFound

        KvTextField {
            id: filterInput

            anchors.left: parent.left
            anchors.right: stoppedChip.left
            anchors.rightMargin: Theme.spacingSmall
            height: parent.height
            codeFont: false
            pixelSize: Theme.fontSizeSmall
            iconName: "search"
            clearable: true
            placeholder: qsTr("filtrar por nome, imagem ou id")
            onEdited: (text) => root.controller.setFilter(text)
            Keys.onEscapePressed: filterInput.clear()
        }

        KvToggleChip {
            id: stoppedChip

            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            switchStyle: true
            labelText: qsTr("Parados")
            tooltip: qsTr("Mostrar também os containers parados")
            active: root.controller ? root.controller.showAll : true
            onToggled: root.controller.setShowAll(!root.controller.showAll)
        }
    }

    // ---- a lista -------------------------------------------------------------

    ContainerListPane {
        id: list

        anchors.top: filterRow.visible ? filterRow.bottom : headerRow.bottom
        anchors.topMargin: Theme.spacingXSmall
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: detailDivider.visible ? detailDivider.top : footer.top
        anchors.margins: Theme.spacingXSmall
        controller: root.controller
        rowsModel: rowsModel
    }

    // ---- o escolhido -----------------------------------------------------------

    Rectangle {
        id: detailDivider

        anchors.bottom: detail.top
        anchors.bottomMargin: Theme.spacingSmall
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.leftMargin: Theme.spacingSmall
        anchors.rightMargin: Theme.spacingSmall
        height: 1
        visible: root.selectedRow !== null
        color: Theme.borderSoft
    }

    ContainerDetail {
        id: detail

        anchors.bottom: footer.top
        anchors.bottomMargin: Theme.spacingSmall
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.leftMargin: Theme.spacingSmall
        anchors.rightMargin: Theme.spacingSmall
        height: visible ? implicitHeight : 0
        visible: root.selectedRow !== null
        row: root.selectedRow
        controller: root.controller
        onCloseRequested: root.controller.select("")
    }

    // ---- compose do projeto ------------------------------------------------------

    Item {
        id: footer

        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.spacingSmall
        height: visible ? Math.max(26, footerText.implicitHeight) : 0
        // Com o motor respondendo, a linha do compose aparece sempre: ligada,
        // ela diz o arquivo; desligada, diz O QUE FALTA (abrir um projeto, ou
        // o projeto nao ter compose.yaml) — o botao nunca promete o que falha.
        visible: root.controller !== null && root.controller.reachable

        Text {
            id: footerText

            anchors.left: parent.left
            anchors.right: composeButtons.left
            anchors.rightMargin: Theme.spacingSmall
            anchors.verticalCenter: parent.verticalCenter
            // Sem compose possivel, so' a frase — inteira — do que falta; os
            // botoes somem (desligados, eles espremiam a frase em "sem …pose").
            wrapMode: Text.Wrap
            maximumLineCount: 2
            elide: Text.ElideRight
            text: !root.controller ? "" : (root.controller.canCompose ? root.controller.composeFile.split("/").pop()
                                                                      : root.controller.composeSummary)
            color: root.controller && root.controller.canCompose ? Theme.textSecondary : Theme.textMuted
            font.family: Theme.monoFont
            font.pixelSize: Theme.fontSizeCaption
        }

        Row {
            id: composeButtons

            visible: root.controller !== null && root.controller.canCompose
            width: visible ? implicitWidth : 0
            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            spacing: Theme.spacingXSmall

            KvButton {
                compact: true
                primary: true
                text: qsTr("compose up")
                enabled: root.controller !== null && root.controller.canCompose
                onClicked: root.controller.composeUp()
            }

            KvButton {
                compact: true
                text: qsTr("compose down")
                enabled: root.controller !== null && root.controller.canCompose
                onClicked: root.controller.composeDown()
            }
        }
    }
}
