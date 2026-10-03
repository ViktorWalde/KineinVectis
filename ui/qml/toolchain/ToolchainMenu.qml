pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Seletor da toolchain (reformulado na 0.3.8 F3, no padrao do painel de
// areas: nada de lista plana numa cor so'). Uma linha por PAPEL com o que o
// build vai usar; o clique abre as opcoes daquele papel, um de cada vez. Os
// papeis DESTE PROJETO (pelos sistemas de build) vem primeiro; os outros
// ficam recolhidos em "Outros papéis".
//
// "Automático" e' o padrao de cada papel: sem escolha, o core nao fixa nada e
// o `PATH` decide — o comportamento historico. So' aparecem candidatos
// DETECTADOS nesta maquina: oferecer um compilador ausente seria oferecer um
// configure que vai falhar.
Item {
    id: root

    property var controller: null
    property var buildSystems: []
    property real menuX: 0
    property real menuY: 0
    property bool menuBelow: false

    readonly property var roles: [
        { key: "cxxCompiler", label: qsTr("Compilador C++") },
        { key: "cCompiler", label: qsTr("Compilador C") },
        { key: "generator", label: qsTr("Gerador") },
        { key: "cmake", label: qsTr("CMake") },
        { key: "cargo", label: qsTr("Cargo") },
        // O papel existia no core desde 2026-09-03 (etapa 22) e a lista o
        // omitia: ninguem escolhia o probe-rs pela tela (roadmaps/35 §5.7).
        { key: "debugAdapter", label: qsTr("Depurador") },
        // O processo que abre na aba de terminal sobre a porta da placa.
        { key: "serialMonitor", label: qsTr("Monitor serial") }
    ]
    // Que papeis cada sistema de build usa. Sem sistema, todos sao do projeto.
    readonly property var rolesBySystem: ({
        cargo: ["cargo", "debugAdapter"],
        cmake: ["cxxCompiler", "cCompiler", "generator", "cmake", "debugAdapter"],
        make: ["cxxCompiler", "cCompiler", "debugAdapter"],
        platformIo: ["debugAdapter", "serialMonitor"]
    })
    readonly property var projectKeys: {
        const keys = [];
        for (let i = 0; i < root.buildSystems.length; i++) {
            const used = root.rolesBySystem[root.buildSystems[i]];
            if (used === undefined) continue;
            for (const key of used) {
                if (keys.indexOf(key) < 0) keys.push(key);
            }
        }
        return keys.length > 0 ? keys : root.roles.map(role => role.key);
    }
    readonly property var projectRoles: root.roles.filter(
        role => root.projectKeys.indexOf(role.key) >= 0 && root.hasOptions(role.key))
    readonly property var otherRoles: root.roles.filter(
        role => root.projectKeys.indexOf(role.key) < 0 && root.hasOptions(role.key))
    readonly property bool usesCmake: root.projectKeys.indexOf("cmake") >= 0

    property string expandedKey: ""
    property bool showOthers: false

    signal dismissRequested()

    function hasOptions(key) {
        return root.controller.candidatesFor(key).length > 0;
    }

    function toggle(key) {
        root.expandedKey = root.expandedKey === key ? "" : key;
    }

    onVisibleChanged: if (visible) {
        root.expandedKey = "";
        root.showOthers = false;
    }

    KvBackdrop {
        anchors.fill: parent
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        onClicked: root.dismissRequested()
    }

    Rectangle {
        id: card

        // Para baixo do ponto (o chip do cabecalho) ou para cima (o rodape,
        // o menu Ambiente); sempre dentro da janela, rolando se nao couber.
        readonly property real room: root.menuBelow ? root.height - root.menuY - Theme.spacingSmall
                                                    : root.menuY - Theme.spacingSmall
        readonly property real wanted: content.height + 2 * Theme.spacingMedium

        x: Math.max(Theme.spacingSmall,
                    Math.min(root.menuX, root.width - width - Theme.spacingSmall))
        y: Math.max(Theme.spacingSmall,
                    Math.min(root.menuBelow ? root.menuY : root.menuY - height,
                             root.height - height - Theme.spacingSmall))
        width: 380
        height: Math.min(wanted, Math.max(200, room))
        radius: Theme.radiusLarge
        color: Theme.background2
        border.color: Theme.borderStrong
        border.width: 1

        MouseArea {
            anchors.fill: parent
        }

        Flickable {
            anchors.fill: parent
            anchors.margins: Theme.spacingMedium
            contentHeight: content.height
            clip: true
            boundsBehavior: Flickable.StopAtBounds

            Column {
                id: content

                width: parent.width
                spacing: 2

                Text {
                    text: qsTr("Toolchain deste projeto")
                    color: Theme.textPrimary
                    font.pixelSize: Theme.fontSizeLarge
                    font.bold: true
                }

                Text {
                    width: parent.width
                    bottomPadding: Theme.spacingSmall
                    text: qsTr("O que o build vai usar. Automático segue o PATH; escolher fixa.")
                    color: Theme.textMuted
                    font.pixelSize: Theme.fontSizeSmall
                    wrapMode: Text.WordWrap
                }

                Text {
                    width: parent.width
                    text: root.controller.errorText
                    visible: root.controller.errorText !== ""
                    color: Theme.errorSoft
                    font.pixelSize: Theme.fontSizeSmall
                    wrapMode: Text.WordWrap
                }

                Repeater {
                    model: root.projectRoles

                    delegate: ToolchainRoleRow {
                        required property var modelData

                        width: content.width
                        controller: root.controller
                        roleKey: modelData.key
                        roleLabel: modelData.label
                        expanded: root.expandedKey === modelData.key
                        onToggleRequested: root.toggle(modelData.key)
                    }
                }

                Text {
                    visible: root.projectRoles.length === 0
                    width: parent.width
                    text: qsTr("Nenhuma ferramenta detectada para este projeto.")
                    color: Theme.textMuted
                    font.pixelSize: Theme.fontSizeBody
                    wrapMode: Text.WordWrap
                }

                Rectangle {
                    visible: root.otherRoles.length > 0
                    width: parent.width
                    height: 30
                    radius: Theme.radius
                    color: othersArea.containsMouse ? Theme.surface2 : "transparent"

                    Text {
                        anchors.left: parent.left
                        anchors.leftMargin: Theme.spacingMedium
                        anchors.verticalCenter: parent.verticalCenter
                        text: (root.showOthers ? qsTr("Esconder outros papéis")
                                               : qsTr("Outros papéis (%1)").arg(root.otherRoles.length))
                        color: Theme.textMuted
                        font.pixelSize: Theme.fontSizeSmall
                        font.bold: true
                    }

                    MouseArea {
                        id: othersArea

                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: root.showOthers = !root.showOthers
                    }
                }

                Repeater {
                    model: root.showOthers ? root.otherRoles : []

                    delegate: ToolchainRoleRow {
                        required property var modelData

                        width: content.width
                        controller: root.controller
                        roleKey: modelData.key
                        roleLabel: modelData.label
                        expanded: root.expandedKey === modelData.key
                        onToggleRequested: root.toggle(modelData.key)
                    }
                }

                Text {
                    visible: root.usesCmake
                    width: parent.width
                    topPadding: Theme.spacingSmall
                    text: qsTr("Vale na próxima configuração do CMake.")
                    color: Theme.textMuted
                    font.pixelSize: Theme.fontSizeCaption
                    wrapMode: Text.WordWrap
                }
            }
        }
    }
}
