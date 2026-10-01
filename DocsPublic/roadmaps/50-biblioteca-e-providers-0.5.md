# 50 — Biblioteca por capacidades e providers na série 0.5

> **Classe: PLANO / ALVO.** Derivado em 2026-09-29 de dois estudos do autor
> (não publicados), confrontados com a série 0.3 e com as decisões
> de 0.4. A [casca da IDE e a ordem das versões](49-frontend-0.3.6-e-sequencia-0.5.md)
> estão no roadmap 49. Este arquivo é autossuficiente para execução; os estudos
> privados são referência de ideias e catálogo, não prova de implementação.

## 1. Resultado de produto

A Biblioteca deixa de ser apenas um catálogo curado de bibliotecas C/C++ e
passa a apresentar **capacidades**: análise estática, cobertura, perfilamento,
segurança, dependências, embarcados etc. Cada capacidade explica sua finalidade,
mostra os providers conhecidos, identifica o que está efetivamente disponível
no projeto e leva à configuração ou execução pelo caminho já existente.

```text
capacidade → provider adequado → detectar/explicar → escolher/configurar
           → Job existente → resultado nas superfícies existentes
```

Isto não é marketplace, API de plugins nem autorização para instalar dezenas
de ferramentas. Uma ferramenta só entra como suportada depois de prova real
de detecção, versão, execução, cancelamento, saída e erro.

## 2. Estado de partida e donos

| Responsabilidade | Dono atual a estender | Regra |
| --- | --- | --- |
| Detecção e versão de ferramentas | `ToolDetector` e catálogo conhecido no core | Não criar segundo scanner da máquina |
| Catálogo de bibliotecas e fluxo escolher/entender/aplicar | domínio `library` e `LibraryPanel` | Preservar licenças, origem, docs e preview |
| Configuração de projeto | Configuration Actions, Settings e donos do domínio | Um valor efetivo, vários atalhos para ele |
| Execução longa | Job System e handlers do core | QML nunca lança ferramenta externa |
| Diagnósticos e cobertura | Problems, Quality, Coverage e modelos existentes | Normalizar só quando houver consumidor real |
| Ações e descoberta | `command.list`, `CommandDispatcher`, Search Everywhere | Sem segunda paleta nem segundo dispatcher |
| Transporte | `CoreClient` e JSON-RPC tipado | Método novo apenas para fronteira nova |

O código decide o estado real no início de cada fatia. `clang-tidy`, cobertura,
sanitizers, CDB, ferramentas Python e as bibliotecas curadas já têm partes
funcionais: sua aparição na nova Biblioteca será uma **exposição do existente**,
não uma integração nova anunciada duas vezes. `ccache`/`sccache` têm plano para
a 0.4; se essa fatia os implementar, a 0.5 apenas incorpora seu estado ao
catálogo. Documentado, detectado e utilizável são três estados distintos.

## 3. Capacidade primeiro, provider depois

O usuário começa por uma pergunta: “quero encontrar problema de memória” ou
“quero analisar dependências”. A UI apresenta o objetivo, os providers
compatíveis, o motivo de uma opção estar pronta ou indisponível e os efeitos
da execução. Nomes comerciais aparecem no detalhe, não viram navegação
permanente do shell.

Metadados mínimos de um provider, definidos no core quando a primeira fatia
provar a necessidade: ID estável, nome, capacidades, tipo, licença/fonte,
documentação, candidatos de binário, forma de consultar versão, compatibilidade
com projeto/alvo, necessidades de rede/dispositivo/privilégio, política de
instalação, tipos de saída e restrições de execução. Campos sem consumidor
real não entram no tipo inicial só porque constam do catálogo de pesquisa.

Estado visível deve distinguir pelo menos: conhecido, não instalado, detectado,
incompatível, precisa de configuração, pronto, requer licença e falhou. A UI
deve dizer **por quê**, **quando foi medido** e **o que fazer**. “Detectado”
não equivale a “pronto”. Um provider pode oferecer várias capacidades; uma
capacidade pode ter vários providers.

O registro é **interno e restrito ao domínio Library/Quality**. Não generalizar
para registry universal de UI, API pública de plugin ou execução de código do
projeto. A abstração só é extraída depois de dois providers de tipos distintos
exercitarem o mesmo contrato sem duplicação de estado.

## 4. Contrato de execução e resultados

Quando um provider for executável, a intenção sai da Library, Search ou
Quality pelo mesmo command ID; o core valida escopo, binário, compatibilidade
e consentimento; a execução retorna um `jobId`; os eventos e `job.cancel`
existentes controlam progresso e cancelamento. A saída bruta e o artefato
ficam acessíveis para auditoria.

Antes de criar `provider.list/status/run`, comparar com os métodos atuais de
`tool`, `library`, `quality` e `setup`: estender o dono existente quando a
responsabilidade for a mesma. Um domínio `provider.*` só entra quando o
contrato comum for realmente consumido por providers diferentes. Não adicionar
`provider.cancel` paralelo a `job.cancel`.

Primeira família normalizada: `Diagnostic`, com fonte, regra, severidade,
mensagem original, arquivo/range, ajuda, referência à saída bruta e estado de
validade. Depois, se houver dois consumidores, `SecurityFinding`, `Metric` e
`Artifact`. Os demais formatos sugeridos pelo catálogo privado (grafo, trace,
evento de hardware, tamanho binário etc.) permanecem candidatos. Problems e
Quality mostram o resultado de seu domínio; uma falha de parser não vira
“nenhum problema encontrado”.

## 5. Sequência proposta para a 0.5

As letras abaixo são fatias verificáveis, não promessas de patch version.

### 0.5-A — mapear capacidades e expor o que já existe

- Inventariar ToolDetector, Library, Setup, Configuration Actions, Quality,
  Coverage e seus estados reais.
- Definir taxonomia curta de capacidades e o dono de cada gesto: descobrir,
  configurar, executar, resultado e voltar.
- Desenhar Library por capacidade com busca/filtro, estados explicados, fonte,
  licença, documentação e escopo global/workspace/alvo.
- Expor primeiro uma capacidade já funcional para provar a informação sem
  introduzir nova execução.

**Aceite:** nenhum estado configurado é apresentado como efetivo sem prova do
core; a Library e Settings/Environment não mantêm cópias do mesmo valor.

### 0.5-B — provar dois tipos de provider

O par de prova proposto é **Cppcheck** (CLI que pode produzir diagnóstico) e
**Renode** (processo externo ligado ao fluxo embarcado). Antes de escolhê-los
definitivamente, medir disponibilidade, formato de saída, licença e aderência
à versão embarcada 0.4. Se um falhar no critério, substituí-lo por outro de
tipo diferente e registrar o motivo.

- Detecção e versão são reais, inclusive falha e binário ausente.
- Compatibilidade depende do projeto/target, sem assumir que instalado basta.
- Execução explícita usa Jobs, com cancelamento e saída bruta.
- Sem instalação silenciosa, `sudo`, redistribuição automática ou captura de
credencial em perfil.

**Aceite:** os dois cabem no mesmo contrato de metadados/status/execução sem
switch nominal no shell; seus resultados aparecem no lugar certo; nenhum
estado “pronto” é atribuído por documentação apenas.

### 0.5-C — resultado coerente e qualidade cotidiana

Normalizar diagnósticos de um provider e provar arquivo/range, severidade,
mensagem original, saída bruta, cancelamento e resultado obsoleto. Expandir a
um segundo produtor, como ShellCheck ou Clang Static Analyzer, somente depois
da paridade. Search Everywhere e comandos encontram a capacidade pelo objetivo
e retornam à superfície correta. Quality reúne evidências existentes sem virar
outro catálogo de ferramentas.

**Aceite:** um erro de build, análise ou parser preserva origem e texto bruto;
um resultado antigo não se apresenta como atual; teclado e filtro funcionam
em lista curta e volumosa.

### 0.5-D — expandir por uso e evidência

Fila candidata, não lote obrigatório:

| Domínio | Candidatos para avaliar após B/C | Dependência |
| --- | --- | --- |
| Build/dependências | Conan, vcpkg, Meson, `ccache`, `sccache` | Reusar decisão e implementação da 0.4 |
| Qualidade e segurança | IWYU, OSV-Scanner, Gitleaks, Bandit | Formato/risco/licença medidos |
| Desempenho local | Valgrind, `perf`, heaptrack, Bloaty | Jobs, artefatos e permissão explicada |
| Rust/Python | cargo-nextest, cargo-llvm-cov, Miri, pip-audit | Workspace e toolchain compatíveis |
| Embarcados/hardware | pyOCD, sigrok-cli, can-utils, tshark | Fluxo 0.4 e dispositivo real |

`/proc` para processo Linux e `perf` como processo são compromissos já
registrados para 0.5 no
[modelo semântico](../especificacoes/modelo-semantico-do-projeto-0.4.md).
RTT/SWO/semihosting pertencem à superfície embarcada. P1/P2 do estudo privado
ficam como radar até aparecerem caso real e mantenedor para a integração.

## 6. Regra de entrada no catálogo curado

Para cada candidato, registrar: necessidade do usuário; capacidade e
superfície dona; estado atual no código; fonte oficial e licença revisadas na
data da fatia; binário/versão e compatibilidade detectáveis; entrada/saída
parseável ou artefato útil; erro e cancelamento; necessidades de rede,
dispositivo ou privilégio; política de distribuição; prova real com a versão
testada. “Conhecido pela comunidade” e “verificado pela Kinein” são rótulos
distintos. O segundo exige teste com ferramenta real e gate que detecte quebra.

Ferramentas comerciais permanecem externas e opcionais. Nada é instalado ou
executado só porque o projeto foi aberto. A Library oferece documentação e
um comando/ação com prévia e consentimento quando houver instalação suportada.

## 7. Critério de pronto da primeira 0.5 publicável

Um usuário consegue procurar uma capacidade, entender qual provider está
disponível e por quê, configurá-lo sem editar JSON manualmente no fluxo
suportado, executá-lo como Job, cancelar, inspecionar a saída original e seguir
um resultado até arquivo/problema/artefato. Os dois providers de prova são
exercitados em condições reais. O shell não ganha ícones permanentes por
provider, a Library não duplica Settings/Environment, e a casca da 0.3.6 não
perde desempenho nem navegação por teclado.

## 8. Adendo de 2026-10-01 — superfície da Library, Welcome e Wizard

Dois estudos do autor (não publicados) detalham o
que esta frente apresenta. Eles entram como direção de desenho. Cada fatia
continua sujeita às regras das §§2–7: prova real, um dono por valor e nada de
marketplace.

- **Library como superfície completa** (cabeçalho só com escopo e busca,
  navegação interna *My Workspace* → *Discover* por categoria, resultados e
  detalhes). Workspace aberto abre em *My Workspace*; sem workspace, em
  *Discover*. Em largura compacta, os detalhes substituem a lista com "voltar",
  sem terceira coluna. Depende da área de superfície completa da
  [0.3.6](49-frontend-0.3.6-e-sequencia-0.5.md) (§9).
- **Estados explícitos** em texto e ícone, nunca só cor: provider
  (embutido, detectado, pronto, disponível, precisa de configuração, requer
  licença, versão incompatível, não suportado, desabilitado) e capability
  (disponível, habilitada, ativa, parcial, indisponível, desabilitada). Não há
  toggle único ambíguo, nem nota ou ranking. Toda recomendação diz o motivo e
  mostra no máximo 3 a 5 itens.
- **Detalhe de provider** com origem, caminho, versão detectada, licença,
  política de distribuição, compatibilidade com o alvo e "última validação".
  Configurar para o workspace leva ao dono no Environment/domínio. A Library não
  copia o formulário.
- **Busca por nome, conceito e problema** ("vazando memória" → Memory
  Analysis), indexada localmente e integrada ao Search Everywhere por deep link.
- **Desempenho:** abrir a Library desenha os metadados em cache primeiro. A
  detecção é assíncrona e em lote, e não dispara um processo por provider ao
  abrir. Varredura de rede ROS ou de probe só acontece com gesto explícito.
- **Migração:** `LibraryPanel` → `LibraryShell` com subviews. A Library C/C++
  atual vira a subview *Development › C/C++ › Libraries* e preserva target
  picker, preview e Configuration Actions, sem big-bang.
- **Welcome e Project Wizard (fatia nova, 0.5-W):** a Welcome oferece
  Criar/Abrir/Clonar, recentes com tags de contexto, resumo do ambiente e
  "Explorar Biblioteca". O Wizard pergunta primeiro o tipo (Software,
  Embarcado, ROS 2, Vazio), depois linguagem/ecossistema e build system, e
  mostra preview antes de criar. Python fica no mesmo nível que C/C++/Rust. Os
  templates atuais `cppCmake`/`rustCargo` passam para dentro do Wizard, e
  `newProjectRequested(templateId)` vira a abertura do Wizard. GitHub é
  opcional e não há login.

Ordem proposta: 0.5-A passa a incluir o shell da Library e *My Workspace*
sobre o que já existe; 0.5-W pode correr em paralelo a 0.5-B, porque não
depende de provider novo.
