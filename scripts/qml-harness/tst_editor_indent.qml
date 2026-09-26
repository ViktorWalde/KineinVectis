import QtQuick
// Carrega os componentes REAIS do projeto (sem copia).
import "../../ui/qml/editor"

// A INDENTACAO: o fallback local e as tres travas da correcao (E1, 2026-09-26).
//
// §7 da especificacao: "renderizacao nao roda no caminho sincrono do TextEdit".
// O mesmo vale aqui, e com mais forca: a tecla NUNCA espera. O fallback local
// decide na hora; a gramatica so' corrige depois, e so' se as tres travas
// passarem.
//
// A regra mora separada do widget pelo mesmo motivo que a do Markdown: o
// espelho plano do harness nao tem os tipos registrados em C++.
Item {
    id: root

    property int failures: 0

    EditorIndentRules {
        id: rules
    }

    EditorIndentCorrection {
        id: correction
    }

    property int approvals: 0
    property int approvedLevel: -1

    Connections {
        target: correction

        function onApproved(lineStart, appliedIndent, level) {
            root.approvals += 1;
            root.approvedLevel = level;
        }
    }

    function check(ok, bit, message) {
        if (!ok) {
            root.failures += bit;
            console.error(message);
        }
    }

    Component.onCompleted: {
        // --- O FALLBACK LOCAL, que e' o caminho normal ---------------------

        // Abrir bloco indenta; o fechador ja' presente (auto-close) desce.
        const abre = rules.forNewline("fn main() {}", 11);
        check(abre.insert === "    ", 1, "abrir bloco: [" + abre.insert + "]");
        check(abre.closerIndent === "", 2, "sem fechador a' frente nao ha' o que descer");

        const comFechador = rules.forNewline("fn main() {}", 11);
        check(comFechador.insert === "    ", 4, "com `}` a' frente tambem indenta");

        // A indentacao da linha e' repetida quando nada abre.
        const repete = rules.forNewline("    let x = 1;", 14);
        check(repete.insert === "    ", 8, "repetir a indentacao: [" + repete.insert + "]");

        // Comentario de linha CONTINUA comentado: quebrar um `//` e deixar a
        // proxima como codigo muda o significado do arquivo.
        const comentario = rules.forNewline("    // uma nota", 15);
        check(comentario.insert === "    // ", 16, "continuar o comentario: [" + comentario.insert + "]");

        // Dentro de `/* */` a continuacao alinha com o asterisco.
        const bloco = rules.forNewline("    /* uma nota", 15);
        check(bloco.insert === "     * ", 32, "continuar o bloco: [" + bloco.insert + "]");

        // Python: o `:` abre.
        const python = rules.forNewline("def f():", 8);
        check(python.insert === "    ", 64, "o `:` abre: [" + python.insert + "]");

        // Contar e produzir niveis sao inversas.
        check(rules.levelsOf("        ") === 2, 128, "dois niveis");
        check(rules.indentFor(3) === "            ", 256, "tres niveis");
        check(rules.levelsOf("") === 0, 512, "sem indentacao, zero niveis");

        // --- AS TRES TRAVAS -------------------------------------------------

        // Resposta do MESMO documento, MESMA versao: passa.
        correction.remember(10, "    ");
        correction.stamp("/p/a.rs", 7);
        check(correction.accept("/p/a.rs", 7, 2) === true, 1024, "a resposta certa passa");
        check(root.approvals === 1 && root.approvedLevel === 2, 2048, "e chega com o nivel");

        // Documento DIFERENTE: a resposta pode chegar depois de trocar de aba,
        // e indentar o arquivo errado e' pior que nao indentar.
        correction.remember(10, "    ");
        correction.stamp("/p/a.rs", 7);
        check(correction.accept("/p/outro.rs", 7, 2) === false, 4096, "outro documento e' descartado");
        check(root.approvals === 1, 8192, "e nada foi aplicado");

        // VERSAO diferente: o core responde a partir da arvore que TEM, que
        // pode ser anterior ao que o autor ja' digitou.
        correction.remember(10, "    ");
        correction.stamp("/p/a.rs", 7);
        check(correction.accept("/p/a.rs", 6, 2) === false, 16384, "versao velha e' descartada");
        check(root.approvals === 1, 32768, "e nada foi aplicado");

        // "Sem resposta" da gramatica (nivel negativo) nao e' erro: o fallback
        // ja' esta' na tela.
        correction.remember(10, "    ");
        correction.stamp("/p/a.rs", 7);
        check(correction.accept("/p/a.rs", 7, -1) === false, 65536, "sem resposta nao aplica");

        // E uma resposta sem pedido nao inventa correcao.
        check(correction.accept("/p/a.rs", 7, 2) === false, 131072, "sem pedido, nada a aceitar");
        check(root.approvals === 1, 262144, "nenhuma aprovacao a mais");

        Qt.exit(root.failures === 0 ? 0 : 1);
    }
}
