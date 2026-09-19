import QtQuick

// As regras PURAS da HUD do Git (2026-09-18, pedido do autor: "uma HUD
// unica para o Git, como as IDEs JetBrains"): as raias do grafo do
// historico a partir dos pais, os arquivos de um patch, a cor de uma linha
// de diff, a pasta de um caminho. Sem tela, para o harness medir.
QtObject {
    id: rules

    // A pasta de primeiro nivel de um caminho relativo ("(raiz)" sem pasta).
    function folderOf(path) {
        const i = path.lastIndexOf("/");
        return i < 0 ? qsTr("(raiz)") : path.substring(0, i);
    }

    // O estado de uma pasta na lista de mudancas (um ListModel com `folder`,
    // `absPath`, `staged`): os caminhos dela, quantos estao staged, e se
    // todos estao. O checkbox da secao le daqui; o clique manda os caminhos.
    function folderState(model, folder) {
        const paths = [];
        let staged = 0;
        for (let i = 0; i < model.count; i++) {
            const row = model.get(i);
            if (row.folder !== folder) continue;
            paths.push(row.absPath);
            if (row.staged) staged++;
        }
        return { paths: paths, staged: staged, all: paths.length > 0 && staged === paths.length };
    }

    // As raias do grafo: a atribuicao classica em linha reta. Cada linha e'
    // { lane, merge, laneCount, edges }: `lane` e' a coluna do commit; um
    // commit com dois pais e' `merge`; `laneCount` e' quantas colunas estao
    // vivas ali; `edges` sao as ligacoes que SAEM desta linha para a de
    // baixo — [{ from, to }] em colunas: as raias que passam direto
    // (from === to) e as que este commit liga aos pais (curvas quando
    // mudam de coluna). Entradas: [{ sha, parents }], do mais novo ao mais velho.
    function lanes(entries) {
        const active = [];   // sha esperado em cada coluna viva
        const out = [];
        for (let i = 0; i < entries.length; i++) {
            const e = entries[i];
            let lane = active.indexOf(e.sha);
            if (lane < 0) {
                lane = active.length;
                active.push(e.sha);
            }
            const parents = e.parents === undefined ? [] : e.parents;
            const before = active.slice();
            // Este commit sai; o primeiro pai herda a coluna; os outros pais
            // ganham colunas novas (ou ja' tem a sua).
            if (parents.length === 0) {
                active.splice(lane, 1);
            } else {
                active[lane] = parents[0];
                for (let p = 1; p < parents.length; p++) {
                    if (active.indexOf(parents[p]) < 0) active.push(parents[p]);
                }
            }
            // Duas colunas esperando o MESMO sha viram uma.
            for (let a = 0; a < active.length; a++) {
                for (let b = active.length - 1; b > a; b--) {
                    if (active[b] === active[a]) active.splice(b, 1);
                }
            }
            // As arestas: cada coluna viva ANTES vai para onde o seu sha esta'
            // DEPOIS (o commit desta linha vai para onde cada pai ficou).
            const edges = [];
            for (let c = 0; c < before.length; c++) {
                if (c === lane) {
                    for (let p = 0; p < parents.length; p++) {
                        const to = active.indexOf(parents[p]);
                        if (to >= 0) edges.push({ from: lane, to: to });
                    }
                } else {
                    const to = active.indexOf(before[c]);
                    if (to >= 0) edges.push({ from: c, to: to });
                }
            }
            out.push({ lane: lane, merge: parents.length > 1,
                       laneCount: Math.max(1, before.length, active.length, lane + 1), edges: edges });
        }
        return out;
    }

    // O filtro do historico: texto vazio aceita tudo; senao, resumo, autor
    // ou sha (prefixo) contem o texto, sem diferenciar caixa.
    function matchesFilter(entry, text) {
        const q = (text || "").trim().toLowerCase();
        if (q === "") return true;
        return (entry.summary || "").toLowerCase().indexOf(q) >= 0
            || (entry.author || "").toLowerCase().indexOf(q) >= 0
            || (entry.sha || "").toLowerCase().indexOf(q) === 0;
    }

    // Os arquivos de um patch unificado, na ordem: [{ path, added, removed }].
    function patchFiles(text) {
        const files = [];
        let current = null;
        const lines = text.split("\n");
        for (let i = 0; i < lines.length; i++) {
            const l = lines[i];
            if (l.startsWith("diff --git ")) {
                const m = /^diff --git a\/(.*) b\/(.*)$/.exec(l);
                current = { path: m ? m[2] : l.substring(11), added: 0, removed: 0 };
                files.push(current);
            } else if (current !== null) {
                if (l.startsWith("+") && !l.startsWith("+++")) current.added++;
                else if (l.startsWith("-") && !l.startsWith("---")) current.removed++;
            }
        }
        return files;
    }

    // A classe de uma linha de diff: "meta" (+++/---/diff/index), "add",
    // "del", "hunk", "ctx".
    function lineKind(line) {
        if (line.startsWith("+++") || line.startsWith("---") || line.startsWith("diff ")
                || line.startsWith("index ")) return "meta";
        if (line.startsWith("+")) return "add";
        if (line.startsWith("-")) return "del";
        if (line.startsWith("@@")) return "hunk";
        return "ctx";
    }

    // "HEAD -> main" vira { name: "main", head: true }; "tag: v1" vira
    // { name: "v1", tag: true }; "origin/main" fica como esta'.
    function refChip(ref) {
        if (typeof ref !== "string") return { name: "", head: false, tag: false };
        if (ref.indexOf("HEAD -> ") === 0) return { name: ref.substring(8), head: true, tag: false };
        if (ref === "HEAD") return { name: "HEAD", head: true, tag: false };
        if (ref.indexOf("tag: ") === 0) return { name: ref.substring(5), head: false, tag: true };
        return { name: ref, head: false, tag: false };
    }
}
