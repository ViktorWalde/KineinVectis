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
}
