import QtQuick
import "../../ui/qml/workspace"

// "Criar Projeto" (0.3.6, roadmap 53 §13.0 item 6): a LINGUAGEM primeiro, o
// ECOSSISTEMA depois, e o Python no mesmo nivel de C/C++ e Rust. O que se
// prova: todo template que o core aceita no `workspace.createProject` tem
// linguagem; escolher a linguagem seleciona o ecossistema dela; criar sem
// linguagem e' recusado com motivo, nunca vira um C++ por padrao; e o
// template vindo de fora (a paleta, quem abrir ja' com um) traz a linguagem.
Item {
    id: root

    property var created: []

    FolderPickerController {
        id: picker

        onCreateProjectRequested: function(parent, name, templateId) {
            root.created.push(parent + "|" + name + "|" + templateId);
        }
    }

    function check(condition, message) {
        if (!condition) {
            console.error("FALHOU: " + message);
            return 1;
        }
        return 0;
    }

    Component.onCompleted: {
        const catalog = picker.templateCatalog;
        let failures = 0;

        // Os quatro templates do contrato (`WorkspaceProjectTemplate`), cada
        // um com a sua linguagem.
        const expected = { cppCmake: "cpp", rustCargo: "rust", python: "python", empty: "empty" };
        for (const templateId in expected) {
            failures += check(catalog.languageOf(templateId) === expected[templateId],
                              "linguagem de " + templateId);
        }
        failures += check(catalog.languages.length === 4, "quatro linguagens");
        failures += check(catalog.language("python").label === "Python", "Python na lista");
        failures += check(catalog.languageOf("platformIo") === "", "template desconhecido sem dono");

        // Nada escolhido ao abrir: criar e' recusado, com motivo.
        picker.chooseTemplate("");
        picker.beginCreateProject();
        picker.currentPath = "/home/x";
        picker.createName = "demo";
        picker.submitCreate();
        failures += check(root.created.length === 0, "criou sem linguagem");
        failures += check(picker.errorText !== "", "recusa sem motivo");

        // A linguagem escolhe o ecossistema dela; criar usa esse template.
        picker.chooseLanguage("python");
        failures += check(picker.createTemplate === "python", "python -> template python");
        failures += check(picker.errorText === "", "a escolha limpa o erro");
        picker.submitCreate();
        failures += check(root.created.join(",") === "/home/x|demo|python", "criou " + root.created);

        // Template vindo de fora traz a linguagem; desconhecido nao escolhe nada.
        picker.chooseTemplate("rustCargo");
        failures += check(picker.createLanguage === "rust", "rustCargo -> rust");
        picker.chooseTemplate("platformIo");
        failures += check(picker.createLanguage === "" && picker.createTemplate === "",
                          "template desconhecido escolheu algo");

        // A previa vem do catalogo: o modulo Python usa _ no lugar de -.
        const preview = catalog.preview("python", "meu-app");
        failures += check(preview.indexOf("meu_app/__init__.py") >= 0, "previa python: " + preview);
        failures += check(catalog.preview("rustCargo", "x").indexOf("cargo new --bin --vcs none x") >= 0,
                          "previa rust mostra o comando");
        failures += check(catalog.preview("", "x").indexOf("Escolha") === 0, "previa sem escolha");

        if (failures !== 0) console.error("FALHAS=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
