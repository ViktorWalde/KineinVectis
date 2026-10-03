pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A JANELA DO BANCO, na area da esquerda (2026-10-03, decisao do autor: "algo
// mais parecido com o painel da JetBrains", o explorador acoplado ao layout e
// o SQL no proprio editor da IDE). Alterna com o Projeto e o Git no mesmo
// slot. Em cima, a arvore das conexoes salvas (conexao › esquema › tabela ›
// coluna); embaixo, o que responde NESTA MÁQUINA. Abrir uma conexao le a
// estrutura dela se ainda nao leu; passar o mouse mostra "console" (o SQL no
// editor) e "editar" (o dialogo da conexao); clique duplo numa tabela traz os
// dados para a secao de baixo DESTA janela — o contexto fica num lugar so'.
//
// Burro: o estado e' do DataSourceController; o que a janela pede sai por
// sinal.
Rectangle {
    id: root

    property var controller: null

    signal consoleRequested(string name)
    signal tableDataRequested(string connection, string engine, string schema, string table)
    signal editRequested(string name)
    signal newRequested()
    signal candidateChosen(int index)
    signal closeRequested()

    // A secao de dados (o resultado do console ou da tabela) aberta, e a
    // fracao da janela que ela ocupa.
    property bool resultsOpen: false
    property real resultsShare: 0.5

    function showResults() {
        root.resultsOpen = true;
    }

    // DIMENSIONAMENTO (2026-10-03, pedido do autor: "um minimo, ou uma
    // adaptacao minima e automatica"): abaixo de 260 o cabecalho e a arvore
    // perdem o nome; com dados abertos, a janela pede a largura em que a
    // grade cabe sem rolar (o shell so' ALARGA, ate' o teto que deixa o
    // editor com 480 px, e nunca encolhe o que a pessoa escolheu).
    readonly property real minimumWidth: 260
    readonly property real desiredWidth: root.resultsOpen
                                         ? Math.min(640, results.naturalWidth + 2 * Theme.spacingSmall) : 0
    signal widenRequested(real width)
    onDesiredWidthChanged: if (root.desiredWidth > root.width) root.widenRequested(root.desiredWidth)

    radius: Theme.radiusLarge
    color: Theme.background1

    DataSourceTree {
        id: tree

        profiles: root.controller ? root.controller.profiles : []
        structures: root.controller ? root.controller.structures : ({})
        readingNames: root.controller ? root.controller.readingNames : ({})
    }

    // ---- o cabecalho: titulo, nova conexao, ler de novo, fechar ------------

    Row {
        id: headerRow

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.spacingSmall
        height: 24
        spacing: Theme.spacingSmall

        Text {
            anchors.verticalCenter: parent.verticalCenter
            width: parent.width - 3 * (24 + parent.spacing)
            text: qsTr("Banco")
            color: Theme.textPrimary
            font.pixelSize: Theme.fontSizeBody
            font.weight: Font.DemiBold
        }

        KvIconButton {
            anchors.verticalCenter: parent.verticalCenter
            compact: true
            iconName: "add"
            tooltip: qsTr("Nova conexão")
            onClicked: root.newRequested()
        }

        KvIconButton {
            anchors.verticalCenter: parent.verticalCenter
            compact: true
            iconName: "refresh"
            tooltip: qsTr("Procurar bancos nesta máquina")
            enabled: root.controller !== null && !root.controller.discovery.discovering
            onClicked: root.controller.discovery.discover()
        }

        KvIconButton {
            anchors.verticalCenter: parent.verticalCenter
            compact: true
            iconName: "close"
            tooltip: qsTr("Fechar a janela do Banco")
            onClicked: root.closeRequested()
        }
    }

    // ---- a arvore -----------------------------------------------------------

    DatabaseTreeView {
        id: treeView

        anchors.top: headerRow.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: root.resultsOpen ? divider.top : machineSection.top
        anchors.margins: Theme.spacingXSmall
        controller: root.controller
        treeModel: tree
        onConsoleRequested: name => root.consoleRequested(name)
        onEditRequested: name => root.editRequested(name)
        onTableDataRequested: (connection, engine, schema, table) =>
            root.tableDataRequested(connection, engine, schema, table)
    }

    // ---- os dados, embaixo da arvore e na mesma janela ----------------------

    // A divisoria arrastavel: a fracao da janela que os dados ocupam.
    Rectangle {
        id: divider

        readonly property real span: root.height - headerRow.height - 2 * Theme.spacingSmall

        visible: root.resultsOpen
        x: 0
        y: root.height - Theme.spacingSmall - root.resultsShare * divider.span - height
        width: root.width
        height: 7
        color: "transparent"

        Rectangle {
            anchors.centerIn: parent
            width: parent.width - 2 * Theme.spacingSmall
            height: 1
            color: dividerArea.containsMouse || dividerArea.pressed ? Theme.accent : Theme.borderSoft
        }

        MouseArea {
            id: dividerArea

            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.SizeVerCursor
            onPositionChanged: mouse => {
                if (!pressed) return;
                const y = divider.mapToItem(root, 0, mouse.y).y;
                root.resultsShare = Math.max(0.2, Math.min(0.85,
                    (root.height - Theme.spacingSmall - y) / divider.span));
            }
        }
    }

    DataSourceResultsPanel {
        id: results

        anchors.top: divider.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.leftMargin: Theme.spacingSmall
        anchors.rightMargin: Theme.spacingSmall
        anchors.bottomMargin: Theme.spacingSmall
        visible: root.resultsOpen
        controller: root.controller
        onCloseRequested: root.resultsOpen = false
    }

    // ---- nesta maquina --------------------------------------------------------

    Column {
        id: machineSection

        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.margins: Theme.spacingSmall
        spacing: 2
        // Com os dados abertos, o espaco e' deles; fechar os dados traz a
        // descoberta de volta.
        visible: !root.resultsOpen
        height: visible ? implicitHeight : 0

        Rectangle { width: parent.width; height: 1; color: Theme.borderSoft }

        Text {
            topPadding: Theme.spacingXSmall
            text: root.controller && root.controller.discovery.discovering ? qsTr("NESTA MÁQUINA — PROCURANDO…")
                                                                           : qsTr("NESTA MÁQUINA")
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeMicro
            font.weight: Font.DemiBold
            font.letterSpacing: 0.8
        }

        Repeater {
            model: root.controller ? root.controller.discovery.candidates : []

            delegate: Rectangle {
                id: machineRow

                required property var modelData
                required property int index

                width: machineSection.width
                height: 24
                radius: Theme.radius
                color: machineArea.containsMouse ? Theme.surface2 : "transparent"

                KvIcon {
                    id: machineIcon

                    anchors.left: parent.left
                    anchors.leftMargin: Theme.spacingXSmall
                    anchors.verticalCenter: parent.verticalCenter
                    size: 14
                    name: DataSourceKinds.engineIcon(machineRow.modelData.profile.engine)
                    disabled: !machineRow.modelData.running
                }

                Text {
                    anchors.left: machineIcon.right
                    anchors.leftMargin: Theme.spacingXSmall
                    anchors.right: parent.right
                    anchors.verticalCenter: parent.verticalCenter
                    text: machineRow.modelData.label
                    color: Theme.textSecondary
                    font.pixelSize: Theme.fontSizeSmall
                    elide: Text.ElideRight
                }

                MouseArea {
                    id: machineArea

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.candidateChosen(machineRow.index)
                }
            }
        }

        Text {
            visible: root.controller !== null && !root.controller.discovery.discovering
                     && root.controller.discovery.candidates.length === 0
            text: qsTr("nada respondeu")
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeCaption
        }
    }
}
