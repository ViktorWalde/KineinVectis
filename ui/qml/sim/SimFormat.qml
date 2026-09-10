// COMO UM NUMERO DE SIMULACAO APARECE, num dono so'.
//
// POR QUE ESTE ARQUIVO EXISTE (2026-09-06). A regra de formatacao nasceu no
// `SimRunResultView` e ia ser copiada para o `SimSystemResultView` na mesma
// fatia. Copia de derivacao diverge em silencio — foi assim que a mesma
// severidade ficou azul num painel e vermelha noutro, e e' por isso que existe
// o `verificar-qml-duplicacao.sh` e o `StatusColors`.
//
// A regra, e o motivo dela: um erro de integracao vale 4,9e-5 e um resultado
// vale -0,276. Notacao fixa esconderia o primeiro em zeros e notacao cientifica
// tornaria o segundo ilegivel, entao a faixa decide qual usar. Nove casas na
// forma fixa porque e' onde a deriva do simpletico (2,8e-10) ainda aparece
// como numero e nao como zero.
//
// A fronteira: ele FORMATA, nao arredonda para efeito nem decide o que mostrar.
// Quem decide o que e' erro, deriva ou valor exato e' o core.
pragma Singleton
import QtQuick

QtObject {
    // O numero como a tela de simulacao o escreve. `undefined`/`null` viram um
    // travessao: ausente e zero sao coisas diferentes, e a tela nao inventa.
    //
    // `NaN` e `Infinity` passam e aparecem com esse nome, de proposito: uma
    // integracao que explodiu tem de ser visivel como tal, e nao virar um
    // travessao que se confunde com "o conceito nao tem esse dado".
    function number(x) {
        if (x === undefined || x === null) {
            return "—";
        }
        const a = Math.abs(x);
        if (a !== 0 && (a < 1e-4 || a >= 1e6)) {
            return x.toExponential(6);
        }
        return x.toFixed(9);
    }
}
