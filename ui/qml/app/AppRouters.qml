pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A FIACAO IPC: liga cada dono ao CoreClient, num lugar so.
//
// Saiu do AppDomains.qml em 2026-09-03, quando ele cruzou 400 ao ganhar o
// dominio `library`. O AppDomains e composition root, e a `ARCHITECTURE.md` §4
// regra 8 nomeia exatamente este caso: a saida e dividir a composicao POR AREA,
// fazendo a contagem de arquivos crescer — nunca subir o limite.
//
// As duas areas sao distintas e a costura estava visivel no arquivo: as
// primeiras 300 linhas INSTANCIAM os donos; as ultimas 110 LIGAM os donos ao
// IPC. Instanciar e ligar sao coisas diferentes.
//
// **Recebe o DONO, nao 18 copias.** A alternativa era declarar uma property
// por controller e repassar cada uma — dezoito propriedades de passagem, que e
// exatamente o que fez do EditorController uma fachada de 791 linhas e o que o
// ShellEditorHost e o BottomPanelHost corrigiram nesta mesma semana.
Item {
    id: root

    property var domains: null

    visible: false

    WorkspaceEventRouter {
        coreClient: root.domains.coreClient
        folderPicker: root.domains.folderPicker
        projectTree: root.domains.projectTree
        searchEverywhereController: root.domains.searchEverywhereController
        workspaceController: root.domains.workspaceController
        projectHealthController: root.domains.projectHealthController
        recentWorkspacesController: root.domains.recentWorkspacesController
    }

    EditorEventRouter {
        coreClient: root.domains.coreClient
        editorController: root.domains.editorController
        lspStatusController: root.domains.lspStatusController
    }

    EditorRequestRouter {
        coreClient: root.domains.coreClient
        editorController: root.domains.editorController
    }

    CoverageEventRouter {
        coreClient: root.domains.coreClient
        coverageController: root.domains.coverageController
    }

    CoverageRequestRouter {
        coreClient: root.domains.coreClient
        coverageController: root.domains.coverageController
    }

    RemoteEventRouter {
        coreClient: root.domains.coreClient
        remoteController: root.domains.remoteController
    }

    RemoteRequestRouter {
        coreClient: root.domains.coreClient
        remoteController: root.domains.remoteController
        runtimeController: root.domains.runtimeController
    }

    JobsEventRouter {
        coreClient: root.domains.coreClient
        jobsController: root.domains.jobsController
        diagnosticsController: root.domains.diagnosticsController
    }

    SettingsEventRouter {
        coreClient: root.domains.coreClient
        settingsController: root.domains.settingsController
    }

    SearchEventRouter {
        coreClient: root.domains.coreClient
        searchController: root.domains.searchController
        searchEverywhereController: root.domains.searchEverywhereController
    }

    SearchRequestRouter {
        coreClient: root.domains.coreClient
        searchController: root.domains.searchController
        searchEverywhereController: root.domains.searchEverywhereController
        editorController: root.domains.editorController
    }

    ProjectTreeRequestRouter {
        coreClient: root.domains.coreClient
        projectTree: root.domains.projectTree
    }

    RuntimeEventRouter {
        coreClient: root.domains.coreClient
        runtimeController: root.domains.runtimeController
        runConfigController: root.domains.runConfigController
    }

    RuntimeRequestRouter {
        coreClient: root.domains.coreClient
        runtimeController: root.domains.runtimeController
        runConfigController: root.domains.runConfigController
    }

    DebugEventRouter {
        coreClient: root.domains.coreClient
        debugController: root.domains.debugController
    }

    DebugRequestRouter {
        coreClient: root.domains.coreClient
        debugController: root.domains.debugController
    }

    ToolchainEventRouter {
        coreClient: root.domains.coreClient
        toolchainController: root.domains.toolchainController
    }

    ToolchainRequestRouter {
        coreClient: root.domains.coreClient
        toolchainController: root.domains.toolchainController
    }

    LibraryEventRouter {
        coreClient: root.domains.coreClient
        libraryController: root.domains.libraryController
    }

    LibraryRequestRouter {
        coreClient: root.domains.coreClient
        libraryController: root.domains.libraryController
    }

    DataSourceEventRouter {
        coreClient: root.domains.coreClient
        dataSourceController: root.domains.dataSourceController
    }

    DataSourceRequestRouter {
        coreClient: root.domains.coreClient
        dataSourceController: root.domains.dataSourceController
    }

    GrafanaEventRouter {
        coreClient: root.domains.coreClient
        grafanaController: root.domains.grafanaController
    }

    GrafanaRequestRouter {
        coreClient: root.domains.coreClient
        grafanaController: root.domains.grafanaController
    }

    EmbeddedEventRouter {
        coreClient: root.domains.coreClient
        embeddedController: root.domains.embeddedController
        runtimeController: root.domains.runtimeController
    }

    EmbeddedRequestRouter {
        coreClient: root.domains.coreClient
        embeddedController: root.domains.embeddedController
    }

    IndexEventRouter {
        coreClient: root.domains.coreClient
        indexController: root.domains.indexController
        searchEverywhereController: root.domains.searchEverywhereController
    }

    IndexRequestRouter {
        coreClient: root.domains.coreClient
        indexController: root.domains.indexController
        searchEverywhereController: root.domains.searchEverywhereController
    }

    PythonEventRouter {
        coreClient: root.domains.coreClient
        pythonController: root.domains.pythonController
    }

    PythonRequestRouter {
        coreClient: root.domains.coreClient
        pythonController: root.domains.pythonController
    }

    ContainerEventRouter {
        coreClient: root.domains.coreClient
        containerController: root.domains.containerController
        runtimeController: root.domains.runtimeController
    }

    ContainerRequestRouter {
        coreClient: root.domains.coreClient
        containerController: root.domains.containerController
    }

    SetupEventRouter {
        coreClient: root.domains.coreClient
        setupController: root.domains.setupController
    }

    SetupRequestRouter {
        coreClient: root.domains.coreClient
        setupController: root.domains.setupController
    }

    ConfigActionEventRouter {
        coreClient: root.domains.coreClient
        configActionController: root.domains.configActionController
    }

    ConfigActionRequestRouter {
        coreClient: root.domains.coreClient
        configActionController: root.domains.configActionController
    }

    GitEventRouter {
        coreClient: root.domains.coreClient
        gitController: root.domains.gitController
    }

    GitRequestRouter {
        coreClient: root.domains.coreClient
        gitController: root.domains.gitController
        editorController: root.domains.editorController
    }
}
