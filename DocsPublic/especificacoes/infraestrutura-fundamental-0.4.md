# Infraestrutura fundamental da base 0.4

> **Classe: PLANO / REQUISITOS.** Escopo obrigatório definido em 2026-11-03.
> A presença de um requisito aqui não declara implementação nem validação.
> A fronteira de lançamento pertence ao [62](../roadmaps/62-base-publica-0.4.md);
> o desenho de cada domínio continua nos documentos indicados abaixo.

## 1. Objetivo e fronteira

**Diretriz de evolução:** a 0.4 aproveita e fortalece a infraestrutura já
construída. Capacidades existentes são auditadas, integradas e tornadas robustas
nos seus donos. Capacidades ausentes entram por integração/orquestração de
ferramentas maduras e pelos pontos de extensão existentes, com provas próprias.

A 0.4 estabelece a base técnica para desenvolver software, firmware e cálculos
científicos na mesma IDE. A 0.5 aprofunda, melhora e amplia essa base. Detecção
de ambiente, DAP, QEMU, Windows, SVD, VFS, plotagem, notebooks e ferramentas
de engenharia têm entregas mínimas obrigatórias nesta versão. Estruturas vazias,
abas demonstrativas e uma classificação como opcionais não aceitam esses itens.

A ordem das linguagens é **C → C++ → Rust e integração Rust/C/C++ → Python**
pela [especificação comum](orquestracao-e-verificacao-de-linguagens.md).
C tem [etapa profunda dedicada](consolidacao-nativa-de-c.md), incluindo C23;
o aceite C1–C8 antecede a consolidação C++.
As especializações usam essa fundação; a expansão para todos os SDKs, placas e
sondas mantém a trilha do [52](../roadmaps/52-arquitetura-executavel-da-0.4.md).
O ciclo embarcado mínimo desta especificação integra o fechamento da base.

O produto orquestra ferramentas existentes. Configuração automática significa
descoberta e preparação assistida, com origem, escolhas e alterações visíveis.
Abrir um projeto não autoriza instalar ferramentas, executar seu código ou
reescrever manifests. O fluxo comum deve dispensar edição manual de JSON,
preservando configurações importadas e a possibilidade de revisão.

A frente científica executa cálculos em kernels/ferramentas do projeto e
apresenta seus resultados. Ela preserva a decisão de manter a emulação em
ferramentas externas e de não criar um motor próprio de simulação física.

## 2. Ponto de partida — inspeção de 2026-11-03

Esta auditoria de fontes identifica mecanismos reutilizáveis. Não executa nem
recertifica as jornadas; resultados anteriores conservam data e alcance no
[40.7](../roadmaps/40.7-registro-das-entregas.md).

| Frente | Evidência no checkout | Trabalho que ainda exige aceite |
| --- | --- | --- |
| Ambiente | `tools/`, `toolchain/`, contexto CDB/Cargo e donos de CMake/RunConfig | Matriz cross e geração/revisão de perfis sem configuração manual |
| DAP | `dap/adapter.rs`, `dap/session.rs`, `handlers/debug.rs` e `DebugController.qml`; GDB, LLDB, probe-rs e debugpy | Jornadas integradas, capacidades, falhas e recuperação nas plataformas exigidas |
| QEMU | `dap/server.rs` e `scripts/verificar_embarcado.py`, com máquina `lm3s6965evb` | Ação integrada de build/emulação, perfis compatíveis e provas ARM/RISC-V |
| SVD | `svdFile` no kit e no launch probe-rs; escopos em `DebugInspectView.qml` | Árvore de descrição offline e integração dos valores; repassar o arquivo ao adaptador não aceita o visualizador completo |
| Windows | `platform/windows.rs`, `kinein-sys`, descoberta MSVC, ponte e gate PowerShell | Retomar os aceites do [60](../roadmaps/60-windows-e-edicao-especial.md), executar a matriz nativa e validar instalador |
| Arquivos | Identidade documental, `fsops/`, Remote SSH com espelho e integração de containers | Contrato comum de recursos e contexto entre provedores, sem confundir espelho local com análise remota comprovada |
| Memória e bibliotecas | `size.rs`, `EmbeddedSizeView.qml` e Eigen no catálogo da Biblioteca | Consumo Flash/RAM correto, integração DSP e jornada científica reproduzível |

Não foram identificados frontend de notebook Jupyter, API compartilhada de
plotagem ou contrato VFS unificado nas áreas inspecionadas de core/UI.
Essas entregas exigem desenho próprio antes de código, reaproveitando os donos
de Python, documentos, jobs, serial e renderização; não criar motores paralelos.

## 3. Entregas obrigatórias

### F1 — Ambiente, toolchains e perfis de projeto

Reusar [toolchains por alvo](../integracoes/39-toolchains-por-alvo.md),
[compilação](../build/22-compilacao-c-cpp-rust.md) e os donos CMake/Cargo.
Detectar ferramentas por fontes conhecidas, versão e capacidades; representar
separadamente host, alvo, ABI, sysroot, SDK, compilador, linker e depurador.
Um nome de executável no PATH não comprova compatibilidade com o projeto.

O corpus mínimo cobre C/C++ de host, `arm-none-eabi`, RISC-V bare metal,
toolchain Espressif e target Rust. Distinguir RISC-V Linux de bare metal e
as toolchains Xtensa/RISC-V da Espressif. Preservar preset, kit e configuração
explicitamente escolhidos; ambiguidade produz escolha explicada.

Preparar perfis reutilizáveis a partir do projeto e do ambiente. CMake distingue
configuração compartilhada de configuração local conforme os
[presets oficiais](https://cmake.org/cmake/help/latest/manual/cmake-presets.7.html).
Cargo conserva os papéis de `Cargo.toml`, `.cargo/config.toml`, toolchain e
target; detectar um compilador não justifica alterar o manifesto de dependências.
Mostrar prévia/diff, destino e escopo antes da escrita; aplicar de forma
transacional e idempotente, sem sobrescrever configuração do projeto.

Aceitar com projetos novos e existentes, caminhos com espaços/Unicode,
ferramenta ausente, múltiplas candidatas, troca de alvo e configuração obsoleta.
O contexto selecionado deve chegar ao build, LSP, verificador e depuração.

### F2 — Depuração DAP consolidada

Estender o cliente e os consumidores existentes. O
[DAP](https://microsoft.github.io/debug-adapter-protocol/overview) padroniza a
comunicação com adaptadores; o
[GDB implementa DAP](https://sourceware.org/gdb/current/onlinedocs/gdb.html/Debugger-Adapter-Protocol.html).
Na cadeia com OpenOCD, a IDE usa DAP com GDB/adaptador, e este se conecta ao
servidor GDB do OpenOCD. Não criar um tradutor DAP/OpenOCD próprio.

O aceite inclui launch/attach aplicáveis ao perfil, breakpoints, continuar,
pausar, passos, stack, variáveis, evaluate, saída e encerramento. Negociar
capacidades e explicar indisponibilidade. Validar C/C++, Rust e Python em seus
contextos; GDB/QEMU exercita o alvo sem placa, e probe-rs conserva regressões
próprias. Prova física mantém registro separado.

Exercitar adaptador ausente/incompatível, crash, timeout, cancelamento,
desconexão, símbolos/fontes divergentes e reinício. Encerrar somente os
processos pertencentes à sessão e retirar frames/valores antigos da UI.

### F3 — Compilar e emular com QEMU

Disponibilizar uma ação integrada que configure quando necessário, compile e
inicie o artefato resultante no perfil de emulação selecionado. Usar os jobs,
RunConfig e servidor de debug existentes; permitir execução e depuração com
breakpoint no mesmo artefato. Expor máquina, arquitetura, imagem/ELF, argumentos,
console, estado e motivo de falha sem exigir montagem manual de comandos.

O QEMU exige uma [máquina compatível](https://www.qemu.org/docs/master/system/target-arm);
detectar `qemu-system-arm` não identifica a placa. O perfil precisa declarar
memória, boot e periféricos disponíveis. Não deduzir compatibilidade de um ELF
somente pela arquitetura, nem anunciar emulação genérica de STM32/ESP32.
QEMU da Espressif e outros emuladores conservam perfis/versionamento próprios.

Aceitar pelo menos uma fixture ARM e uma RISC-V com toolchains reais: build,
console, breakpoint, inspeção, parada e repetição. Build falho impede executar
artefato antigo; perfil incompatível e emulador ausente têm recuperação visível.
Firmware e sensores simulados não comprovam periféricos ou hardware físico.

Sem placa física, a validação de software embarcado usa QEMU com as fixtures
ARM/RISC-V e máquinas compatíveis acima. Registrar emulação, ferramenta,
perfil e operações comprovadas. Flash em placa, sonda, sinais elétricos e
periféricos físicos conservam evidências separadas e ficam não provados
enquanto não exercitados. A ausência de hardware permite avançar nas provas
independentes de software; não transforma emulação em aceite físico nem
dispensa requisitos obrigatórios de C6/F1–F5.

### F4 — Windows nativo e distribuição

Windows integra o fechamento da 0.4; deixa de ser uma entrega exigida somente
depois dela. O [60](../roadmaps/60-windows-e-edicao-especial.md) continua dono
do porte e de sua matriz. O mínimo é Windows 11 x86_64 com core e UI nativos,
toolchains efetivas e instalador validado em ambiente limpo.

Provar abrir/editar/salvar/recuperar, LSP, terminal, Git, configurar, compilar,
testar, executar e depurar. Cobrir descoberta por PATH, locais convencionais e
registro quando pertinente, `.exe`, quoting, Unicode, espaços, letras de
unidade, UNC, diferenças de caixa, ambiente e cancelamento de descendentes.
O suporte USB/serial e cross deve declarar os backends e dispositivos medidos.

Validar instalação, primeira execução, atualização e desinstalação preservando
projetos/configurações. WSL2 é contexto adicional, com caminhos e ferramentas
próprios; um teste em WSL2 não aceita Windows nativo. O Linux mantém regressões.
Sem execução da matriz no Windows, este bloqueador permanece aberto.

### F5 — Visualizador CMSIS-SVD

Carregar `.svd` e apresentar árvore dispositivo → periférico → registrador →
campo, com endereço, largura, acesso, descrição e reset. Essa estrutura segue o
[formato CMSIS-SVD](https://arm-software.github.io/CMSIS_5/5.8.0/SVD/html/svd_Format_pg.html).
Reusar parser aberto compatível com o projeto e validar sua licença; distribuir
somente descrições cuja licença permita redistribuição.

O visualizador deve funcionar offline, sem sessão de depuração. Durante uma
parada, associar valores ao alvo correto pelo adaptador e leitura de memória
existentes, mostrando indisponibilidade/obsolescência. Respeitar acesso e efeitos
de leitura descritos pelo SVD; não fazer polling indiscriminado de registradores.
Escrita mantém os aceites específicos da trilha de Embarcados.

Aceitar exemplos com herança, arrays, clusters e bitfields, arquivo inválido,
referências cíclicas e recurso não suportado, com limites de tamanho/profundidade.
Fixtures precisam ter origem/licença registradas. Um parser por modelo de chip
não substitui o suporte ao formato.

### F6 — Recursos de arquivos e VFS

Estabelecer uma camada de recursos com consumidores reais no editor, análise
e build. Cada recurso identifica provedor, autoridade/workspace, caminho e
revisão; separar URI, caminho local, caminho do alvo e posição LSP.
Preservar documentos ainda não salvos, snapshots e mapeamento de diagnósticos.

O mínimo inclui arquivos locais Linux/Windows e jornadas por SSH e container,
reusando espelho, transporte e montagens existentes. LSP, build e debugger devem
resolver arquivos no ambiente correto; não aplicar flags/sysroot remotos a
ferramentas locais por fallback silencioso. Provedores declaram leitura,
escrita, observação, atomicidade e limites, sem prometer semânticas que não têm.

Aceitar abrir, editar, salvar, navegar e diagnosticar em cada contexto; conferir
conflito externo, desconexão/reconexão, rename, links, caixa, Unicode e respostas
atrasadas. Invalidação de ambiente/versão deve alcançar todos os consumidores.
Uma interface abstrata sem provedores integrados não encerra F6.

### F7 — API de plotagem e dados locais

Construir um componente reutilizável de séries, eixos, unidades, timestamps,
zoom, pausa e exportação sobre Qt Quick. O
[scene graph do Qt](https://doc.qt.io/qt-6/qtquick-visualcanvas-scenegraph.html)
já permite renderização por APIs gráficas; aproveitar essa infraestrutura,
sem introduzir outro motor de UI. Renderização GPU deve ser medida no backend
real, com degradação explícita quando aceleração não estiver disponível.

Integrar um produtor científico e um fluxo serial, além da fixture sintética
reproduzível. Usar buffers limitados, lotes e redução de pontos quando
necessária; exibir perdas/amostragem e encerrar produtores sem bloquear a UI.
Registrar orçamento de pontos/taxa, latência, frames, CPU e memória com a
configuração da medição. Não assumir 120 FPS como prova de desempenho.

Dados de dispositivos e métricas locais ficam no ambiente do projeto, sob
ação explícita. Não há coleta de uso do produto nem envio de dados a serviço.

### F8 — Notebooks científicos integrados

Abrir, editar, salvar e reabrir `.ipynb` em uma superfície própria, com células
de código/Markdown, execução individual/em sequência, saída, erro, gráficos,
interrupção e reinício do kernel. Usar Python nativo do ambiente selecionado
e o [protocolo Jupyter](https://jupyter-client.readthedocs.io/en/stable/messaging.html)
em processo separado. Essa escolha preserva o contexto de extensões nativas e
a arquitetura Python do projeto. [Pyodide](https://pyodide.org/en/stable/) é
uma alternativa tecnológica para WebAssembly; sua adoção não é necessária ao
aceite do notebook nativo.

Aceitar NumPy/SciPy/Matplotlib no ambiente científico declarado, preservando
metadados e arquivos existentes. Identificar kernel, cwd, versão e dependências;
não executar células ao abrir. Limitar saídas, tratar kernel morto e respostas
antigas após reinício/troca de documento, sem perder células não salvas.
Exercitar exportação explícita de coeficientes/tabela para um projeto C/C++,
com build e comparação numérica contra referência. Não prometer tradução
automática de Python arbitrário para C.

### F9 — Bibliotecas e utilitários de engenharia

Reusar a Biblioteca para Eigen e acrescentar o fluxo CMSIS-DSP, com origem,
versão, licença, arquitetura e requisitos de compilação. O
[CMSIS-DSP](https://arm-software.github.io/CMSIS_6/latest/DSP/) oferece funções
otimizadas para Cortex-M/Cortex-A. Evidência de DSP no código pode motivar uma
sugestão; adicionar dependência exige escolha e prévia do projeto afetado.
Aceitar fixtures que configurem, compilem e exercitem o uso numérico aplicável.

Entregar conversor de ponto fixo, cálculo de divisor de tensão/filtro RC e
gerador LUT como consumidores da mesma estrutura de ferramentas. Explicitar
unidades, convenção de bits/sinal, arredondamento, saturação e domínio de entrada;
validar limites e exportar resultados reproduzíveis. Abas vazias não são aceite.

Consolidar o relatório Flash/RAM do ELF por `build.size`, apresentando origem,
capacidade e momento do build. Conferir endereços de carga/execução, seções
inicializadas e regiões do linker; não chamar tamanho estático de uso dinâmico.
Um percentual sem capacidade conhecida não deve ser inventado. Atualizar após
build válido e identificar resultado antigo enquanto o código muda.

## 4. Sequência e dependências

| Etapa | Trabalho e dependências |
| --- | --- |
| 1 | Consolidar contexto/build/LSP de C de host e o verificador na mesma fixture, depois aprofundar C++; ambiente F1, identidade VFS e portabilidade acompanham os consumidores |
| 2 | Consolidar DAP F2 e ciclo QEMU F3; extrair regressões reais de erro/cancelamento e preservar as capacidades embarcadas existentes |
| 3 | Consolidar Rust e fronteiras Rust/C/C++; fechar provedores F6 e matriz/instalador Windows F4; integrar árvore SVD F5 ao debug existente |
| 4 | Consolidar Python; implementar notebooks F8, plotagem F7 e ferramentas/bibliotecas F9 com jornadas científicas reais |
| 5 | Repetir jornadas finais Linux/Windows, polimento, desempenho, idiomas, contribuição, documentação e pacote conforme o 62 |

O desenho de Windows/VFS começa com a primeira etapa; sua prova integrada
depende dos consumidores. Integridade, falhas e regressões podem alterar a
ordem das correções. O acabamento cromático de Instalar Ferramentas acompanha
as jornadas pela [especificação §0.1](instalar-ferramentas.md#01-polimento-visual-e-de-interação-da-base-04).

A relação entre ferramentas e algoritmo segue a
[orquestração §12.1](orquestracao-e-verificacao-de-linguagens.md#121-dependências-entre-ferramentas-e-verificador):
contexto e ferramentas consolidados por jornada, verificador endurecido na
mesma fatia e validação final nos ambientes suportados. Os demais fundamentos
mantêm seus aceites de release; sua execução integral não é pré-condição para
aprimorar a comparação host já integrada.

## 5. Evidências de fechamento e contribuição

Cada F1–F9 exige implementação, integração na UI e prova reproduzível, com
fixture, versão das ferramentas, plataforma, passos e limites. Ferramenta ou
ambiente ausente deixa o item NÃO PROVADO; não o converte em opcional.
Aceitar tanto o ciclo abrir → detectar → configurar → compilar → emular →
depurar quanto o ciclo notebook → resultado numérico → exportação → build.

A preparação para colaboradores e divulgação também integra a 0.4, conforme
[contribuindo/11](../contribuindo/11-ci-publica-e-tarefas-para-colaboradores.md):
README fiel, instruções reproduzíveis, CI, templates, tarefas pequenas revisadas,
labels e demonstração do fluxo comprovado. Divulgação não deve afirmar
primazia mundial nem capacidades sem evidência. O marco da base é a entrega
dos aceites; campanhas públicas acompanham a versão efetivamente validada.
