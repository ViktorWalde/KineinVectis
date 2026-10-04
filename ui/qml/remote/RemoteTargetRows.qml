pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Os alvos SSH do projeto, na janela acoplada (2026-10-04). O ESCOLHIDO vira
// um cartao com o estado da ultima sonda e a acao principal DENTRO dele (a
// acao e' sobre aquele alvo). Os outros sao linhas com o ultimo contato.
//
//   ┌ ● rpi            pi@192.168.0.20:22 ┐   o ponto: verde respondeu,
//   │   aarch64 · Linux 6.6.31            │   vermelho falhou, ambar
//   └ [            Sondar             ]   ┘   primeira conexao, cinza nunca
//     ○ servidor  respondeu ha' 2 h          deploy@srv:2222
Column {
    id: root

    property var controller: null
    // A acao principal (RemoteActionRules.primaryFor): { label, kind, enabled, busy, hint }.
    property var action: ({ label: "", kind: "", enabled: false, busy: false, hint: "" })

    signal primaryTriggered(string kind)

    readonly property var targets: root.controller ? root.controller.targets : []
    readonly property bool firstContact: root.controller !== null && root.controller.trust.isFirstContact(root.controller.probeFailure)

    function probedThis(name) {
        return root.controller !== null && root.controller.probedName === name;
    }

    // O relogio do "ha' 3 min": anda so' com a janela a vista.
    property real nowSeconds: Date.now() / 1000

    Timer {
        interval: 30000
        repeat: true
        running: root.visible
        triggeredOnStart: true
        onTriggered: root.nowSeconds = Date.now() / 1000
    }

    // O ponto: a sonda desta sessao, senao o ultimo contato guardado no
    // projeto (0.153.0) — todos os alvos, nao so' o escolhido.
    function stateColor(name) {
        const state = root.probedThis(name) ? (root.controller.probeOk ? "ok" : "fail")
                                            : root.controller.contacts.state(name);
        if (root.probedThis(name) && root.firstContact) return Theme.warningSoft; // pergunta, nao erro
        return state === "ok" ? Theme.successSoft : (state === "fail" ? Theme.errorSoft : Theme.textMuted);
    }

    function stateText(name) {
        const c = root.controller;
        if (c.probing && c.selectedName === name) return qsTr("sondando…");
        if (!root.probedThis(name)) return c.contacts.describe(name, root.nowSeconds);
        return c.probeOk ? c.probeArch + " · " + c.probeKernel
                         : (c.probeMessage !== "" ? c.probeMessage : qsTr("a última sonda falhou"));
    }

    function destination(target) {
        return (target.user ? target.user + "@" : "") + target.host + (target.port ? ":" + target.port : "");
    }

    spacing: 2

    // Sem alvo, ou com um alvo configurado e AINDA NAO SALVO (colar a linha ssh
    // ou escolher um alias com outro alvo escolhido): o cartao com o gesto que
    // resolve — configurar, depois salvar.
    readonly property bool editingNew: root.targets.length > 0 && root.controller !== null
                                       && root.controller.selectedName === ""

    Rectangle {
        visible: root.targets.length === 0 || root.editingNew
        width: root.width
        height: emptyColumn.implicitHeight + 2 * Theme.spacingMedium
        radius: Theme.radiusLarge
        color: Theme.background0
        border.width: 1
        border.color: root.editingNew ? Theme.accentDim : Theme.borderSoft

        Column {
            id: emptyColumn

            anchors.fill: parent
            anchors.margins: Theme.spacingMedium
            spacing: Theme.spacingSmall

            Row {
                spacing: Theme.spacingSmall

                KvIcon {
                    anchors.verticalCenter: parent.verticalCenter
                    name: "remote"
                    size: 16
                }

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    text: !root.editingNew ? qsTr("Nenhum alvo neste projeto")
                          : (root.controller.draft.name.trim() !== ""
                             ? qsTr("%1 · não salvo").arg(root.controller.draft.name.trim())
                             : qsTr("Nenhum alvo escolhido"))
                    color: Theme.textSecondary
                    font.pixelSize: Theme.fontSizeBody
                }
            }

            KvButton {
                width: parent.width
                primary: true
                text: root.action.label
                enabled: root.action.enabled
                onClicked: root.primaryTriggered(root.action.kind)
            }

            Text {
                width: parent.width
                text: root.action.hint
                color: Theme.textMuted
                font.pixelSize: Theme.fontSizeCaption
                wrapMode: Text.WordWrap
            }
        }
    }

    Repeater {
        model: root.targets

        delegate: Rectangle {
            id: row

            required property var modelData

            readonly property bool selected: root.controller !== null && root.controller.selectedName === row.modelData.name

            width: root.width
            height: row.selected ? card.implicitHeight + 2 * Theme.spacingMedium : 44
            radius: row.selected ? Theme.radiusLarge : Theme.radius
            color: row.selected ? Theme.background0 : (rowHover.hovered ? Theme.surface2 : "transparent")
            border.width: row.selected ? 1 : 0
            border.color: Theme.accentDim
            clip: true

            Behavior on height {
                NumberAnimation { duration: Theme.motionMedium; easing.type: Theme.easingStandard }
            }

            Behavior on color {
                ColorAnimation { duration: Theme.motionFast }
            }

            HoverHandler {
                id: rowHover
            }

            // Clicar escolhe; no escolhido o clique fica com os botoes.
            MouseArea {
                anchors.fill: parent
                enabled: !row.selected
                cursorShape: Qt.PointingHandCursor
                onClicked: root.controller.select(row.modelData.name)
            }

            Column {
                id: card

                anchors.top: parent.top
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.margins: row.selected ? Theme.spacingMedium : 0
                anchors.leftMargin: Theme.spacingMedium
                anchors.rightMargin: Theme.spacingSmall
                spacing: Theme.spacingSmall

                Item {
                    width: parent.width
                    height: row.selected ? 20 : 44

                    Rectangle {
                        id: stateDot

                        anchors.left: parent.left
                        anchors.verticalCenter: parent.verticalCenter
                        width: 8
                        height: 8
                        radius: width / 2
                        color: root.stateColor(row.modelData.name)
                    }

                    Column {
                        id: nameText

                        anchors.left: stateDot.right
                        anchors.leftMargin: Theme.spacingSmall
                        anchors.verticalCenter: parent.verticalCenter
                        width: parent.width * 0.5

                        Text {
                            width: Math.min(implicitWidth, parent.width)
                            text: row.modelData.name
                            color: row.selected ? Theme.textPrimary : Theme.textSecondary
                            font.pixelSize: Theme.fontSizeBody
                            font.weight: row.selected ? Font.DemiBold : Font.Normal
                            elide: Text.ElideRight
                        }

                        // Os outros alvos dizem o ultimo contato na propria linha.
                        Text {
                            visible: !row.selected
                            width: parent.width
                            text: root.controller.contacts.describe(row.modelData.name, root.nowSeconds)
                            color: Theme.textMuted
                            font.pixelSize: Theme.fontSizeMicro
                            elide: Text.ElideRight
                        }
                    }

                    Text {
                        anchors.left: nameText.right
                        anchors.leftMargin: Theme.spacingSmall
                        anchors.right: parent.right
                        anchors.verticalCenter: parent.verticalCenter
                        horizontalAlignment: Text.AlignRight
                        text: root.destination(row.modelData)
                        color: Theme.textMuted
                        font.family: Theme.monoFont
                        font.pixelSize: Theme.fontSizeMicro
                        elide: Text.ElideMiddle
                    }
                }

                Text {
                    visible: row.selected
                    width: parent.width
                    leftPadding: 16
                    text: root.stateText(row.modelData.name)
                    readonly property bool failed: root.probedThis(row.modelData.name) && !root.controller.probeOk
                                                   && !root.controller.probing

                    // Primeira conexao nao e' erro: e' uma pergunta (ambar).
                    color: failed && root.firstContact ? Theme.warningSoft
                           : failed ? Theme.errorSoft
                           : (root.probedThis(row.modelData.name) ? Theme.textSecondary : Theme.textMuted)
                    font.family: root.probedThis(row.modelData.name) && root.controller.probeOk ? Theme.monoFont : Theme.uiFont
                    font.pixelSize: Theme.fontSizeCaption
                    wrapMode: Text.WordWrap
                    maximumLineCount: 4
                    elide: Text.ElideRight
                }

                // Primeira conexao (0.153.0): quem o servidor diz ser, ANTES de
                // confiar. A acao principal, logo abaixo, grava esta chave.
                RemoteHostKeyBox {
                    width: parent.width
                    visible: row.selected && root.controller.trust.name === row.modelData.name
                             && root.controller.trust.showing
                    trust: root.controller.trust
                }

                // A linha armada (copiar a chave) a vista ANTES de rodar.
                Rectangle {
                    visible: row.selected && root.controller.armedCommand !== ""
                    width: parent.width
                    height: armedText.implicitHeight + 2 * Theme.spacingSmall
                    radius: Theme.radius
                    color: Theme.background1

                    Text {
                        id: armedText

                        anchors.fill: parent
                        anchors.margins: Theme.spacingSmall
                        wrapMode: Text.WrapAnywhere
                        text: "$ " + root.controller.armedCommand
                        color: Theme.textPrimary
                        font.family: Theme.monoFont
                        font.pixelSize: Theme.fontSizeCaption
                    }
                }

                KvButton {
                    visible: row.selected
                    width: parent.width
                    primary: true
                    text: root.action.label
                    enabled: root.action.enabled
                    onClicked: root.primaryTriggered(root.action.kind)
                }

                Text {
                    visible: row.selected
                    width: parent.width
                    text: root.action.hint
                    color: Theme.textMuted
                    font.pixelSize: Theme.fontSizeCaption
                    wrapMode: Text.WordWrap
                }
            }
        }
    }
}
