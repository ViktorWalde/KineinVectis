import QtQuick

// D1b (docs/roadmaps/24): busca e substituição DENTRO do arquivo aberto (Ctrl+F /
// Ctrl+H). Lógica pura sobre o buffer do editor — nada de IPC: o texto já
// está na UI (o Ctrl+Shift+F, que varre o DISCO, é que vive no core).
Item {
    id: root

    property var surfaceBridge: null
    property var textController: null

    // D1 (docs/roadmaps/24): estado da barra em property PRÓPRIA. NUNCA usar o
    // `visible` do Item como estado — este controller vive dentro do
    // EditorController (Item invisível) e `Item.visible` lê a visibilidade
    // EFETIVA, que ficaria presa em false.
    property bool barVisible: false
    property bool replaceMode: false

    property string query: ""
    property string replacement: ""
    property bool caseSensitive: false
    property bool wholeWord: false
    property bool useRegex: false

    // Offsets [start, end) de cada ocorrência, na ordem do texto.
    property var matches: []
    property int current: -1
    property bool invalidRegex: false

    readonly property int matchCount: matches.length
    // "3 de 17" — 1-based para o humano; 0 quando não há match.
    readonly property int currentDisplay: current >= 0 ? current + 1 : 0

    // NÃO se chama "matchesChanged": esse nome já é o sinal automático da
    // property `matches`. Um só sinal para "matches e/ou atual mudaram",
    // emitido uma vez por transição (evita re-realçar o texto em dobro).
    signal searchStateChanged()

    visible: false

    function ready() {
        return surfaceBridge !== null && surfaceBridge.ready()
                && textController !== null;
    }

    // Regex escapada para busca LITERAL (o modo padrão).
    function escapeRegex(text) {
        return text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    }

    // Monta o RegExp conforme os toggles. Devolve null se a regex do
    // usuário for inválida (campo entra em estado de erro).
    function buildPattern() {
        if (query === "") {
            return null;
        }
        let source = useRegex ? query : escapeRegex(query);
        if (wholeWord) {
            source = "\\b(?:" + source + ")\\b";
        }
        const flags = caseSensitive ? "g" : "gi";
        try {
            return new RegExp(source, flags);
        } catch (error) {
            return null;
        }
    }

    // Varre o texto inteiro e guarda os offsets. O AVANÇO FORÇADO no match
    // de largura zero (regex tipo `a*` ou `\b`) evita o laço infinito.
    function recompute(preferredOffset) {
        const previous = matches.length;
        matches = [];
        current = -1;
        invalidRegex = false;
        if (!ready() || query === "") {
            if (previous > 0) {
                searchStateChanged();
            }
            return;
        }
        const pattern = buildPattern();
        if (pattern === null) {
            invalidRegex = true;
            searchStateChanged();
            return;
        }
        const text = surfaceBridge.text();
        const found = [];
        let match = pattern.exec(text);
        while (match !== null) {
            found.push({ start: match.index, end: match.index + match[0].length });
            if (match[0].length === 0) {
                pattern.lastIndex++;
            }
            if (pattern.lastIndex > text.length) {
                break;
            }
            match = pattern.exec(text);
        }
        matches = found;
        if (found.length > 0) {
            current = indexAtOrAfter(preferredOffset);
        }
        searchStateChanged();
    }

    // Primeiro match que começa em/depois do offset; circula pro 0.
    function indexAtOrAfter(offset) {
        for (let i = 0; i < matches.length; i++) {
            if (matches[i].start >= offset) {
                return i;
            }
        }
        return 0;
    }

    function cursorOffset() {
        if (!ready()) {
            return 0;
        }
        const surface = surfaceBridge.editorSurface;
        return Math.min(surface.selectionStart, surface.selectionEnd);
    }

    // Abre a barra. Pré-preenche com a SELEÇÃO; sem seleção, com a palavra
    // sob o cursor (comportamento do VS Code/JetBrains).
    function open(withReplace) {
        if (!ready()) {
            return;
        }
        const surface = surfaceBridge.editorSurface;
        const start = Math.min(surface.selectionStart, surface.selectionEnd);
        const end = Math.max(surface.selectionStart, surface.selectionEnd);
        const seed = end > start
                ? surfaceBridge.text().substring(start, end)
                : textController.currentWord();
        // Seleção multi-linha não vira semente de busca (seria ruído).
        if (seed !== "" && seed.indexOf("\n") < 0) {
            query = seed;
        }
        replaceMode = withReplace === true;
        barVisible = true;
        recompute(start);
    }

    function close() {
        barVisible = false;
        replaceMode = false;
        matches = [];
        current = -1;
        invalidRegex = false;
        searchStateChanged();
        if (ready()) {
            surfaceBridge.focusEditor();
        }
    }

    function setQuery(text) {
        if (query === text) {
            return;
        }
        query = text;
        recompute(cursorOffset());
        selectCurrent();
    }

    function setReplacement(text) {
        replacement = text;
    }

    function toggleCaseSensitive() {
        caseSensitive = !caseSensitive;
        recompute(cursorOffset());
        selectCurrent();
    }

    function toggleWholeWord() {
        wholeWord = !wholeWord;
        recompute(cursorOffset());
        selectCurrent();
    }

    function toggleRegex() {
        useRegex = !useRegex;
        recompute(cursorOffset());
        selectCurrent();
    }

    // Navegação CIRCULAR: passar do último volta ao primeiro.
    function step(delta) {
        if (matches.length === 0) {
            return;
        }
        const count = matches.length;
        current = ((current + delta) % count + count) % count;
        selectCurrent();
        // O "atual" mudou → o realce forte tem que pular junto.
        searchStateChanged();
    }

    function findNext() {
        step(1);
    }

    function findPrevious() {
        step(-1);
    }

    // Seleciona o match atual no editor (sem roubar o foco da barra: o
    // TextEdit rola até o cursor, e o usuário continua digitando na busca).
    function selectCurrent() {
        if (!ready() || current < 0 || current >= matches.length) {
            return;
        }
        const match = matches[current];
        surfaceBridge.editorSurface.select(match.start, match.end);
    }

    // Substitui o match atual e avança. Em regex, `$1` e afins são
    // resolvidos pelo replace do JS sobre o TRECHO casado.
    function replaceCurrent() {
        if (!ready() || current < 0 || current >= matches.length) {
            return;
        }
        const surface = surfaceBridge.editorSurface;
        const match = matches[current];
        const matched = surfaceBridge.text().substring(match.start, match.end);
        const pattern = buildPattern();
        if (pattern === null) {
            return;
        }
        const replaced = useRegex
                ? matched.replace(new RegExp(pattern.source,
                                             caseSensitive ? "" : "i"), replacement)
                : replacement;
        surface.remove(match.start, match.end);
        surface.insert(match.start, replaced);
        // O texto mudou: recomputa a partir do fim do que foi inserido, o
        // que já deixa o "próximo" como atual (avanço natural).
        recompute(match.start + replaced.length);
        selectCurrent();
    }

    // Substitui TUDO numa passada. DE TRÁS PRA FRENTE: mexer no fim não
    // invalida os offsets do começo (o contrário invalidaria todos).
    function replaceAll() {
        if (!ready() || matches.length === 0) {
            return;
        }
        const surface = surfaceBridge.editorSurface;
        const pattern = buildPattern();
        if (pattern === null) {
            return;
        }
        const text = surfaceBridge.text();
        const single = useRegex
                ? new RegExp(pattern.source, caseSensitive ? "" : "i") : null;
        for (let i = matches.length - 1; i >= 0; i--) {
            const match = matches[i];
            const matched = text.substring(match.start, match.end);
            const replaced = single !== null
                    ? matched.replace(single, replacement) : replacement;
            surface.remove(match.start, match.end);
            surface.insert(match.start, replaced);
        }
        recompute(cursorOffset());
    }
}
