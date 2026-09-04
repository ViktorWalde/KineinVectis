// Cor por ESTADO DE DOMINIO, num dono so.
//
// POR QUE ESTE ARQUIVO EXISTE (2026-09-03). A mesma derivacao estava escrita em
// varios lugares, e a copia ja tinha divergido de verdade:
//
//   ProblemsPanel.severityColor   severidade desconhecida -> infoSoft  (azul)
//   EditorGutter.diagnosticColor  severidade desconhecida -> errorSoft (vermelho)
//
// O MESMO diagnostico aparecia azul no painel e vermelho na sarjeta, e nada
// reclamava. Foi medido assim: 19 derivacoes `campo === "valor"` repetidas em
// mais de um arquivo da UI.
//
// A fronteira que este arquivo respeita: ele NAO decide nada de dominio. Ele
// TRADUZ um estado que o core ja decidiu para um token do `Theme`. Traduzir e'
// trabalho da UI; decidir nao e'.
//
// O `Theme` continua sendo a paleta e nao ganha vocabulario de dominio: ele nao
// sabe o que e um breakpoint, um diagnostico ou um arquivo em conflito.
pragma Singleton
import QtQuick

QtObject {
    // Severidade de diagnostico (LSP/compilador).
    //
    // Desconhecida vira ERRO, e a escolha e' deliberada: esconder um problema
    // e' pior que mostra-lo com peso demais. Era aqui que as duas copias
    // discordavam.
    function severity(value) {
        if (value === "warning") {
            return Theme.warningSoft;
        }
        if (value === "note" || value === "info" || value === "hint") {
            return Theme.infoSoft;
        }
        return Theme.errorSoft;
    }

    // Estado de um arquivo no git.
    function gitKind(kind) {
        if (kind === "conflicted") {
            return Theme.errorSoft;
        }
        if (kind === "untracked" || kind === "added") {
            return Theme.successSoft;
        }
        if (kind === "deleted") {
            return Theme.textDisabled;
        }
        return Theme.infoSoft;
    }

    // Cor de um indice ANSI do terminal. As tres faixas do padrao xterm-256:
    // 0-15 sao a paleta do tema (o autor escolhe), 16-231 o cubo 6x6x6 e
    // 232-255 a rampa de cinza. Uma string passa direto — o core ja pode ter
    // resolvido a cor (ANSI de 24 bits).
    //
    // Estava escrito dentro do TerminalViewport, um renderer visual. "Qual cor
    // e' o ANSI 214" e' regra, nao desenho, e por isso mora aqui junto de
    // severidade e estado de git.
    function ansi(value) {
        if (typeof value === "string") {
            return value;
        }
        const number = value | 0;
        if (number < 16) {
            return Theme.terminalPalette[number];
        }
        if (number < 232) {
            const cube = number - 16;
            const convert = function(part) {
                return part === 0 ? 0 : 55 + part * 40;
            };
            return Qt.rgba(convert(Math.floor(cube / 36)) / 255,
                           convert(Math.floor((cube % 36) / 6)) / 255,
                           convert(cube % 6) / 255, 1);
        }
        const gray = (8 + (number - 232) * 10) / 255;
        return Qt.rgba(gray, gray, gray, 1);
    }

    // Frente e fundo de um trecho do terminal. `inverse` (SGR 7) TROCA os dois,
    // e e' por isso que as duas funcoes calculam o par inteiro em vez de so' o
    // lado que devolvem. O fundo ausente e' transparente, nao a cor do editor:
    // celula sem fundo proprio deixa passar o do painel.
    function terminalForeground(span) {
        const fg = span.fg !== undefined ? ansi(span.fg) : Theme.textPrimary;
        const bg = span.bg !== undefined ? ansi(span.bg) : Theme.backgroundEditor;
        return span.inverse === true ? bg : fg;
    }

    function terminalBackground(span) {
        const fg = span.fg !== undefined ? ansi(span.fg) : Theme.textPrimary;
        const bg = span.bg !== undefined ? ansi(span.bg) : "transparent";
        return span.inverse === true ? fg : bg;
    }
}
