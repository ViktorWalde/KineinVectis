pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Seletor da toolchain: um papel por bloco, os candidatos detectados abaixo.
//
// "Automático" e sempre a primeira opcao de cada papel, e e o padrao: sem
// escolha, o core nao fixa nada e o `PATH` continua decidindo — que e o
// comportamento historico da IDE. Escolher e' opt-in, e some do comando quando
// o usuario libera.
//
// So aparecem candidatos DETECTADOS nesta maquina. Oferecer um compilador
// ausente seria oferecer um configure que vai falhar.
Item {
    id: root

    property var controller: null
    property real menuX: 0
    property real menuY: 0

    readonly property var roles: [
        { key: "cxxCompiler", label: qsTr("Compilador C++") },
        { key: "cCompiler", label: qsTr("Compilador C") },
        { key: "generator", label: qsTr("Gerador") },
        { key: "cmake", label: qsTr("CMake") },
        { key: "cargo", label: qsTr("Cargo") },
        // O papel existia no core desde 2026-09-03 (etapa 22) e esta lista o
        // omitia: ninguem conseguia escolher o probe-rs pela tela. Fio ligado
        // em 2026-09-11 (roadmaps/35 §5.7).
        { key: "debugAdapter", label: qsTr("Depurador") },
        // Monitor serial (2026-09-12): o processo que abre na aba de terminal
        // sobre a porta da placa. Nasceu ja' na lista, para nao repetir o
        // buraco do debugAdapter.
        { key: "serialMonitor", label: qsTr("Monitor serial") }
    ]

    signal dismissRequested()

    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        onClicked: root.dismissRequested()
    }

    Rectangle {
        x: Math.max(Theme.spacingSmall,
                    Math.min(root.menuX, root.width - width - Theme.spacingSmall))
        y: Math.max(Theme.spacingSmall,
                    Math.min(root.menuY - height, root.height - height - Theme.spacingSmall))
        width: 280
        height: coluna.height + 2 * Theme.spacingSmall
        radius: Theme.radius
        color: Theme.background2
        border.color: Theme.borderStrong
        border.width: 1

        MouseArea {
            anchors.fill: parent
        }

        Column {
            id: coluna

            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.margins: Theme.spacingSmall
            spacing: 2

            Text {
                text: qsTr("Toolchain deste projeto")
                color: Theme.textPrimary
                font.pixelSize: 12
                font.bold: true
            }

            Text {
                width: parent.width
                text: root.controller.errorText
                visible: root.controller.errorText !== ""
                color: Theme.errorSoft
                font.pixelSize: 10
                wrapMode: Text.WordWrap
            }

            Repeater {
                model: root.roles

                Column {
                    id: bloco

                    required property var modelData

                    readonly property var opcoes: root.controller.candidatesFor(
                                                      bloco.modelData.key)

                    width: coluna.width
                    spacing: 1
                    visible: opcoes.length > 0

                    Item {
                        width: 1
                        height: Theme.spacingXSmall
                    }

                    Text {
                        text: bloco.modelData.label
                        color: Theme.textMuted
                        font.pixelSize: 10
                    }

                    // "Automático" primeiro: e o padrao, e sair de uma escolha
                    // precisa ser tao facil quanto entrar nela.
                    Rectangle {
                        readonly property bool ativo: {
                            const selecao = root.controller.selectionFor(bloco.modelData.key);
                            return selecao === null || selecao.id === undefined;
                        }

                        width: coluna.width
                        height: 22
                        radius: Theme.radiusXSmall
                        color: automaticoArea.containsMouse ? Theme.surface2 : "transparent"

                        Text {
                            anchors.verticalCenter: parent.verticalCenter
                            anchors.left: parent.left
                            anchors.leftMargin: Theme.spacingSmall
                            text: (parent.ativo ? "✓ " : "   ") + qsTr("automático (PATH)")
                            color: parent.ativo ? Theme.accent : Theme.textSecondary
                            font.pixelSize: 11
                        }

                        MouseArea {
                            id: automaticoArea

                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: root.controller.choose(bloco.modelData.key, "")
                        }
                    }

                    Repeater {
                        model: bloco.opcoes

                        Rectangle {
                            id: opcao

                            required property var modelData

                            readonly property bool ativo: {
                                const selecao = root.controller.selectionFor(bloco.modelData.key);
                                return selecao !== null && selecao.id === opcao.modelData.id;
                            }

                            width: coluna.width
                            height: 22
                            radius: Theme.radiusXSmall
                            color: opcaoArea.containsMouse ? Theme.surface2 : "transparent"

                            Text {
                                anchors.verticalCenter: parent.verticalCenter
                                anchors.left: parent.left
                                anchors.leftMargin: Theme.spacingSmall
                                anchors.right: parent.right
                                anchors.rightMargin: Theme.spacingSmall
                                text: (opcao.ativo ? "✓ " : "   ") + opcao.modelData.label
                                color: opcao.ativo ? Theme.accent : Theme.textSecondary
                                font.pixelSize: 11
                                elide: Text.ElideMiddle
                            }

                            MouseArea {
                                id: opcaoArea

                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: root.controller.choose(bloco.modelData.key,
                                                                  opcao.modelData.id)
                            }
                        }
                    }
                }
            }

            Text {
                width: coluna.width
                text: qsTr("Vale na próxima configuração do CMake.")
                color: Theme.textMuted
                font.pixelSize: 10
                wrapMode: Text.WordWrap
            }
        }
    }
}
