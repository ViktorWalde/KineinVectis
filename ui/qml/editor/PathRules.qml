import QtQuick

// Regras PURAS de caminho de arquivo, num dono so.
//
// POR QUE ESTE ARQUIVO EXISTE (2026-09-03). Medido: `relativeToRoot` estava
// escrito SEIS vezes na UI (editor, jobs, shell, search x2, project) e
// `baseName` TRES. Pior que a repeticao, a derivacao "este caminho esta
// DEBAIXO daquele" — `p === a || p.indexOf(a + "/") === 0` — aparecia tres
// vezes so' dentro do EditorDocumentController, uma delas para renomear abas e
// outra para renomear a lista de recentes, lado a lado.
//
// O "+ /" nao e' detalhe: sem ele, renomear "src/app" tambem pegaria
// "src/application", e o bug so' apareceria com dois diretorios de nome
// parecido no mesmo projeto.
//
// POR QUE MORA NA PASTA `editor`. O modulo QML da IDE e' PLANO: cada arquivo
// e' registrado com QT_RESOURCE_ALIAS reduzido ao nome, entao no app todos sao
// irmaos e este tipo e' visivel de qualquer pasta. Ja o harness de logica
// (scripts/qml-harness) carrega os controllers por CAMINHO RELATIVO REAL, onde
// so' resolve o que esta na MESMA pasta. Enquanto os dois modelos coexistirem,
// um tipo compartilhado precisa morar junto de quem o harness exercita.
// Consolidar as copias de jobs/shell/search/project depende de resolver essa
// diferenca primeiro — esta registrado em DocsPublic/roadmaps/39 §4.
QtObject {
    function baseName(path) {
        return path.substring(path.lastIndexOf("/") + 1);
    }

    function relativeTo(root, path) {
        if (root !== "" && path.indexOf(root + "/") === 0) {
            return path.substring(root.length + 1);
        }
        return path;
    }

    // O proprio caminho conta como estando debaixo dele mesmo.
    function isUnder(path, ancestor) {
        return path === ancestor || path.indexOf(ancestor + "/") === 0;
    }

    // Reescreve `path` quando `from` foi renomeado para `to`. Devolve o
    // caminho INALTERADO se a renomeacao nao o afeta — o chamador nao precisa
    // testar `isUnder` antes.
    function renamed(path, from, to) {
        if (path === from) {
            return to;
        }
        if (path.indexOf(from + "/") === 0) {
            return to + path.substring(from.length);
        }
        return path;
    }
}
