pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O PROVEDOR DE INSTALACAO de toolchain (integracoes/39 §5, 2026-09-13):
// o catalogo que o core publica, com URL, tamanho e sha256 VISIVEIS antes do
// clique; o botao instala NA PASTA DA IDE, como job, com o checksum conferido
// antes de desempacotar. Nada roda como root; nada e' baixado sem clique.
// Burro: le e pede ao ToolchainController; recomendadas vem primeiro.
Item {
    id: root

    property var toolchainController: null

    readonly property var lista: root.toolchainController
        ? root.toolchainController.installableSorted() : []
    readonly property bool aberto: root.toolchainController !== null
        && root.toolchainController.installCatalogVisible

    implicitHeight: coluna.implicitHeight

    function mib(bytes) {
        return Math.round(Number(bytes) / (1024 * 1024));
    }

    Column {
        id: coluna

        anchors.left: parent.left
        anchors.right: parent.right
        spacing: Theme.spacingXSmall

        Row {
            width: parent.width
            spacing: Theme.spacingSmall

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: qsTr("Toolchains instaláveis (%1, %2 recomendada(s))")
                      .arg(root.lista.length)
                      .arg(root.lista.filter(t => t.recommended === true).length)
                color: Theme.textSecondary
                font.pixelSize: 11
                font.bold: true
            }

            KvButton {
                text: root.aberto ? qsTr("Ocultar catálogo") : qsTr("Mostrar catálogo")
                compact: true
                enabled: root.toolchainController !== null
                onClicked: root.toolchainController.installCatalogVisible = !root.aberto
            }
        }

        Text {
            width: parent.width
            visible: root.aberto
            wrapMode: Text.WordWrap
            text: qsTr("Baixa para %1 com o SHA-256 publicado conferido ANTES de desempacotar; o tar do sistema desempacota. Nada no sistema, nada sem clique.")
                  .arg(root.toolchainController ? root.toolchainController.installRoot : "")
            color: Theme.textMuted
            font.pixelSize: 10
        }

        Text {
            width: parent.width
            visible: text !== ""
            wrapMode: Text.WordWrap
            text: root.toolchainController ? root.toolchainController.lastInstallOutcome : ""
            color: text.indexOf("falhou") === 0 ? Theme.errorSoft : Theme.textSecondary
            font.pixelSize: 10
        }

        ListView {
            id: catalogo

            width: parent.width
            height: root.aberto ? 168 : 0
            visible: root.aberto
            clip: true
            spacing: Theme.spacingXSmall
            model: root.lista

            delegate: Item {
                id: linha

                required property var modelData

                width: catalogo.width
                height: textos.implicitHeight + Theme.spacingXSmall

                Column {
                    id: textos

                    anchors.left: parent.left
                    anchors.right: botao.left
                    anchors.rightMargin: Theme.spacingSmall
                    spacing: 1

                    Text {
                        width: parent.width
                        elide: Text.ElideRight
                        text: (linha.modelData.recommended === true ? "★ " : "")
                              + linha.modelData.label + " · " + linha.modelData.version
                              + " · " + root.mib(linha.modelData.sizeBytes) + " MiB"
                        color: Theme.textPrimary
                        font.pixelSize: 11
                    }

                    Text {
                        width: parent.width
                        elide: Text.ElideMiddle
                        text: linha.modelData.url
                        color: Theme.textMuted
                        font.family: Theme.monoFont
                        font.pixelSize: 9
                    }

                    Text {
                        width: parent.width
                        elide: Text.ElideMiddle
                        text: "sha256 " + linha.modelData.sha256 + " · " + linha.modelData.license
                        color: Theme.textMuted
                        font.family: Theme.monoFont
                        font.pixelSize: 9
                    }
                }

                KvButton {
                    id: botao

                    anchors.right: parent.right
                    anchors.verticalCenter: parent.verticalCenter
                    compact: true
                    text: linha.modelData.installed === true ? qsTr("Instalada")
                          : root.toolchainController.installing === linha.modelData.id
                            ? qsTr("baixando…") : qsTr("Instalar")
                    enabled: linha.modelData.installed !== true
                             && root.toolchainController.installing === ""
                    onClicked: root.toolchainController.install(linha.modelData.id)
                }
            }
        }
    }
}
