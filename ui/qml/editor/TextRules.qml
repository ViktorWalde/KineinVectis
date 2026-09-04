import QtQuick

// Regras PURAS de texto de codigo, num dono so.
//
// POR QUE ESTE ARQUIVO EXISTE (2026-09-03). `isWordChar` estava escrito DUAS
// vezes, com implementacoes DIFERENTES para a mesma regra:
//
//   EditorTextController   comparacao de faixas: (ch >= "a" && ch <= "z") ...
//   EditorTextSurface      expressao regular: /[A-Za-z0-9_]/.test(character)
//
// As duas concordam — por sorte, nao por construcao. Bastava uma delas ganhar
// "$" (comum em identificador gerado) para o autocomplete e o auto-close
// discordarem sobre onde comeca uma palavra, sem nada reclamar. Foi
// exatamente assim que a divergencia de cor de severidade nasceu, e ela virou
// o 16o gate (docs/roadmaps/38 §3).
//
// As tabelas de par vieram junto porque sao a MESMA classe de coisa: regra de
// dominio sobre texto de codigo. Elas moravam dentro de um componente VISUAL
// (EditorTextSurface), e regra de dominio nao mora na UI.
//
// POR QUE NAO E' SINGLETON. Um singleton QML so' existe atraves do modulo
// `KineinVectis`, e o harness de logica (scripts/qml-harness) carrega os
// controllers por CAMINHO RELATIVO, sem modulo montado — importar o modulo
// tornaria estas regras inexercitaveis fora do app. Sendo sem estado, varias
// instancias custam nada e o que importa e' haver uma unica DEFINICAO.
//
// LIMITE DELIBERADO: `isWordChar` e' ASCII. Identificador com acento ou
// caractere nao-ASCII NAO conta como palavra. C e C++ so' admitem isso via
// escape universal (\uXXXX) e Rust exige XID_Start/XID_Continue, que pedem
// tabela Unicode inteira. Ate' aparecer caso real medido, a regra fica ASCII.
QtObject {
    // Abridor -> fechador que o auto-close insere.
    readonly property var pairOpeners: ({ "(": ")", "[": "]", "{": "}",
                                          "\"": "\"", "'": "'" })

    // Fechadores que aceitam type-over (digitar por cima em vez de duplicar).
    readonly property var pairClosers: ({ ")": true, "]": true, "}": true,
                                          "\"": true, "'": true })

    function isWordChar(character) {
        return character !== "" && /[A-Za-z0-9_]/.test(character);
    }
}
