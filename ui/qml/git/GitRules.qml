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

    // As raias do grafo: a atribuicao classica em linha reta. Cada linha e'
    // { lane, merge, laneCount }: `lane` e' a coluna do commit; um commit com
    // dois pais e' `merge`; `laneCount` e' quantas colunas estao vivas ali.
    // Entradas: [{ sha, parents: [sha] }], do mais novo ao mais velho.
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
            out.push({ lane: lane, merge: parents.length > 1, laneCount: Math.max(1, active.length, lane + 1) });
        }
        return out;
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
