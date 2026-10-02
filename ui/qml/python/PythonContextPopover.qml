import QtQuick
import KineinVectis

// O CONTEXTO PYTHON do projeto (0.3.8 F3): o que Executar, os testes e o LSP
// usam — interpretador, de onde ele vem, versao, modulo nativo — e as acoes
// que o core sabe fazer (criar o .venv, instalar os stubs da placa). Aberto
// pelo chip do cabecalho, para baixo. A precedencia do interpretador e' do
// core: a tela mostra e oferece; nao escolhe.
Item {
    id: root

    property var controller: null

    readonly property var interpreter: root.controller.status.interpreter !== undefined
                                       ? root.controller.status.interpreter : null
    readonly property string warning: {
        if (root.interpreter !== null && root.interpreter.warning !== undefined
                && root.interpreter.warning !== null) {
            return String(root.interpreter.warning);
        }
        return root.controller.hasEnvironment ? "" : root.controller.bannerMessage();
    }

    signal dismissRequested()

    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        onClicked: root.dismissRequested()
    }

    Rectangle {
        x: Math.max(Theme.spacingSmall,
                    Math.min(root.controller.menuX, root.width - width - Theme.spacingSmall))
        y: Math.max(Theme.spacingSmall,
                    Math.min(root.controller.menuY, root.height - height - Theme.spacingSmall))
        width: 400
        height: content.height + 2 * Theme.spacingMedium
        radius: Theme.radiusLarge
        color: Theme.background2
        border.color: Theme.borderStrong
        border.width: 1

        MouseArea {
            anchors.fill: parent
        }

        Column {
            id: content

            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.margins: Theme.spacingMedium
            spacing: Theme.spacingSmall

            Column {
                width: parent.width
                spacing: 2

                Text {
                    text: qsTr("Python deste projeto")
                    color: Theme.textPrimary
                    font.pixelSize: Theme.fontSizeLarge
                    font.bold: true
                }

                Text {
                    width: parent.width
                    text: qsTr("O interpretador de Executar, dos testes e do LSP. Quem escolhe é o core: .venv do projeto primeiro, depois o do sistema.")
                    color: Theme.textMuted
                    font.pixelSize: Theme.fontSizeSmall
                    wrapMode: Text.WordWrap
                }
            }

            PythonContextFact {
                width: parent.width
                label: qsTr("Interpretador")
                value: root.interpreter !== null ? root.interpreter.interpreter : qsTr("nenhum encontrado")
                code: root.interpreter !== null
            }

            PythonContextFact {
                width: parent.width
                visible: root.interpreter !== null
                label: qsTr("Origem")
                value: root.controller.hasEnvironment && root.interpreter !== null
                       ? root.interpreter.origin : qsTr("sistema (sem ambiente próprio)")
            }

            PythonContextFact {
                width: parent.width
                visible: root.interpreter !== null && root.interpreter.version !== undefined
                label: qsTr("Versão")
                value: root.interpreter !== null && root.interpreter.version !== undefined
                       ? String(root.interpreter.version).replace(/^Python /, "") : ""
            }

            PythonContextFact {
                width: parent.width
                visible: root.controller.nativeModuleLine() !== ""
                label: qsTr("Módulo nativo")
                value: root.controller.nativeModuleLine()
                detail: root.controller.nativeModuleBuildHint()
            }

            Text {
                width: parent.width
                visible: root.warning !== ""
                text: "⚠ " + root.warning
                color: Theme.warningSoft
                font.pixelSize: Theme.fontSizeSmall
                wrapMode: Text.WordWrap
            }

            Text {
                width: parent.width
                visible: root.controller.lastOutcome !== ""
                text: root.controller.lastOutcome
                color: Theme.textSecondary
                font.pixelSize: Theme.fontSizeSmall
                wrapMode: Text.WordWrap
            }

            Row {
                spacing: Theme.spacingSmall
                topPadding: Theme.spacingXSmall

                KvButton {
                    visible: root.controller.needsEnvironment
                    text: root.controller.creating ? qsTr("Criando…") : root.controller.actionLabel()
                    primary: true
                    enabled: !root.controller.creating
                    onClicked: root.controller.createEnvironment()
                }

                KvButton {
                    visible: root.controller.needsStubs
                    text: root.controller.installingStubs ? qsTr("Instalando…")
                                                          : qsTr("Instalar stubs da placa")
                    enabled: !root.controller.installingStubs
                    onClicked: root.controller.installStubs()
                }

                KvButton {
                    text: qsTr("Verificar de novo")
                    onClicked: root.controller.statusRequested()
                }
            }
        }
    }
}
