import QtQuick

// Presentation of index.context only: no compiler, path or kit resolution.
QtObject {
    id: root

    property var context: ({})
    readonly property var rows: buildRows()

    function originLabel(origin) {
        switch (origin) {
        case "cdb": return qsTr("da compile_commands.json");
        case "cmakeFileApi": return qsTr("do file-api do CMake");
        case "cmakeCache": return qsTr("do cache do CMake");
        case "kit": return qsTr("do kit");
        }
        return "";
    }

    function buildRows() {
        const rows = [];
        const context = root.context;
        if (!context.path) return rows;
        const add = (label, value, origin) => rows.push({
            label: label, value: value || qsTr("não informado"), origin: origin || ""
        });
        const effective = context.effective || {};
        const sourced = (label, field) => add(label, field ? field.value : "",
                                             field ? root.originLabel(field.origin) : "");
        add(qsTr("Fonte do contexto"), context.source, "");
        if (context.language === "c" || context.language === "cpp" || context.language === "rust") {
            sourced(qsTr("Compilador"), effective.compiler);
            sourced(qsTr("Padrão"), effective.standard);
            sourced(qsTr("Target"), effective.target);
            sourced(qsTr("Sysroot"), effective.sysroot);
            if (effective.kit) add(qsTr("Kit"), effective.kit.preset || qsTr("padrão do projeto"), qsTr("do kit"));
        }
        if (context.targets && context.targets.length) add(qsTr("Targets CMake"), context.targets.join("\n"), qsTr("do file-api do CMake"));
        if (context.unit) {
            const unit = context.unit;
            const origin = root.originLabel(effective.flagsOrigin);
            add(qsTr("Diretório de trabalho"), unit.directory, origin);
            add(qsTr("Saída"), unit.output, origin);
            add(qsTr("Includes (%1)").arg(unit.includes.length), unit.includes.join("\n") || qsTr("nenhum declarado"), origin);
            add(qsTr("Defines (%1)").arg(unit.defines.length), unit.defines.join("\n") || qsTr("nenhum declarado"), origin);
            add(qsTr("Argumentos (%1)").arg(unit.arguments.length), unit.arguments.join("\n") || qsTr("nenhum declarado"), origin);
        }
        if (context.crate) {
            const target = context.crate;
            add(qsTr("Pacote Rust"), target.package, "cargo metadata");
            add(qsTr("Alvo Cargo"), target.target + " (" + target.kind + ")", "cargo metadata");
            add(qsTr("Edição Rust"), target.edition, "cargo metadata");
            add(qsTr("Manifesto"), target.manifest, "cargo metadata");
            add(qsTr("Features"), (target.features || []).join("\n") || qsTr("nenhuma declarada"), "cargo metadata");
        }
        if (context.python) {
            add(qsTr("Interpretador Python"), context.python.interpreter, context.python.origin);
            add(qsTr("Versão Python"), context.python.version, context.python.origin);
        }
        return rows;
    }
}
