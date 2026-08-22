import QtQuick

// Estado do painel "Contexto de compilação" (L3 fatia 1).
//
// Guarda o contexto do arquivo ativo e traduz o `origin` do protocolo em algo
// que uma pessoa entende. A tradução é o ponto da fatia: `exact`, `borrowed` e
// `none` não são estados equivalentes de um mesmo dado — são três graus de
// CONFIANÇA, e apresentá-los igual seria a IDE mentindo sobre o que sabe.
//
// Zero regra de negócio: quem resolve o contexto é o core, pelo
// `project.fileContext`. Aqui só mora apresentação.
Item {
    id: root

    property string path: ""
    property string origin: ""
    property string borrowedFrom: ""
    property string compiler: ""
    property string standard: ""
    property var defines: []
    property var includes: []
    property var flags: []
    property string database: ""
    property string command: ""
    property string note: ""

    readonly property bool hasContext: origin !== ""

    // O rótulo diz o GRAU DE CONFIANÇA, não o nome técnico do enum.
    readonly property string originLabel: {
        if (origin === "exact") {
            return qsTr("comando próprio deste arquivo");
        }
        if (origin === "borrowed") {
            return borrowedFrom !== ""
                 ? qsTr("emprestado de %1").arg(borrowedFrom)
                 : qsTr("emprestado de outra unidade");
        }
        if (origin === "none") {
            return qsTr("sem contexto de compilação");
        }
        return "";
    }

    // Cor por confiança: verde para exato, âmbar para emprestado (é um palpite
    // bom, mas é palpite), apagado para ausente.
    function originColor(theme) {
        if (origin === "exact") {
            return theme.successSoft;
        }
        if (origin === "borrowed") {
            return theme.warningSoft;
        }
        return theme.textMuted;
    }

    function clear() {
        path = "";
        origin = "";
        borrowedFrom = "";
        compiler = "";
        standard = "";
        defines = [];
        includes = [];
        flags = [];
        database = "";
        command = "";
        note = "";
    }

    function handleResolved(context) {
        if (!context) {
            clear();
            return;
        }
        path = context.path !== undefined ? context.path : "";
        origin = context.origin !== undefined ? context.origin : "";
        borrowedFrom = context.borrowedFrom !== undefined ? context.borrowedFrom : "";
        compiler = context.compiler !== undefined ? context.compiler : "";
        standard = context.standard !== undefined ? context.standard : "";
        defines = context.defines !== undefined ? context.defines : [];
        includes = context.includes !== undefined ? context.includes : [];
        flags = context.flags !== undefined ? context.flags : [];
        database = context.database !== undefined ? context.database : "";
        command = context.command !== undefined ? context.command : "";
        note = context.note !== undefined ? context.note : "";
    }
}
