import QtQuick

// O TECLADO DA ARVORE, como decisao pura (P1 da
// `especificacoes/projetos-arquivos-e-integracao-desktop-0.3.md`).
//
// Medido em 2026-09-26: a arvore do projeto nao tinha teclado NENHUM — nem
// setas, nem Home/End, nem expandir/recolher, nem busca pelo nome digitado.
// Quem nao usa mouse nao navegava no projeto. A matriz da §4 daquela
// especificacao abre com essa linha, e e' por ela que a P1 comeca.
//
// AS LINHAS SAO PLANAS. O `ProjectTreeController` ja' mantem um `ListModel`
// achatado, com `depth` em cada linha — a arvore ja' esta' resolvida quando
// chega aqui. Entao navegar e' aritmetica de indice, e aritmetica de indice
// tem harness.
//
// O QUE ENTRA: um retrato das linhas visiveis e o indice de onde o cursor
// esta'. O QUE SAI: uma INTENCAO — mover, expandir, recolher, ativar. Quem
// mexe no modelo e' o controller; aqui ninguem toca em nada.
QtObject {
    id: root

    // Nada a fazer. Existe para que o chamador nunca precise conferir `null`.
    readonly property var nada: ({ "kind": "none", "index": -1 })

    function moverPara(indice) {
        return { "kind": "move", "index": indice };
    }

    // "E' PASTA?" TEM UM DONO SO', e ele ja' existia: `ProjectTreeRules`.
    // A catraca de duplicacao pegou a terceira copia desta pergunta no
    // projeto — e' literalmente o defeito que ela descreve, com `severity`
    // azul num painel e vermelha no outro.
    readonly property ProjectTreeRules arvore: ProjectTreeRules {}

    function ehPasta(linha) {
        return linha !== undefined && linha !== null && root.arvore.isDirectory(linha.kind);
    }

    // O PAI de uma linha e' a primeira linha ACIMA com profundidade menor.
    // Nao ha' ponteiro de pai no modelo plano, e nao precisa haver: a
    // profundidade ja' diz a estrutura inteira.
    function indiceDoPai(linhas, indice) {
        if (indice <= 0 || indice >= linhas.length) {
            return -1;
        }
        const alvo = linhas[indice].depth;
        for (let atual = indice - 1; atual >= 0; --atual) {
            if (linhas[atual].depth < alvo) {
                return atual;
            }
        }
        return -1;
    }

    // Seta para BAIXO/CIMA: uma linha visivel por vez, sem dar a volta.
    //
    // Dar a volta parece util e nao e': quem segura a seta para chegar ao fim
    // de uma pasta longa acaba no topo sem perceber que passou.
    function aoDescer(linhas, indice) {
        if (linhas.length === 0) {
            return root.nada;
        }
        if (indice < 0) {
            return root.moverPara(0);
        }
        return indice + 1 < linhas.length ? root.moverPara(indice + 1) : root.nada;
    }

    function aoSubir(linhas, indice) {
        if (linhas.length === 0) {
            return root.nada;
        }
        if (indice < 0) {
            return root.moverPara(linhas.length - 1);
        }
        return indice > 0 ? root.moverPara(indice - 1) : root.nada;
    }

    // SETA PARA A DIREITA: abre o que esta' fechado, e entra no que ja' esta'
    // aberto. E' o comportamento que todo navegador de arvore tem, e o motivo
    // e' que a mesma tecla serve para "mostre-me o conteudo" e "va' para ele".
    function aoAvancar(linhas, indice) {
        if (indice < 0 || indice >= linhas.length) {
            return root.nada;
        }
        const linha = linhas[indice];
        if (!root.ehPasta(linha)) {
            return root.nada;
        }
        if (!linha.expanded) {
            return { "kind": "expand", "index": indice };
        }
        // Ja' aberta: o primeiro filho e' a proxima linha, SE houver uma mais
        // funda. Pasta aberta e vazia nao leva a lugar nenhum.
        const proxima = indice + 1;
        if (proxima < linhas.length && linhas[proxima].depth > linha.depth) {
            return root.moverPara(proxima);
        }
        return root.nada;
    }

    // SETA PARA A ESQUERDA: fecha o que esta' aberto, e sobe para o pai no
    // resto dos casos. Simetrica da direita, e pelo mesmo motivo.
    function aoRecuar(linhas, indice) {
        if (indice < 0 || indice >= linhas.length) {
            return root.nada;
        }
        const linha = linhas[indice];
        if (root.ehPasta(linha) && linha.expanded) {
            return { "kind": "collapse", "index": indice };
        }
        const pai = root.indiceDoPai(linhas, indice);
        return pai >= 0 ? root.moverPara(pai) : root.nada;
    }

    function aoIrParaOTopo(linhas) {
        return linhas.length > 0 ? root.moverPara(0) : root.nada;
    }

    function aoIrParaOFim(linhas) {
        return linhas.length > 0 ? root.moverPara(linhas.length - 1) : root.nada;
    }

    // ENTER: abrir arquivo, ou abrir/fechar pasta. Uma tecla, duas leituras,
    // porque a pergunta de quem aperta e' a mesma — "mostre-me isto".
    function aoAtivar(linhas, indice) {
        if (indice < 0 || indice >= linhas.length) {
            return root.nada;
        }
        if (!root.ehPasta(linhas[indice])) {
            return { "kind": "activate", "index": indice };
        }
        return linhas[indice].expanded
             ? { "kind": "collapse", "index": indice }
             : { "kind": "expand", "index": indice };
    }

    // BUSCA PELO NOME DIGITADO: a partir da linha SEGUINTE, dando a volta.
    //
    // Aqui dar a volta e' certo, e a diferenca importa: digitar procura em
    // todo lugar, enquanto a seta anda numa direcao. Comeca depois do cursor
    // para que apertar a mesma letra passeie pelos nomes que comecam com ela.
    function aoDigitar(linhas, indice, prefixo) {
        const agulha = String(prefixo).toLowerCase();
        if (agulha === "" || linhas.length === 0) {
            return root.nada;
        }
        for (let passo = 1; passo <= linhas.length; ++passo) {
            const candidato = ((indice < 0 ? -1 : indice) + passo + linhas.length)
                            % linhas.length;
            const nome = String(linhas[candidato].name).toLowerCase();
            if (nome.indexOf(agulha) === 0) {
                return root.moverPara(candidato);
            }
        }
        return root.nada;
    }

    // O PREFIXO SE ACUMULA enquanto a digitacao e' rapida, e recomeca quando
    // ela para. Sem isto, procurar `README` exigiria apertar `r` seis vezes e
    // torcer; com isto, digitar `re` acha o que `r` sozinho nao distingue.
    readonly property int pausaMs: 900

    function prefixoAcumulado(anterior, quandoMs, agoraMs, tecla) {
        const continuando = anterior !== "" && (agoraMs - quandoMs) < root.pausaMs;
        return continuando ? anterior + tecla : tecla;
    }
}
