import QtQuick

// Os donos do "Ambiente do projeto": o que se configura ANTES de compilar —
// toolchain e kits, bibliotecas, banco, observabilidade, embarcados,
// containers, instalar ferramentas. Sairam do AppDomains em 2026-09-12 (ele
// estava em 390/400 quando o dominio `container` nasceu): e' a Frente 1 do
// arquitetura/27 — <X>Domain na UI — aplicada ao agrupamento que o menu
// "Ambiente" ja' fazia. Nao ha logica aqui: so instanciacao e fiacao.
Item {
    id: root

    property var coreClient: null
    // O FolderPicker da Start Screen (Main.qml): o seletor de pasta NATIVO
    // que o kit reutiliza para o SDK/sysroot, em vez de um dialogo proprio.
    property var folderPicker: null
    // Para onde Gravar e depurar compoe (fiacao, nao IPC): executar e salvar
    // a gravacao, e o editor dos arquivos da placa. Os donos moram no AppDomains.
    property var runtimeController: null
    property var runConfigController: null
    property var editorController: null

    readonly property alias toolchainController: toolchainController
    readonly property alias libraryController: libraryController
    readonly property alias dataSourceController: dataSourceController
    readonly property alias grafanaController: grafanaController
    readonly property alias platformioController: platformioController
    readonly property alias embeddedController: embeddedController
    readonly property alias setupController: setupController
    readonly property alias containerController: containerController
    readonly property alias remoteController: remoteController

    visible: false

    // Toolchain (roadmap 30, etapa 5): qual executavel cumpre cada papel neste
    // projeto. Sem escolha, tudo e automatico e o PATH continua decidindo.
    ToolchainController {
        id: toolchainController

        workspaceRoot: root.coreClient.workspaceRoot
        // A escolha de pasta vai ao picker existente com o proposito; a
        // resposta volta pelo Main.qml (onFolderPicked -> handlePickedPath).
        onFolderPickRequested: function(purpose, startPath) {
            if (root.folderPicker) {
                root.folderPicker.openFor(purpose, startPath !== "" ? startPath : root.coreClient.homeDir);
            }
        }
    }

    // Bibliotecas C/C++ (roadmaps/35, etapa 19/20): o catalogo curado. Nao
    // escreve arquivo — produz plano, e quem escreve e o configaction.
    LibraryController {
        id: libraryController

        workspaceRoot: root.coreClient.workspaceRoot
    }

    // Fontes de dados (roadmaps/35, etapa 26): o catalogo de conexoes. Guarda
    // o PERFIL, nunca a senha — a decisao esta em `DocsPublic/seguranca/40`.
    DataSourceController {
        id: dataSourceController

        workspaceRoot: root.coreClient.workspaceRoot
    }

    // Observabilidade (roadmaps/35, etapa 27): o Grafana que observa este
    // projeto. Guarda o ENDERECO e a politica, nunca o token — mesma regra da
    // senha de banco. A licenca AGPL do Grafana decide a forma: a IDE conversa
    // com ele pela HTTP API e nunca o embute.
    GrafanaController {
        id: grafanaController

        workspaceRoot: root.coreClient.workspaceRoot
    }

    // O PIO Home (0.191.0, roadmaps/52 §14.45): o servidor do PlatformIO que
    // o core sobe so' em 127.0.0.1. Nao e' do projeto: vive entre workspaces.
    PlatformioController {
        id: platformioController
    }

    // O arquivo que o PIO Home pede (um defeito do `pio check`, um simbolo do
    // inspetor) abre no editor, na linha. Fiacao entre dois donos: aqui.
    Connections {
        target: platformioController

        function onOpenFileRequested(path, line, column) {
            root.editorController.openDiagnostic(path, line, column);
        }
    }

    // Embarcados (roadmaps/35 §5.7): a sonda no USB. O kit (chip, alvo,
    // depurador) continua no toolchainController — e' kit, nao sonda.
    EmbeddedController {
        id: embeddedController

        workspaceRoot: root.coreClient.workspaceRoot
    }

    // A identidade lida pelo canal (E5) SUGERE o chip; quem grava o kit e' o
    // ToolchainController, e so' o chip muda — alvo/sysroot ficam como estao.
    // Fiacao entre dois donos: mora aqui, na composicao, nao em nenhum deles.
    Connections {
        target: embeddedController.identity

        function onKitChipRequested(chip) {
            toolchainController.applyKit(toolchainController.sysroot,
                                         toolchainController.targetTriple, chip);
        }
    }

    // Como instalar o que falta (2026-09-04): passo a passo OFICIAL para a
    // distro detectada. Nao instala nada — mostra e, se o autor pedir, escreve
    // no terminal da IDE.
    SetupController {
        id: setupController
    }

    // Containers (roadmaps/28 §0: Docker e Podman NATIVOS; priorizado em
    // 2026-09-12). O motor e' da MAQUINA, nao do projeto; o compose e' do
    // projeto — por isso o controller conhece o workspaceRoot.
    ContainerController {
        id: containerController

        workspaceRoot: root.coreClient.workspaceRoot
    }

    // O alvo Linux por SSH (P6 fatia 1 do roadmaps/42, 2026-09-17): a Pi como
    // recurso do projeto. Guarda o PERFIL, nunca senha — SSH e' por chave.
    RemoteController {
        id: remoteController

        workspaceRoot: root.coreClient.workspaceRoot
    }

    // Composição de gravação: intenção nativa ou comando manual entre seus donos.
    Connections {
        target: embeddedController.flash

        function onRunRequested(command, flash) {
            root.runtimeController.startRun(command, flash);
        }

        function onSaveRequested(name, command, flash) {
            root.runConfigController.saveConfiguration(name, command, flash);
        }
    }

    // Arquivos na placa (C2): "enviar o arquivo aberto" pergunta ao editor
    // qual e' NO CLIQUE; o baixado abre no editor. Fiacao entre dois donos.
    Connections {
        target: embeddedController.files

        function onUploadCurrentRequested() {
            embeddedController.files.upload(root.editorController.currentFilePath(), "");
        }

        function onOpenLocalRequested(local) {
            root.editorController.openDiagnostic(local, 1, 1);
        }
    }
}
