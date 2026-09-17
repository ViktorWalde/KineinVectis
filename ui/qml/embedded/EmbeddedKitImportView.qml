pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O GERENCIADOR DE TOOLCHAIN QUE LE O DISCO (42 §8 itens b e d, 2026-09-13):
// um caminho, dois botoes. "Ler sysroot" diz o que a pasta contem (headers,
// bibliotecas, libc, .pc). "Importar kit" le um SDK Yocto (environment-setup),
// uma arvore Buildroot (output/host) ou uma pasta de toolchain, e mostra a
// PROPOSTA — compiladores, sysroot, alvo, arquivo de toolchain — antes de
// "Aplicar proposta ao kit". Nada e' gravado sem o segundo clique. Burro:
// le e pede ao ToolchainController.
Item {
    id: root

    property var toolchainController: null

    readonly property var linhas: root.toolchainController
        ? root.toolchainController.proposalLines() : []

    implicitHeight: coluna.implicitHeight

    Column {
        id: coluna

        anchors.left: parent.left
        anchors.right: parent.right
        spacing: Theme.spacingXSmall

        Text {
            text: qsTr("Sysroot e SDK do alvo")
            color: Theme.textSecondary
            font.pixelSize: 11
            font.bold: true
        }

        EmbeddedKitField {
            id: campoCaminho

            labelText: qsTr("Pasta/SDK")
            placeholder: qsTr("environment-setup-* (Yocto), output/host (Buildroot), zephyr-sdk-* ou pasta com bin/")
            // O seletor de pasta escreve aqui; a digitacao tambem vale.
            value: root.toolchainController ? root.toolchainController.importPath : ""
        }

        Row {
            width: parent.width
            spacing: Theme.spacingSmall

            // O navegador de pastas da propria IDE (o da Start Screen), nao
            // um dialogo do sistema: a mesma listagem do core, o mesmo visual.
            KvButton {
                text: qsTr("Escolher pasta…")
                compact: true
                enabled: root.toolchainController !== null
                onClicked: root.toolchainController.pickImportPath()
            }

            KvButton {
                text: qsTr("Ler sysroot")
                compact: true
                enabled: root.toolchainController !== null && campoCaminho.text.trim() !== ""
                onClicked: root.toolchainController.inspectSysroot(campoCaminho.text)
            }

            KvButton {
                text: qsTr("Importar kit")
                compact: true
                enabled: root.toolchainController !== null && campoCaminho.text.trim() !== ""
                onClicked: root.toolchainController.importKit(campoCaminho.text)
            }

            KvButton {
                text: qsTr("Aplicar proposta ao kit")
                primary: true
                compact: true
                visible: root.toolchainController !== null && root.toolchainController.hasKitProposal
                onClicked: root.toolchainController.applyProposal()
            }
        }

        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            visible: text !== ""
            text: root.toolchainController ? root.toolchainController.sysrootSummary() : ""
            color: Theme.textMuted
            font.pixelSize: 10
        }

        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            visible: text !== ""
            text: root.toolchainController ? root.toolchainController.importError : ""
            color: Theme.errorSoft
            font.pixelSize: 10
        }

        Repeater {
            model: root.linhas

            Text {
                id: linhaProposta

                required property string modelData

                width: coluna.width
                elide: Text.ElideMiddle
                text: linhaProposta.modelData
                color: Theme.textSecondary
                font.family: Theme.monoFont
                font.pixelSize: 9
            }
        }

        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            visible: text !== ""
            text: root.toolchainController && root.toolchainController.toolchainFile !== ""
                  ? qsTr("toolchain file do kit: %1").arg(root.toolchainController.toolchainFile) : ""
            color: Theme.textMuted
            font.family: Theme.monoFont
            font.pixelSize: 9
        }
    }
}
