import QtQuick

// O MENU DE CONFIGURAÇÃO DO CMAKE (C1-23 do C-04, consolidação C §17): as
// configurações dos presets, a ativa marcada, e "Automático". O chip do
// cabeçalho e a faixa do Project Health abrem o mesmo menu. Escolher só troca
// a configuração ativa (prefixo "cmake.configuration:", vazio = automático);
// não configura. Saiu do ProjectHealthController, que decide o aviso: este
// objeto diz o que há para escolher.
QtObject {
    id: root

    // Do `cmake.status` (0.210.0), pelo argumento `configuration` da ponte.
    property string preset: ""
    property string source: ""
    property string refused: ""
    property string directory: ""
    property var choices: []
    // O registro (`selectedConfiguration`): sem presets, é ele que ainda
    // precisa da volta ao automático.
    property string recorded: ""

    // A configuração ativa veio do registro; senão, das regras do automático.
    readonly property bool chosen: source === "configuration"
    // Com presets, com registro ou sem configuração ativa. Um registro
    // ilegível chega sem nome, só com a recusa, e o "Automático" o apaga.
    readonly property bool offered: choices.length > 0 || recorded !== "" || refused !== ""

    function reset() {
        preset = "";
        source = "";
        refused = "";
        directory = "";
        choices = [];
        recorded = "";
    }

    function handle(configuration, recordedPreset) {
        const value = configuration === undefined || configuration === null ? ({}) : configuration;
        preset = value.preset || "";
        source = value.source || "";
        refused = value.refused || "";
        directory = value.directory || "";
        choices = value.choices || [];
        recorded = recordedPreset || "";
    }

    function nameOf(presetName) {
        const choice = choices.find(entry => entry.preset === presetName);
        return choice !== undefined && choice.displayName ? choice.displayName : presetName;
    }

    // O texto do chip: a configuração ativa, ou a falta dela.
    function summary() {
        if (!offered) return "";
        if (refused !== "") return qsTr("⚠ sem configuração");
        if (chosen) return nameOf(preset);
        return preset !== "" ? qsTr("automático · %1").arg(nameOf(preset)) : qsTr("automático");
    }

    function tooltip() {
        if (refused !== "") return qsTr("Configuração do CMake: %1").arg(refused);
        return qsTr("Configuração do CMake em %1 — clique para trocar").arg(directory);
    }

    // A pasta, ou por que ela não é conhecida; sem o CMake, também a
    // condition que só ele decide (C1-26), na linha seguinte.
    function choiceTooltip(choice) {
        const folder = !choice.buildDir ? (choice.buildDirRefused || "")
                     : choice.configured ? choice.buildDir
                                         : qsTr("%1 — ainda sem configure").arg(choice.buildDir);
        return [folder, choice.conditionNote || ""].filter(line => line !== "").join("\n");
    }

    // Um item por configuração; a pasta vai à direita, cortada no meio, e o
    // caminho inteiro fica na dica. Pasta desconhecida não se escolhe.
    function items() {
        const entries = choices.map(choice => ({
            label: choice.displayName || choice.preset,
            detail: choice.buildDir || qsTr("pasta desconhecida"),
            tooltip: root.choiceTooltip(choice),
            action: "cmake.configuration:" + choice.preset,
            icon: root.chosen && choice.preset === root.preset ? "check" : "",
            enabled: (choice.buildDir || "") !== "",
            shortcut: ""
        }));
        if (entries.length > 0) {
            entries.push({ separator: true, label: "", action: "", enabled: false });
        }
        const automatic = !chosen && refused === "";
        entries.push({
            label: qsTr("Automático"),
            detail: automatic ? directory : "",
            tooltip: qsTr("A IDE escolhe: a pasta .kinein/build que ela já configurou, o preset padrão ou o configure sem preset"),
            action: "cmake.configuration:",
            icon: automatic ? "check" : "",
            enabled: true,
            shortcut: ""
        });
        return entries;
    }
}
