.pragma library

// LISTA QUE VEM DO C++ — dono unico (2026-10-03). O que atravessa o C++
// (QVariantMap/QVariantList: settings, resultado do banco) chega ao QML como
// uma lista que NAO e' um Array do JS: `Array.isArray` da' falso, e so' ha'
// `length` e indices. O defeito apareceu duas vezes — o trilho (40.7 §7.160)
// e o Resultado do Banco todo em "null" —, entao a pergunta mora aqui.

function isList(value) {
    return value !== undefined && value !== null && typeof value !== "string"
           && (Array.isArray(value) || typeof value.length === "number");
}

// Copia para um Array do JS (vazio quando nao e' lista).
function listOf(value) {
    const out = [];
    if (isList(value)) {
        for (let i = 0; i < value.length; i++) {
            out.push(value[i]);
        }
    }
    return out;
}
