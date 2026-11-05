import QtQuick
import KineinVectis

// O formulário real conserva escolhas livres e só emite edição por gesto.
Item {
    id: root
    width: 800
    height: 600
    property int failures: 0
    property int edits: 0
    ProjectOptionsController { id: options }
    QtObject {
        id: creation
        property string createLanguage: "c"
        property var projectOptions: options
    }
    ProjectEnvironmentFields {
        id: fields
        width: 500
        controller: creation
    }
    ConfigActionField {
        id: target
        y: 300
        width: 500
        param: ({ name: "target", label: "Alvo", required: true, suggestions: ["main", "other"] })
        onEdited: value => { target.value = value; root.edits++; }
    }
    Component {
        id: temporaryField
        KvSelect { choices: ["one", "two"] }
    }
    function check(ok, reason) {
        if (!ok) { failures++; console.error("GUARD: " + reason); }
    }
    function findSelect(item) {
        if (item.menuOpen !== undefined) return item;
        for (const child of item.children) {
            const found = findSelect(child);
            if (found) return found;
        }
        return null;
    }
    function run() {
        const compiler = fields.children[1];
        const standard = fields.children[4];
        check(compiler.menuOpen !== undefined && standard.menuOpen !== undefined, "campos selecionáveis na criação");
        check(options.executable === "" && options.standard === "", "nenhum primeiro valor imposto");
        options.resolved([{language: "c", executable: "/tools/GCC ação", version: "GCC fixture"},
            {language: "c", executable: "/tools/Clang", version: "Clang fixture"},
            {language: "cpp", executable: "/tools/g++", version: "g++"}]);
        check(compiler.normalizedChoices.length === 2, "instalações filtradas por linguagem");
        compiler.choose("1");
        check(options.payload().executable === "/tools/Clang", "seleção envia caminho exato");
        check(compiler.displayLabel.indexOf("Clang fixture") >= 0, "versão e caminho identificam instalação");
        options.refresh();
        compiler.open();
        check(!compiler.menuOpen, "busca não oferece lista anterior");
        options.resolved([]);
        check(options.executable === "/tools/Clang" && compiler.customMode, "atualização não substitui instalação");
        options.standard = "iso9899:1990";
        check(standard.customMode && options.payload().standard === "iso9899:1990", "dialeto literal adicional persiste");
        standard.choose("3");
        check(options.standard === "17", "padrão selecionável independente do compilador");
        standard.choose("empty");
        check(options.payload().standard === undefined, "vazio não impõe revisão");
        standard.choose("custom");
        check(standard.customMode && options.standard === "", "personalizado não escolhe revisão");
        const selector = findSelect(target);
        check(selector !== null, "alvos usam seletor");
        selector.choose("1");
        check(target.value === "other" && edits === 1, "ação recebe alvo escolhido sem transcrição");
        selector.choose("custom");
        check(target.value === "other" && edits === 1, "abrir personalizado não altera parâmetro");
        target.value = "target_future";
        check(selector.customMode, "alvo adicional continua permitido");
        target.param = { name: "target", label: "Alvo", required: true, suggestions: ["new"] };
        check(target.value === "target_future" && edits === 1, "reconsulta não substitui parâmetro");
        selector.open();
        check(selector.menuOpen, "menu abre fora da área recortada");
        const menu = root.Window.window.contentItem.children.find(child => child.items === selector.menuItems);
        menu.prepare();
        menu.handleKey({ key: Qt.Key_Escape, accepted: false });
        check(!selector.menuOpen && target.value === "target_future" && edits === 1, "Escape cancela sem editar");
        selector.open();
        menu.prepare();
        menu.handleKey({ key: Qt.Key_Home, accepted: false });
        menu.handleKey({ key: Qt.Key_Down, accepted: false });
        menu.handleKey({ key: Qt.Key_Return, accepted: false });
        check(!selector.menuOpen && target.value === "new" && edits === 2, "teclado confirma somente item escolhido");
        selector.choose("custom");
        selector.open();
        target.visible = false;
        check(!selector.menuOpen, "ocultar formulário fecha menu");
        check(!selector.customRequested, "reabrir formulário não restaura edição personalizada vazia");
        check(target.value === "new", "ocultar formulário conserva valor selecionado");
        creation.createLanguage = "rust";
        check(standard.normalizedChoices.some(choice => choice.value === "2024"), "edições Rust preservadas");
        creation.createLanguage = "cpp";
        check(standard.normalizedChoices.some(choice => choice.value === "26"), "padrões C++ preservados");
        creation.createLanguage = "python";
        check(!standard.visible, "Python não recebe padrão C");
        const temporary = temporaryField.createObject(root);
        temporary.open();
        const temporaryMenu = root.Window.window.contentItem.children.find(child => child.items === temporary.menuItems);
        temporary.destroy();
        Qt.callLater(function() { Qt.callLater(function() {
            check(!temporaryMenu || !temporaryMenu.visible, "destruir formulário não deixa menu ativo");
            console.log("choice fields: " + failures + " falhas");
            Qt.exit(failures === 0 ? 0 : 1);
        }); });
    }
    Component.onCompleted: Qt.callLater(run)
}
