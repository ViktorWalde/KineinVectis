pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A aba Testes: a ARVORE dos testes listados (test.discover, 2026-09-13) com o
// status do ultimo run e um "rodar so' este" por linha; sem arvore, os casos
// que rodaram; embaixo, a saida bruta do runner (e' onde o pytest explica a
// falha). Burro: le e pede ao JobsController.
Item {
    id: panel

    property var controller: null
    property bool running: false

    ListModel {
        id: modeloVazio
    }

    readonly property var discovered: panel.controller ? panel.controller.discoveredModel : modeloVazio
    readonly property var cases: panel.controller ? panel.controller.testModel : modeloVazio
    readonly property var output: panel.controller ? panel.controller.testOutputModel : modeloVazio
    readonly property string summary: panel.controller ? panel.controller.testSummary : ""
    readonly property bool hasTree: panel.discovered.count > 0
    readonly property bool hasOutput: panel.output.count > 0

    function statusColor(status) {
        if (status === "passed") {
            return Theme.successSoft;
        }
        if (status === "failed") {
            return Theme.errorSoft;
        }
        return Theme.textMuted;
    }

    Row {
        id: cabecalho

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        spacing: Theme.spacingSmall

        KvButton {
            text: panel.controller && panel.controller.discovering ? qsTr("listando…") : qsTr("Listar testes")
            compact: true
            enabled: panel.controller !== null && !panel.controller.discovering && !panel.running
            onClicked: panel.controller.discoverTests("")
        }

        Text {
            anchors.verticalCenter: parent.verticalCenter
            visible: panel.summary !== ""
            text: panel.summary
            color: Theme.textSecondary
            font.pixelSize: 11
            font.bold: true
        }
    }

    ListView {
        id: lista

        anchors.top: cabecalho.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.bottom: panel.hasOutput ? outputSeparator.top : parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        clip: true
        spacing: 1
        model: panel.hasTree ? panel.discovered : panel.cases
        onCountChanged: if (!panel.hasTree) positionViewAtEnd()

        VerticalScrollBar {
            parent: lista
            anchors.right: lista.right
            anchors.top: lista.top
            anchors.bottom: lista.bottom
            contentSize: lista.contentHeight
            viewportSize: lista.height
            position: lista.contentY
            onMoveRequested: function(position) {
                lista.contentY = position;
            }
        }

        Text {
            anchors.centerIn: parent
            visible: lista.count === 0 && !panel.running
            text: qsTr("Nenhum teste. Listar testes mostra a árvore; Testes (Ctrl+Shift+F9) roda tudo.")
            color: Theme.textMuted
            font.pixelSize: 11
        }

        delegate: Item {
            id: linha

            required property var model

            width: lista.width
            height: 20

            Rectangle {
                id: ponto

                width: 7
                height: 7
                radius: 4
                anchors.verticalCenter: parent.verticalCenter
                color: panel.statusColor(linha.model.status)
            }

            Text {
                anchors.left: ponto.right
                anchors.leftMargin: Theme.spacingSmall
                anchors.right: botaoUm.visible ? botaoUm.left : parent.right
                anchors.rightMargin: Theme.spacingSmall
                anchors.verticalCenter: parent.verticalCenter
                text: panel.hasTree && linha.model.file !== undefined && linha.model.file !== ""
                      ? linha.model.file + "  " + linha.model.name : linha.model.name
                color: linha.model.status === "failed" ? Theme.textPrimary : Theme.textSecondary
                font.family: Theme.monoFont
                font.pixelSize: 11
                elide: Text.ElideMiddle
            }

            // Rodar SO' este: o id exato que o core deu (pytest posicional,
            // cargo --exact, ctest -R ancorado).
            KvIconButton {
                id: botaoUm

                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                visible: panel.hasTree
                iconName: "run"
                tooltip: qsTr("Rodar só este teste")
                compact: true
                enabled: !panel.running && panel.controller !== null
                onClicked: panel.controller.runOneTest(linha.model.id, "")
            }
        }
    }

    Rectangle {
        id: outputSeparator

        visible: panel.hasOutput
        height: visible ? 1 : 0
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: testOutputView.top
        anchors.bottomMargin: visible ? Theme.spacingSmall : 0
        color: Theme.borderSoft
    }

    ListView {
        id: testOutputView

        visible: panel.hasOutput
        height: visible ? Math.floor(panel.height * 0.45) : 0
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        clip: true
        model: panel.output
        onCountChanged: positionViewAtEnd()

        VerticalScrollBar {
            parent: testOutputView
            anchors.right: testOutputView.right
            anchors.top: testOutputView.top
            anchors.bottom: testOutputView.bottom
            contentSize: testOutputView.contentHeight
            viewportSize: testOutputView.height
            position: testOutputView.contentY
            onMoveRequested: function(position) {
                testOutputView.contentY = position;
            }
        }

        delegate: Text {
            required property string line

            width: testOutputView.width - 16
            text: line
            color: Theme.textSecondary
            font.family: Theme.monoFont
            font.pixelSize: 11
            wrapMode: Text.NoWrap
            elide: Text.ElideRight
        }
    }
}
