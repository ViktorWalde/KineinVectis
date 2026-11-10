// C1-23 (consolidação C §17): o menu de configuração do CMake. O
// ProjectHealthController, a ProjectHealthBanner e o AppMenuPopup REAIS: a
// ativa marcada, "Automático", a pasta cortada no meio com o caminho inteiro
// na dica, a pasta desconhecida que não se escolhe e diz por quê, e a faixa
// que dá a volta ao automático quando não há configuração ativa nem presets.
import QtQuick
import KineinVectis

Item {
    id: root
    width: 900
    height: 400

    property int configureRequests: 0
    property string chosen: "nenhuma"
    property int failures: 0

    ProjectHealthController {
        id: controller

        workspaceRoot: "/tmp/proj"
        workspaceTrusted: true
        workspaceKind: "cmake"
        workspaceBuildSystems: ["cmake"]
        toolsList: [
            { id: "cmake", status: "detected" },
            { id: "ninja", status: "detected" },
            { id: "clangd", status: "detected" },
            { id: "gxx", status: "detected" }
        ]

        onAutoConfigureRequested: root.configureRequests += 1
    }

    ProjectHealthBanner {
        id: banner

        width: parent.width
        menuSpace: root
        active: controller.active
        status: controller.status
        message: controller.message
        actionLabel: controller.actionLabel
    }

    AppMenuPopup {
        id: popup

        y: 40
        width: parent.width
        height: 360
        visible: false
        items: []
        onActionRequested: action => root.chosen = action
    }

    function check(condition, label) {
        if (!condition) {
            root.failures += 1;
            console.error("FALHOU: " + label);
        }
    }

    readonly property string longPath: "/mnt/discos de trabalho/projetos com nomes compridos/saídas do build/clang-release-com-sanitizers"
    readonly property var choices: [
        { preset: "debug", displayName: "Depuração", buildDir: "build/debug", configured: false },
        { preset: "release", buildDir: "build/release", configured: true },
        { preset: "ci/linux", buildDirRefused: "o preset \"ci/linux\" não declara binaryDir, e o nome não serve de "
                                               + "pasta em .kinein/presets: declare binaryDir no preset ou mude o nome",
          configured: false },
        { preset: "elsewhere", buildDir: root.longPath, configured: true }
    ]

    function delegates() {
        const list = [];
        const frame = popup.children[1];
        const view = frame.children.find(child => child.contentItem !== undefined && child.model !== undefined);
        // Os delegates nascem no layout: forçado aqui, sem esperar quadro.
        view.forceLayout();
        for (let i = 0; i < view.contentItem.children.length; i++) {
            const child = view.contentItem.children[i];
            if (child.modelData !== undefined) list.push(child);
        }
        return list.sort((a, b) => a.index - b.index);
    }

    function textsOf(item) {
        const texts = [];
        for (let i = 0; i < item.children.length; i++) {
            const child = item.children[i];
            if (child.elide !== undefined) texts.push(child);
        }
        return texts;
    }

    function areaOf(item) {
        for (let i = 0; i < item.children.length; i++) {
            if (item.children[i].hoverEnabled !== undefined) return item.children[i];
        }
        return null;
    }

    Timer {
        interval: 50
        running: true
        onTriggered: {
            // A configuração escolhida: marcada no menu, e a faixa não avisa.
            controller.handleCmakeStatus(true, false, "", "release", [], "",
                                         { directory: "build/release", origin: "configuration",
                                           preset: "release", selectedConfiguration: "release" },
                                         { preset: "release", source: "configuration",
                                           directory: "build/release", choices: root.choices });
            const menu = controller.configuration;
            check(controller.status === "ok", "escolhida e configurada: a faixa some: " + controller.message);
            check(menu.offered && menu.summary() === "release", "o chip: " + menu.summary());
            check(menu.tooltip().indexOf("build/release") >= 0, "a dica do chip diz a pasta: " + menu.tooltip());
            const items = menu.items();
            check(items.length === 6, "quatro presets, o separador e o automatico: " + items.length);
            check(items.map(item => item.icon || "").join(",") === ",check,,,,",
                  "so' a escolhida marcada: " + JSON.stringify(items.map(item => item.icon)));
            check(items[0].label === "Depuração" && items[0].action === "cmake.configuration:debug"
                  && items[0].tooltip === "build/debug — ainda sem configure",
                  "o displayName e a pasta sem configure: " + JSON.stringify(items[0]));
            check(!items[2].enabled && items[2].detail === "pasta desconhecida"
                  && items[2].tooltip.indexOf("não serve de pasta") >= 0,
                  "pasta desconhecida nao se escolhe e diz por que: " + JSON.stringify(items[2]));
            check(items[3].detail === root.longPath && items[3].tooltip === root.longPath,
                  "a pasta fora da raiz inteira no detalhe e na dica: " + JSON.stringify(items[3]));
            check(items[4].separator === true && !items[4].enabled, "o separador nao se escolhe");
            check(items[5].label === "Automático" && items[5].action === "cmake.configuration:"
                  && items[5].icon === "" && items[5].detail === "" && items[5].enabled,
                  "o automatico desmarcado: " + JSON.stringify(items[5]));

            // O menu na tela: mais largo, a pasta cortada no meio, e a dica.
            popup.items = items;
            popup.visible = true;
            check(popup.menuWidth === 440, "o menu com pastas e' mais largo: " + popup.menuWidth);
            const rows = root.delegates();
            check(rows.length === 6, "seis linhas desenhadas: " + rows.length);
            const longRow = rows[3];
            const detail = root.textsOf(longRow).find(text => text.text === root.longPath);
            check(detail !== undefined && detail.elide === Text.ElideMiddle && detail.truncated,
                  "o caminho longo e' cortado no meio: " + (detail === undefined ? "sem detalhe" : detail.truncated));
            check(detail !== undefined && detail.width <= longRow.width * 0.55 + 0.5,
                  "o detalhe deixa lugar ao rotulo: " + (detail === undefined ? 0 : detail.width));
            const label = root.textsOf(longRow).find(text => text.text === "elsewhere");
            check(label !== undefined && !label.truncated, "o rotulo curto nao e' cortado");
            root.areaOf(longRow).entered();
            check(TooltipController.shown && TooltipController.text === root.longPath
                  && TooltipController.owner === longRow, "a dica tem o caminho inteiro: " + TooltipController.text);
            root.areaOf(longRow).exited();
            check(!TooltipController.shown, "sair do item esconde a dica");
            // O sinal acima pula o enabled da area: o pairar de verdade so'
            // chega ao desabilitado se a area dele estiver ligada.
            check(root.areaOf(rows[2]).enabled && root.areaOf(rows[2]).hoverEnabled,
                  "o pairar chega ao item desabilitado");
            check(!root.areaOf(rows[4]).enabled, "o separador nao recebe o pairar");
            const before = popup.currentIndex;
            root.areaOf(rows[2]).entered();
            check(TooltipController.shown && TooltipController.text.indexOf("não serve de pasta") >= 0,
                  "o item desabilitado tambem explica: " + TooltipController.text);
            check(popup.currentIndex === before, "pairar no desabilitado nao o seleciona");
            popup.activate(2);
            check(root.chosen === "nenhuma", "clicar no desabilitado nao escolhe");
            popup.activate(5);
            check(root.chosen === "cmake.configuration:", "o automatico escolhe null: " + root.chosen);
            root.areaOf(rows[2]).exited();
            popup.visible = false;

            // O automático com o preset padrão: marcado, com a pasta dele.
            controller.handleCmakeStatus(false, false, "", "", [], "", {},
                                         { preset: "debug", source: "CMakePresets.json",
                                           directory: "build/debug", choices: root.choices });
            check(menu.summary() === "automático · Depuração", "o chip do automatico: " + menu.summary());
            const automatic = menu.items();
            check(automatic.map(item => item.icon || "").join(",") === ",,,,,check"
                  && automatic[5].detail === "build/debug", "o automatico marcado com a pasta: "
                  + JSON.stringify(automatic[5]));
            check(root.configureRequests === 1, "pasta do automatico sem cache: configura uma vez, como antes");

            // Sem presets e sem registro: nada a oferecer, o chip some.
            controller.workspaceRoot = "/tmp/outro";
            check(!menu.offered && menu.summary() === "", "trocar de projeto limpa o menu");
            controller.handleCmakeStatus(true, false, "", "", [], "", {},
                                         { source: "noPresets", directory: ".kinein/build" });
            check(!menu.offered && menu.summary() === "", "sem presets nem registro: sem chip");

            // O registro órfão sem presets (C1-23): a faixa dá a volta ao
            // automático, e o configure automático não roda sem configuração.
            controller.workspaceRoot = "/tmp/orfao";
            const orphan = "configuração escolhida removida dos presets: projeto; escolha outra";
            controller.handleCmakeStatus(false, false, "", "", [], "",
                                         { selectedConfiguration: "projeto" }, { refused: orphan });
            check(root.configureRequests === 1, "sem configuracao ativa: nao configura sozinho");
            check(controller.status === "warning" && controller.message === orphan
                  && controller.actionLabel === "Escolher configuração…"
                  && controller.actionTarget === "cmakeConfigurationChoice",
                  "a faixa diz a recusa e oferece o menu: " + controller.message + " / " + controller.actionLabel);
            check(menu.summary() === "⚠ sem configuração" && menu.tooltip().indexOf(orphan) >= 0,
                  "o chip avisa: " + menu.summary());
            const back = controller.menuItems();
            check(back.length === 1 && back[0].label === "Automático" && back[0].enabled
                  && (back[0].icon || "") === "", "so' a volta ao automatico: " + JSON.stringify(back));
            // Escolher não configura (achado na UI nativa: a escolha de uma
            // pasta sem cache liberava o configure automático que esperava).
            controller.handleConfigurationChosen();
            controller.handleCmakeStatus(false, false, "", "", [], "", {},
                                         { source: "noPresets", directory: ".kinein/build" });
            check(root.configureRequests === 1, "a escolha nao configura: " + root.configureRequests);
            check(controller.actionTarget === "cmakeConfigure" && controller.actionLabel === "Configurar",
                  "a faixa oferece o configure: " + controller.actionLabel);

            // O registro ilegível chega sem nome, só com a recusa.
            controller.workspaceRoot = "/tmp/ilegivel";
            controller.handleCmakeStatus(false, false, "", "", [], "", {},
                                         { refused: "registro da configuração ilegível: .kinein/configuration.json" });
            check(menu.offered && controller.actionTarget === "cmakeConfigurationChoice"
                  && controller.menuItems().length === 1, "o ilegivel tambem tem a volta: " + controller.actionTarget);
            check(root.configureRequests === 1, "o ilegivel nao configura sozinho");
            // Outro projeto não herda a escolha: sem recusa, configura sozinho.
            controller.handleCmakeStatus(false, false, "", "", [], "", {},
                                         { source: "noPresets", directory: ".kinein/build" });
            check(root.configureRequests === 2, "a escolha e' de um projeto so': " + root.configureRequests);
            Qt.exit(root.failures === 0 ? 0 : 1);
        }
    }
}
