pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Window

// A tela de BOAS-VINDAS (Etapa 2 F7, 2026-09-18; redesenhada em 2026-10-03,
// pedidos do autor: um boas-vindas estilizado que apresente a IDE, com um
// degrade suave da paleta do icone e uma onda animada desligavel):
//
//   ┌──────────────────────────────────────────────── [Animação] ┐
//   │              ▣ BETA PUBLICO 0.3.x · LINUX                  │
//   │              Bem-vindo ao Kinein Vectis                    │
//   │              Do codigo a placa,                            │
//   │              no mesmo lugar.  (ambar)                      │
//   │              o que a IDE e', em 2 linhas                   │
//   │              [Criar] [Abrir] [Configuracoes]               │
//   │              Projetos recentes                             │
//   │  ～～～ a onda (ardosia, ambar escuro, ambar) ～～～～～～～  │
//   └────────────────────────────────────────────────────────────┘
//
// So' boas-vindas (decisao do autor, 2026-10-03): sem trilhos, sem janelas,
// sem painel de baixo, sem atalhos de area (o shell cuida disso) e sem a linha
// do Ambiente (ela foi para o painel Ferramentas, dentro do projeto). O fundo
// ambienta; o foco e' o texto e os projetos. Comecar em 1 clique — ou em Enter, no recente em destaque.
Rectangle {
    id: root

    required property var recentWorkspacesController
    // A onda do fundo (preferencia do usuario, 0.151.0).
    property bool animationEnabled: true
    property var urlDecoder: null
    property bool folderDropReady: false

    signal openWorkspaceRequested()
    signal workspacePathDropped(string path)
    signal newProjectRequested(string templateId)
    signal settingsRequested()
    signal animationToggled(bool enabled)

    focus: visible
    // Sem workspace, o teclado e' desta tela: Enter abre o recente em destaque.
    onVisibleChanged: if (visible) forceActiveFocus()
    Component.onCompleted: if (visible) forceActiveFocus()
    Keys.onUpPressed: recentWorkspacesController.moveHighlight(-1)
    Keys.onDownPressed: recentWorkspacesController.moveHighlight(1)
    Keys.onReturnPressed: recentWorkspacesController.openHighlighted()
    Keys.onEnterPressed: recentWorkspacesController.openHighlighted()

    color: Theme.backgroundEditor
    radius: Theme.radiusLarge
    border.color: Theme.accent
    border.width: folderDropReady ? 2 : 0

    function localDropPath(event) {
        if (!event.hasUrls || event.urls.length !== 1 || urlDecoder === null) return "";
        return urlDecoder.localDirectoryPathFromUrls(event.urls);
    }

    function acceptFolderDrop(drop) {
        folderDropReady = false;
        const path = localDropPath(drop);
        if (path === "") { drop.accepted = false; return; }
        drop.accept(Qt.CopyAction);
        workspacePathDropped(path);
    }

    DropArea {
        anchors.fill: parent
        z: 2
        onEntered: function(drag) {
            root.folderDropReady = root.localDropPath(drag) !== "";
            drag.accepted = root.folderDropReady;
        }
        onExited: root.folderDropReady = false
        onDropped: function(drop) { root.acceptFolderDrop(drop); }
    }

    // O fundo: a onda na paleta do icone (WelcomeWave). Ela so' anda com a
    // preferencia ligada, a tela a vista e a janela em uso; abrir um projeto
    // esconde esta tela e a onda para sozinha.
    WelcomeWave {
        anchors.fill: parent
        animated: root.animationEnabled && root.visible && root.Window.active
    }

    // Ligar e desligar a onda, no canto (a escolha fica guardada).
    KvToggleChip {
        anchors.top: parent.top
        anchors.right: parent.right
        anchors.margins: Theme.spacingMedium
        z: 3
        switchStyle: true
        iconName: "observability"
        labelText: qsTr("Animação")
        tooltip: qsTr("A onda do fundo desta tela; desligada, nada se move nem gasta processador")
        active: root.animationEnabled
        onToggled: root.animationToggled(!root.animationEnabled)
    }


    Flickable {
        id: startFlick

        anchors.fill: parent
        contentWidth: width
        contentHeight: Math.max(height, contentColumn.y + contentColumn.height + Theme.spacingRegion)
        clip: true
        boundsBehavior: Flickable.StopAtBounds
        flickableDirection: Flickable.VerticalFlick

        Column {
            id: contentColumn

            width: Math.min(720, startFlick.width - 2 * Theme.spacingRegion)
            x: (startFlick.width - width) / 2
            y: Math.max(Theme.spacingRegion, (startFlick.height - height) / 2)
            spacing: Theme.spacingLarge

            // O selo: a marca, o estagio e a plataforma.
            Row {
                spacing: Theme.spacingSmall

                Rectangle {
                    width: 26
                    height: 26
                    radius: Theme.radius
                    color: "transparent"
                    border.width: 1
                    border.color: Theme.accent

                    Image {
                        anchors.centerIn: parent
                        width: 16
                        height: 16
                        source: "qrc:/KineinVectis/assets/icons/app/kinein-64.png"
                        smooth: true
                        mipmap: true
                    }
                }

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    text: qsTr("BETA PÚBLICO %1 · LINUX").arg(Qt.application.version)
                    color: Theme.accent
                    font.family: Theme.monoFont
                    font.pixelSize: Theme.fontSizeCaption
                    font.weight: Font.DemiBold
                    font.letterSpacing: 1.2
                }
            }

            // Boas-vindas e a IDE apresentada (texto proprio, nao o do site).
            Text {
                width: parent.width
                text: qsTr("Bem-vindo ao Kinein Vectis")
                color: Theme.textSecondary
                font.pixelSize: Theme.fontSizeLarge
            }

            Text {
                width: parent.width
                wrapMode: Text.WordWrap
                textFormat: Text.StyledText
                text: qsTr("Do código à placa,<br><font color=\"%1\">no mesmo lugar.</font>").arg(Theme.accent)
                color: Theme.textPrimary
                font.pixelSize: Theme.fontSizeHero
                font.weight: Font.Bold
                lineHeight: 1.05
            }

            Text {
                width: parent.width
                wrapMode: Text.WordWrap
                text: qsTr("C, C++, Rust e Python — de um programa no computador ao firmware de um microcontrolador ou um Linux embarcado. Abra uma pasta ou crie um projeto: a IDE reconhece o build, o compilador e a placa, e mostra o que vai fazer antes de fazer.")
                color: Theme.textSecondary
                font.pixelSize: Theme.fontSizeMedium
                lineHeight: 1.25
            }

            // As tres acoes como CARTOES que dizem para que servem (0.3.8,
            // retorno do autor). UM gesto para criar, e a linguagem se escolhe
            // dentro dele (53 §13.0 item 6).
            Row {
                id: actionTiles

                width: parent.width
                spacing: Theme.spacingMedium

                StartActionTile {
                    width: (actionTiles.width - 2 * actionTiles.spacing) / 3
                    primary: true
                    iconName: "add"
                    title: qsTr("Criar projeto")
                    description: qsTr("C/C++, Rust, Python ou uma pasta vazia — você vê os arquivos antes")
                    onActivated: root.newProjectRequested("")
                }

                StartActionTile {
                    width: (actionTiles.width - 2 * actionTiles.spacing) / 3
                    iconName: "project"
                    title: qsTr("Abrir projeto")
                    description: qsTr("Uma pasta que já existe — ou arraste a pasta para esta tela")
                    onActivated: root.openWorkspaceRequested()
                }

                StartActionTile {
                    width: (actionTiles.width - 2 * actionTiles.spacing) / 3
                    iconName: "settings"
                    title: qsTr("Configurações")
                    description: qsTr("Fonte, salvar sozinho, rigor do build")
                    onActivated: root.settingsRequested()
                }
            }

            RecentWorkspacesCard {
                width: parent.width
                controller: root.recentWorkspacesController
            }
        }
    }
}
