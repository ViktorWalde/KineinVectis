# Registro de saídas do dogfooding

> **Classe: LOG** (`docs/README.md`). Append-only, cada entrada datada. **Nunca
> reescrever uma entrada** — envelhecer é a função dela. Não é estado nem plano:
> não descreve o que a IDE é hoje, descreve o que **forçou sair dela** num dia
> específico.
>
> **Etapa 12 do [`roadmaps/34`](../../docs/roadmaps/34-depois-do-mvp.md) §4.1**,
> criada em 2026-09-03.

## Por que este arquivo existe

O `GUIAIA.md` §2 define o protocolo desde sempre: ao ouvir **"estou no Kinein"**,
registrar cada saída para outra ferramenta com motivo exato, projeto/arquivo,
ação que faltou, impacto e reprodução mínima — *"esses dados passam a ordenar o
backlog antes de confortos hipotéticos"*.

O protocolo existia. **O artefato não.** O que havia era prosa de sessão
(`PONTO_ATUAL.md` §0, um parágrafo denso), e prosa não se consulta: não dá para
ordenar a frente C do `roadmaps/34` por dor real lendo narrativa. A frente C tem
nove itens e **nenhum critério** para ordená-los; é este arquivo que produz o
critério.

**Sem isto, "o que falta na IDE" é opinião.**

## A regra que separa dado de impressão

> **Entrada sem reprodução mínima NÃO CONTA.** Não ordena nada, não vira fatia.

Uma impressão isolada não vira funcionalidade (`GUIAIA.md` §2, última linha:
*"reproduzir antes de alterar arquitetura"*). Se você não consegue escrever os
passos que reproduzem, a entrada fica **em quarentena** — registre assim mesmo,
marcada, mas ela não entra na ordenação.

## O que é uma SAÍDA (e o que não é)

```text
E' SAIDA        abandonei a Kinein e abri outra ferramenta para terminar a
                tarefa. O nome da ferramenta e o que ela fez entram na entrada.

NAO E' SAIDA    regressao visual, atrito ou bug que eu contornei DENTRO da
                Kinein. Isso e' bug e vai para a fila normal (PONTO_ATUAL),
                nao para ca'. Misturar os dois esvazia o valor deste arquivo:
                ele existe para medir o que a IDE nao consegue fazer, nao o
                que ela faz mal.
```

## Ordem de prioridade (`GUIAIA.md` §2)

```text
1  perda de dados / seguranca / crash
2  bloqueio diario  (forcou a saida)
3  regressao funcional
4  conforto / feature
```

Feedback de amigo ou testador usa o mesmo funil, **sempre identificado por
origem, distro e versão do artefato**.

## Formato de uma entrada

```markdown
### AAAA-MM-DD — <ferramenta que abri> — <resumo em uma linha>

- **motivo exato:**
- **projeto/arquivo:**
- **ação que faltou:**
- **impacto:**            (1 dados/crash · 2 bloqueio · 3 regressão · 4 conforto)
- **reprodução mínima:**  (sem isto a entrada não conta)
- **origem:**             (autor · testador: distro + versão do artefato)
```

## Entradas

### Estado medido em 2026-09-03: NENHUMA saída registrada

Medido ao criar este arquivo, e o resultado surpreende — por isso está escrito:

```bash
grep -rniE "abrir o vs ?code|voltei ao|precisei (do|de)|tive que usar" \
     docs-privada/ PONTO_ATUAL.md     # -> vazio
```

O gatilho **"estou no Kinein" já foi recebido** (`PONTO_ATUAL.md` §0). Mas o que
a prosa daquela seção registra são **regressões dentro da Kinein** — scrollback e
barra no Assistente, largura e fluidez no resize, faixa da entrada multilinha,
divisor que sumia, árvore `Project` fechando, scroll perdido em chat longo,
caret depois do texto, verde/bold agressivo do prompt. Pela definição acima,
**nada disso é saída**: são bugs, foram tratados nas correções 0.52, 0.56 e 0.57,
e o único que continua aberto (alinhamento do caret da TUI) já tem dono próprio
em `docs/roadmaps/26-terminal-rendering-parity-roadmap.md`.

**A leitura honesta:** zero saídas registradas **não** significa que a IDE
substituiu o VS Code. Significa que o dogfooding até aqui exercitou o
**Assistente/terminal** e não o ciclo completo de desenvolvimento — o critério do
TR1 (`GUIAIA.md` §2) é *uma semana desenvolvendo C/C++ e Rust sem abrir outro
editor*, e essa semana ainda não aconteceu. O arquivo nasce vazio de propósito,
com a medição registrada, para que a primeira entrada real seja reconhecível
como a primeira.

<!-- Novas entradas ENTRAM ABAIXO desta linha, mais recente primeiro. -->
