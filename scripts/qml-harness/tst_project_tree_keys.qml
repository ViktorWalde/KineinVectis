// O TECLADO DA ARVORE DO PROJETO (P1).
//
// Por que existe (2026-09-26): medido no checkout, a arvore nao tinha teclado
// nenhum — zero `Keys.` no `ProjectExplorer` e no `ProjectTreeController`.
// Quem nao usa mouse nao navegava no projeto, e a matriz da §4 da
// `especificacoes/projetos-arquivos-e-integracao-desktop-0.3.md` abre por essa
// linha.
//
// O modelo da arvore ja' e' PLANO, com `depth` em cada linha: navegar e'
// aritmetica de indice, e aritmetica de indice se mede.
//
// MUTACOES QUE PROVAM O GATE: faca `aoDescer` dar a volta (`% length`) e a
// assercao do fim cai; troque `linhas[atual].depth < alvo` por `<=` no
// `indiceDoPai` e a esquerda passa a parar num irmao em vez do pai.
import QtQuick
import KineinVectis

Item {
    id: root

    property int falhas: 0

    ProjectTreeKeyRules {
        id: teclas
    }

    // Uma arvore de verdade, achatada como o controller a mantem:
    //
    //   src/            depth 0, aberta
    //     main.rs       depth 1
    //     lib/          depth 1, fechada
    //   docs/           depth 0, aberta e VAZIA
    //   README.md       depth 0
    readonly property var linhas: [
        { name: "src", kind: "directory", depth: 0, expanded: true },
        { name: "main.rs", kind: "file", depth: 1, expanded: false },
        { name: "lib", kind: "directory", depth: 1, expanded: false },
        { name: "docs", kind: "directory", depth: 0, expanded: true },
        { name: "README.md", kind: "file", depth: 0, expanded: false }
    ]

    function conferir(condicao, mensagem) {
        if (!condicao) {
            console.error("FALHOU: " + mensagem);
            root.falhas += 1;
        }
    }

    function conferirIntencao(intencao, kind, indice, mensagem) {
        root.conferir(intencao.kind === kind && intencao.index === indice,
                      mensagem + " (veio " + intencao.kind + "/" + intencao.index + ")");
    }

    Component.onCompleted: {
        const l = root.linhas;

        // AS SETAS ANDAM UMA LINHA VISIVEL POR VEZ.
        root.conferirIntencao(teclas.aoDescer(l, 0), "move", 1, "descer nao andou");
        root.conferirIntencao(teclas.aoSubir(l, 1), "move", 0, "subir nao andou");

        // E NAO DAO A VOLTA. Quem segura a seta no fim de uma pasta longa
        // acabaria no topo sem perceber que passou.
        root.conferirIntencao(teclas.aoDescer(l, 4), "none", -1, "desceu alem da ultima");
        root.conferirIntencao(teclas.aoSubir(l, 0), "none", -1, "subiu alem da primeira");

        // SEM CURSOR, a primeira tecla escolhe a ponta de onde ela vem.
        root.conferirIntencao(teclas.aoDescer(l, -1), "move", 0, "descer sem cursor");
        root.conferirIntencao(teclas.aoSubir(l, -1), "move", 4, "subir sem cursor");

        // DIREITA: abre o que esta' fechado...
        root.conferirIntencao(teclas.aoAvancar(l, 2), "expand", 2, "direita nao abriu `lib`");
        // ...e entra no que ja' esta' aberto.
        root.conferirIntencao(teclas.aoAvancar(l, 0), "move", 1, "direita nao entrou em `src`");
        // Pasta aberta e VAZIA nao leva a lugar nenhum — `docs` e' seguida de
        // uma linha da MESMA profundidade, entao nao tem filho.
        root.conferirIntencao(teclas.aoAvancar(l, 3), "none", -1, "direita entrou em pasta vazia");
        // Arquivo nao abre nada.
        root.conferirIntencao(teclas.aoAvancar(l, 4), "none", -1, "direita fez algo num arquivo");

        // ESQUERDA: fecha o que esta' aberto...
        root.conferirIntencao(teclas.aoRecuar(l, 0), "collapse", 0, "esquerda nao fechou `src`");
        // ...e sobe para o PAI no resto. `main.rs` (depth 1) sobe para `src`.
        root.conferirIntencao(teclas.aoRecuar(l, 1), "move", 0, "esquerda nao subiu ao pai");
        // Pasta FECHADA sobe ao pai, em vez de tentar fechar de novo.
        root.conferirIntencao(teclas.aoRecuar(l, 2), "move", 0, "pasta fechada nao subiu ao pai");
        // Na raiz nao ha' pai.
        root.conferirIntencao(teclas.aoRecuar(l, 4), "none", -1, "achou pai de uma linha raiz");

        // O PAI E' A PRIMEIRA LINHA ACIMA COM PROFUNDIDADE MENOR — e nao um
        // irmao de mesma profundidade.
        root.conferir(teclas.indiceDoPai(l, 2) === 0,
                      "o pai de `lib` nao e' `src`: " + teclas.indiceDoPai(l, 2));
        root.conferir(teclas.indiceDoPai(l, 0) === -1, "inventou pai para a primeira linha");

        // HOME e END.
        root.conferirIntencao(teclas.aoIrParaOTopo(l), "move", 0, "Home errou");
        root.conferirIntencao(teclas.aoIrParaOFim(l), "move", 4, "End errou");

        // ENTER: arquivo abre; pasta alterna.
        root.conferirIntencao(teclas.aoAtivar(l, 1), "activate", 1, "Enter nao abriu o arquivo");
        root.conferirIntencao(teclas.aoAtivar(l, 0), "collapse", 0, "Enter nao fechou a aberta");
        root.conferirIntencao(teclas.aoAtivar(l, 2), "expand", 2, "Enter nao abriu a fechada");

        // BUSCA PELO NOME DIGITADO: comeca DEPOIS do cursor e da' a volta —
        // ao contrario da seta, digitar procura em todo lugar.
        root.conferirIntencao(teclas.aoDigitar(l, -1, "re"), "move", 4, "nao achou `README.md`");
        root.conferirIntencao(teclas.aoDigitar(l, 4, "s"), "move", 0, "a busca nao deu a volta");
        root.conferirIntencao(teclas.aoDigitar(l, 0, "MAIN"), "move", 1,
                              "a busca distinguiu maiuscula");
        root.conferirIntencao(teclas.aoDigitar(l, 0, "zzz"), "none", -1,
                              "achou o que nao existe");
        root.conferirIntencao(teclas.aoDigitar(l, 0, ""), "none", -1,
                              "prefixo vazio moveu o cursor");

        // APERTAR A MESMA LETRA PASSEIA pelos nomes que comecam com ela — e
        // e' so' por isso que a busca comeca na linha SEGUINTE. Com dois
        // nomes em `s`, ficar parado no primeiro seria o defeito.
        const doisEsses = [
            { name: "src", kind: "directory", depth: 0, expanded: false },
            { name: "setup.py", kind: "file", depth: 0, expanded: false }
        ];
        root.conferirIntencao(teclas.aoDigitar(doisEsses, 0, "s"), "move", 1,
                              "`s` com o cursor em `src` nao foi para `setup.py`");
        root.conferirIntencao(teclas.aoDigitar(doisEsses, 1, "s"), "move", 0,
                              "`s` de novo nao voltou para `src`");

        // O PREFIXO SE ACUMULA enquanto a digitacao e' rapida, e recomeca
        // quando ela para. Sem isto, `README` exigiria apertar `r` seis vezes.
        root.conferir(teclas.prefixoAcumulado("r", 1000, 1200, "e") === "re",
                      "a digitacao rapida nao acumulou");
        root.conferir(teclas.prefixoAcumulado("r", 1000, 1000 + teclas.pausaMs + 1, "e") === "e",
                      "a digitacao lenta nao recomecou");
        root.conferir(teclas.prefixoAcumulado("", 0, 5000, "r") === "r",
                      "a primeira tecla nao virou prefixo");

        // LISTA VAZIA nao explode nem inventa indice.
        root.conferirIntencao(teclas.aoDescer([], -1), "none", -1, "desceu numa lista vazia");
        root.conferirIntencao(teclas.aoIrParaOFim([]), "none", -1, "End numa lista vazia");
        root.conferirIntencao(teclas.aoAtivar([], 0), "none", -1, "Enter numa lista vazia");

        Qt.exit(root.falhas === 0 ? 0 : 1);
    }
}
