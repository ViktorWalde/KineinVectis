import QtQuick

// E1 (docs/diario/18, trilha E): auto-close de pares, type-over do fechador,
// surround da selecao e backspace apagando o par vazio.
//
// Vive fora do EditorTextSurface porque decidir o que uma TECLA faz com um PAR
// nao e' responsabilidade de quem desenha o texto. A superficie so pergunta "a
// tecla ja foi tratada?" e respeita a resposta; as regras de par ficam aqui,
// onde da' para le-las inteiras.
QtObject {
    id: root

    // A superficie de texto sobre a qual as edicoes acontecem.
    required property TextEdit target
    property bool enabled: true

    // E6 (2026-07-17): rastreador dos fechadores que NOS inserimos
    // (AutoCloseRegions, C++). O type-over so pula fechador nosso — pular
    // qualquer fechador engolia o `)` digitado a mao em `foo(bar)`
    // (divergencia medida contra o Code OSS, docs/diario/18). E' `var` e nao o
    // tipo C++ de proposito: o tst_autoclose roda em qml-qt6 puro e injeta um
    // fake, o padrao do tst_completion. null = sem type-over NENHUM: duplicar
    // um fechador e' visivel e corrigivel; engolir tecla e' silencioso.
    property var regions: null

    // E3: a insercao com dedent vive no EditorTextController, nao aqui.
    signal closerBraceRequested()

    readonly property var pairOpeners: ({ "(": ")", "[": "]", "{": "}",
                                          "\"": "\"", "'": "'" })
    readonly property var pairClosers: ({ ")": true, "]": true, "}": true,
                                          "\"": true, "'": true })

    function isWordChar(character) {
        return character !== "" && /[A-Za-z0-9_]/.test(character);
    }

    // true = tecla consumida (event.accepted pelo chamador).
    function handleTypingKey(event) {
        if (event.text === "" || !root.enabled) {
            return false;
        }
        // Ctrl puro é atalho; Ctrl+Alt (AltGr em layouts europeus) produz
        // caractere legítimo e passa.
        if ((event.modifiers & Qt.ControlModifier)
                && !(event.modifiers & Qt.AltModifier)) {
            return false;
        }
        const character = event.text;
        const closer = root.pairOpeners[character];
        const position = root.target.cursorPosition;
        const content = root.target.text;
        const hasSelection =
            root.target.selectionStart !== root.target.selectionEnd;

        // CR1: "<" logo após `#include ` fecha em "<>" (contexto seguro;
        // "<" genérico é comparação/template/shift e NÃO auto-fecha).
        if (character === "<" && !hasSelection && position > 0) {
            const lineStart = content.lastIndexOf("\n", position - 1) + 1;
            const beforeCursor = content.substring(lineStart, position);
            if (/^\s*#\s*include\s+$/.test(beforeCursor)) {
                root.target.insert(position, "<>");
                root.target.cursorPosition = position + 1;
                if (root.regions !== null) {
                    root.regions.notePairInserted(position + 1);
                }
                return true;
            }
        }

        if (hasSelection && closer !== undefined) {
            // Abridor com seleção ativa ENVOLVE em vez de substituir.
            const start = root.target.selectionStart;
            const end = root.target.selectionEnd;
            const selected = content.substring(start, end);
            root.target.remove(start, end);
            root.target.insert(start, character + selected + closer);
            root.target.select(start + 1, end + 1);
            if (root.regions !== null) {
                root.regions.notePairInserted(end + 1);
            }
            return true;
        }
        if (!hasSelection && content.charAt(position) === character) {
            // Barra invertida ESCAPA a aspa: em `"a\` + cursor + `"`, quem digita
            // `"` quer INSERIR uma aspa escapada, nao pular a que esta ali —
            // mesmo quando a aspa ali foi auto-inserida (medido 2026-07-17;
            // invariante do Code OSS, que nunca faz type-over de aspa precedida
            // de barra).
            const escapada = (character === "\"" || character === "'")
                && position > 0 && content.charAt(position - 1) === "\\";
            // type-over pela ORIGEM: pula somente o fechador que NOS inserimos
            // (e' o `regions` quem sabe). Fechador escrito a mao nao e' nosso:
            // cai adiante e, se nenhum ramo consumir, o TextEdit insere o
            // caractere normalmente. Sem gate por pairClosers: assim o `>` do
            // `#include <>` tambem faz type-over do fechador auto-inserido.
            if (!escapada && root.regions !== null
                    && root.regions.isAutoClosedAt(position)) {
                root.regions.consumeAt(position);
                root.target.cursorPosition = position + 1;
                return true;
            }
        }
        if (closer !== undefined) {
            const previous = position > 0 ? content.charAt(position - 1) : "";
            const next = content.charAt(position);
            const quote = character === "\"" || character === "'";
            // Aspas coladas em palavra não duplicam (don't → don''t);
            // colchetes/parênteses antes de palavra ou aspas também não.
            // Aspa antes de OUTRA aspa tampouco (era o type-over cego que
            // mascarava este caso; sem ele, a regra fica explicita — como o
            // Code OSS, que nao auto-fecha aspa antes de aspa).
            if (quote && (root.isWordChar(previous) || root.isWordChar(next)
                          || next === "\"" || next === "'")) {
                return false;
            }
            if (!quote && (root.isWordChar(next) || next === "\"" || next === "'")) {
                return false;
            }
            root.target.insert(position, character + closer);
            root.target.cursorPosition = position + 1;
            if (root.regions !== null) {
                root.regions.notePairInserted(position + 1);
            }
            return true;
        }
        if (character === "}" && !hasSelection) {
            // E3: a inserção (com dedent quando a linha é só
            // whitespace) vive no EditorTextController; o type-over
            // acima tem precedência e não re-indenta.
            root.closerBraceRequested();
            return true;
        }
        return false;
    }

    function handlePairBackspace() {
        if (!root.enabled
                || root.target.selectionStart !== root.target.selectionEnd) {
            return false;
        }
        const position = root.target.cursorPosition;
        if (position <= 0) {
            return false;
        }
        const content = root.target.text;
        const previous = content.charAt(position - 1);
        const closer = root.pairOpeners[previous];
        if (closer !== undefined && content.charAt(position) === closer) {
            root.target.remove(position - 1, position + 1);
            return true;
        }
        return false;
    }
}
