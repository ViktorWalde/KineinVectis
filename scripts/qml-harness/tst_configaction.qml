// Configuration Actions: o estado da UI do ciclo list -> preview -> apply.
//
// Por que existe: o `ConfigActionController` guarda o CONSENTIMENTO. Se ele
// mandar ao core um `expected` errado — ou nenhum —, o Apply deixa de ser
// "aplique o que eu vi" e vira "sobrescreva o que estiver la". Isso nao quebra
// build, nao acende qmllint e nao aparece na tela: a acao continua funcionando
// no caso feliz. E' exatamente a forma de falha silenciosa que a §4 regra 11
// manda cobrir.
//
// O outro alvo e' a resposta ATRASADA: preview de uma acao chegando depois de
// o usuario ter trocado de acao pintaria o diff de A embaixo do titulo de B —
// e o Apply gravaria o de A.
import QtQuick
import "../../ui/qml/configaction"

Item {
    id: root

    property var lastPreviewId: ""
    property var lastApply: null
    property int listRequests: 0

    // Duble do LibraryController com a superficie que o painel unificado usa.
    QtObject {
        id: bibliotecas

        property var libraries: []
        property string selecionada: ""
        property var plan: null
        property string errorText: ""
        property string selectedId: ""
        property string target: ""

        // `librariesChanged` NAO e' declarado: toda `property` do QML ja' cria
        // o sinal de mudanca dela, e declarar de novo e' erro de carga. E' o
        // mesmo sinal que o ConfigActionController escuta.
        function select(id) { bibliotecas.selecionada = id; }
        function listRequested() {}
    }

    ConfigActionController {
        id: controller

        onListRequested: root.listRequests += 1
        onPreviewRequested: function (id, params) {
            root.lastPreviewId = id;
        }
        onApplyRequested: function (id, params, expected) {
            root.lastApply = { id: id, params: params, expected: expected };
        }
    }

    function actionsFixture() {
        return [
            {
                id: "cmake.addExecutable",
                title: "Adicionar executavel",
                description: "Cria um target",
                scope: "cmake",
                category: "CMake Targets",
                risk: "medium",
                // O SHAPE E' O DO CORE: `riskLabel` e `riskExplanation`
                // entraram em 2026-09-04, quando a previa deixou de mostrar a
                // palavra crua do enum. Fixture atrasada testa um protocolo
                // que nao existe mais.
                riskLabel: "edita configuração",
                riskExplanation: "edita um arquivo de configuração do projeto",
                effect: "edit",
                state: "available",
                affects: ["CMakeLists.txt"],
                params: [
                    { name: "name", label: "Nome do target", required: true, placeholder: "app" },
                    { name: "sources", label: "Fontes", required: true, placeholder: "src/main.cpp" }
                ],
                docs: []
            },
            {
                id: "cmake.createDebugPreset",
                title: "Criar preset Debug",
                description: "Preset debug",
                scope: "cmake",
                category: "CMake Basic",
                risk: "medium",
                // O SHAPE E' O DO CORE: `riskLabel` e `riskExplanation`
                // entraram em 2026-09-04, quando a previa deixou de mostrar a
                // palavra crua do enum. Fixture atrasada testa um protocolo
                // que nao existe mais.
                riskLabel: "edita configuração",
                riskExplanation: "edita um arquivo de configuração do projeto",
                effect: "edit",
                state: "unavailable",
                reason: "o preset debug ja existe em CMakePresets.json",
                affects: ["CMakePresets.json"],
                params: [],
                docs: []
            }
        ];
    }

    Component.onCompleted: {
        let failures = 0;

        controller.handleListed(root.actionsFixture(), ["cmake"]);
        if (controller.actionsModel.count !== 2) failures += 1;

        // Acao com parametro obrigatorio vazio nao pede preview: mandar um
        // pedido que o core recusaria e' ruido, nao diagnostico.
        root.lastPreviewId = "";
        controller.select("cmake.addExecutable");
        if (root.lastPreviewId !== "") failures += 2;
        if (controller.missingRequiredParam() !== "Nome do target") failures += 4;

        // Com todos os obrigatorios preenchidos, o preview sai.
        controller.setParam("name", "app");
        controller.setParam("sources", "src/main.cpp");
        controller.requestPreview();
        if (root.lastPreviewId !== "cmake.addExecutable") failures += 8;

        // Acao indisponivel nunca pede preview, mesmo selecionada.
        root.lastPreviewId = "";
        controller.select("cmake.createDebugPreset");
        if (root.lastPreviewId !== "") failures += 16;
        if (controller.canPreview()) failures += 32;

        // Resposta ATRASADA de outra acao e' descartada: sem isso, o diff de
        // uma acao apareceria sob o titulo de outra.
        controller.handlePreviewed({
            id: "cmake.addExecutable",
            title: "Adicionar executavel",
            summary: "resumo antigo",
            files: [{ path: "CMakeLists.txt", before: "antes\n", after: "depois\n" }],
            report: [],
            notes: []
        });
        if (controller.previewId !== "") failures += 64;

        // Sem preview vigente, o Apply NAO dispara.
        root.lastApply = null;
        controller.apply();
        if (root.lastApply !== null) failures += 128;

        // Ciclo completo: preview da acao selecionada, e o Apply devolve ao
        // core o `before` que o preview mostrou (compare-before-save).
        controller.select("cmake.addExecutable");
        controller.setParam("name", "app");
        controller.setParam("sources", "src/main.cpp");
        controller.requestPreview();
        controller.handlePreviewed({
            id: "cmake.addExecutable",
            title: "Adicionar executavel",
            summary: "Acrescenta add_executable(app)",
            files: [{ path: "CMakeLists.txt", before: "antes\n", after: "depois\n" }],
            report: [],
            notes: ["cuidado"]
        });
        if (controller.previewSummary !== "Acrescenta add_executable(app)") failures += 256;
        if (controller.previewNotes.length !== 1) failures += 512;

        controller.apply();
        if (root.lastApply === null) {
            failures += 1024;
        } else {
            if (root.lastApply.expected.length !== 1) failures += 2048;
            if (root.lastApply.expected[0].path !== "CMakeLists.txt") failures += 4096;
            if (root.lastApply.expected[0].content !== "antes\n") failures += 8192;
        }

        // Arquivo que sera CRIADO nao tem `before`: o `expected` precisa
        // OMITIR o content, porque "nao existia" e' diferente de "vazio".
        controller.handlePreviewed({
            id: "cmake.addExecutable",
            title: "Adicionar executavel",
            summary: "cria",
            files: [{ path: "CMakePresets.json", after: "{}\n" }],
            report: [],
            notes: []
        });
        root.lastApply = null;
        controller.apply();
        if (root.lastApply === null) {
            failures += 16384;
        } else if (root.lastApply.expected[0].content !== undefined) {
            failures += 32768;
        }

        // Depois de aplicar, a lista e' repedida: o estado das acoes mudou
        // junto com o disco.
        const before = root.listRequests;
        controller.handleApplied("cmake.addExecutable", "feito", ["CMakeLists.txt"], "");
        if (root.listRequests !== before + 1) failures += 65536;
        if (controller.previewId !== "") failures += 131072;

        // Erro de OUTRO dominio nao pode acender o erro deste dialogo.
        controller.handleFailed("git.status", "nada a ver");
        if (controller.errorText !== "") failures += 262144;
        controller.handleFailed("configAction.apply", "o arquivo mudou");
        if (controller.errorText !== "o arquivo mudou") failures += 524288;

        // A LISTA E' UMA SO' (2026-09-04): acoes e bibliotecas lado a lado.
        //
        // O autor relatou que "Biblioteca C/C++" e "Configure Actions" eram
        // duas telas para a MESMA coisa — ativar algo no projeto — e que por
        // isso o Cargo parecia inalcancavel. A juncao nao inventa camada: uma
        // biblioteca ja' E' um pacote de acoes de configuracao.
        controller.libraryController = bibliotecas;
        bibliotecas.libraries = [
            { id: "fmt", name: "fmt", summary: "formatacao", license: "MIT",
              pinnedVersion: "12.2.0", applied: true },
            { id: "zlib", name: "zlib", summary: "compressao", license: "Zlib",
              pinnedVersion: "1.3.1", applied: false }
        ];
        controller.rebuildList();

        // As duas bibliotecas entraram DEPOIS das acoes, sem apagar nenhuma.
        const total = controller.actionsModel.count;
        if (total < 4) failures += 1048576;
        const penultima = controller.actionsModel.get(total - 2);
        const ultima = controller.actionsModel.get(total - 1);
        if (penultima.actionId !== "library:fmt") failures += 2097152;
        // A LINGUA DE ESTADO E' A MESMA das acoes: `alreadyApplied` pinta de
        // verde nos dois casos. Duas telas com a mesma bolinha significando
        // coisas diferentes seria pior que nao ter bolinha.
        if (penultima.actionState !== "alreadyApplied") failures += 4194304;
        if (ultima.actionState !== "available") failures += 8388608;

        // Escolher uma biblioteca NAO pede preview de acao: quem monta o plano
        // dela e' o dominio `library`, e o painel so' escolhe qual visao abrir.
        root.lastPreviewId = "";
        controller.select("library:zlib");
        if (!controller.isLibrary(controller.selectedId)) failures += 16777216;
        if (root.lastPreviewId !== "") failures += 33554432;
        if (bibliotecas.selecionada !== "zlib") failures += 67108864;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
