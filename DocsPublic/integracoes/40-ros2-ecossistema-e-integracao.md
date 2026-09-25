# ROS 2 na Kinein Vectis — avaliação do ecossistema e da integração

**Estado:** pesquisa e proposta de produto; nenhuma integração adotada por este documento.  
**Pesquisa:** 2026-09-24 a 2026-09-25; consolidação: 2026-09-25.  
**Horizonte:** futuro, 0.4+; não integra o escopo da 0.3 e não reserva uma versão de entrega.  
**Premissa:** IDE C/C++/Rust/Python, Qt/QML → JSON-RPC sobre stdio → core Rust → ferramentas como processos.

## 1. Parecer e limite da evidência

**ROS 2 tem bom encaixe na Vectis, sobretudo para projetos C++/Python e sistemas que combinam Linux com microcontroladores.** A integração que vale construir primeiro é a continuidade entre ambiente, pacotes, build, execução e depuração. Visualização 3D, análise de sinais e simulação devem inicialmente abrir ferramentas externas especializadas. Esta é uma avaliação de produto, fundamentada nas capacidades e lacunas discriminadas abaixo; não é uma afirmação de suporte já entregue.

A avaliação por linguagem é desigual: C++ e Python têm bibliotecas cliente mantidas pela equipe principal do ROS 2; C e Rust aparecem na categoria comunitária da documentação. Isso não impede seu uso, mas impede anunciar cobertura equivalente sem provas específicas. [Bibliotecas cliente, consulta 2026-09-24][clients]

O recorte recomendado é:

| Frente | Avaliação para a Vectis | Condição para anunciar suporte |
| --- | --- | --- |
| C++/Python em Linux | Primeira fatia ROS 2 recomendada | Workspace com interfaces próprias, build/teste, ambiente do editor e depuração de um processo comprovados |
| C | Compatível com a premissa, especialmente `rcl`/`rclc` e firmware | Não confundir biblioteca C de infraestrutura com experiência completa equivalente a `rclcpp` |
| Rust | Frente explicitamente qualificada por biblioteca e versão | Separar `rclrs` de `r2r`; provar geração de mensagens, build e debug da combinação escolhida |
| Linux embarcado remoto | Reaproveitamento potencial de SSH/rsync e DAP | Provar ambiente remoto, ABI, bibliotecas e mapeamento de fontes; copiar um ELF não basta |
| micro-ROS em MCU | Ponte futura com a frente Embarcados | Firmware, transporte, Agent e mensagens precisam de validação conjunta, incluindo placa real |
| Visualização e simulação | Integração de ferramentas como processos | Identificar versões, configuração, responsabilidade pelo processo e resultado observável |

São preservadas as decisões de [integrações nativas](README.md), os [modos de adaptação de componentes abertos](../roadmaps/adaptacao-de-plugins-abertos.md) e os levantamentos de [ferramentas de embarcados](36-ferramentas-de-embarcados.md) e [toolchains por alvo](39-toolchains-por-alvo.md). ROS 2 não substitui o painel Embarcados nem desfaz a decisão de dedicar uma versão ao seu layout, atalhos e fluxo.

### 1.1 O que foi efetivamente verificado

Foram consultados documentação oficial, repositórios dos mantenedores, arquivos de licença, manifests, código selecionado, testes selecionados e metadados do GitHub. Algumas páginas de `docs.ros.org` recusaram acesso automatizado; nesses casos foi lida a fonte pública correspondente em `ros2/ros2_documentation`. Não se usaram anúncios, estrelas ou listas de terceiros como prova de confiabilidade.

A implementação local foi examinada no HEAD `34cec5bffd25727c04dab964aacc00e78f30b93c`. As seguintes medições foram feitas em 2026-09-24, sem instalar ferramentas e sem alterar a configuração do shell do usuário:

| Verificação | Resultado observado | O que não prova |
| --- | --- | --- |
| Busca de `ros2` no PATH inicial | Não encontrado | Ausência de ROS no computador |
| Inspeção de `/opt/ros` | Prefixo `lyrical` presente | Integridade de todos os pacotes |
| Carregamento de `/opt/ros/lyrical/setup.bash` em subprocesso Bash e `ros2 --help` | CLI executou; apresentou `launch`, `bag`, `node`, `topic`, `control`, `lifecycle` e outros verbos | Comunicação, funcionamento de controladores ou qualquer robô |
| `ros2 node list --help` | Expôs `--no-daemon` e `--spin-time` | Descoberta de nós; nenhuma listagem de grafo foi executada |
| `ros2 launch --help` | Expôs argumentos, `--show-args`, `--launch-prefix` e filtro de prefixo | Execução ou análise de um launch real |
| `ros2 pkg executables demo_nodes_cpp` | Listou, entre outros, `talker` e `listener` | Execução ou troca de mensagens entre os exemplos |
| Metadados Python e `colcon build --help` | `colcon-core` 0.21.2, `colcon-cmake` 0.2.30, `colcon-ros` 0.5.0; seleção de pacotes e handler `compile_commands` disponíveis | Build ROS 2 concluído; handler instalado não significa compilation database válido |
| `rosdep --version` | 0.27.0 | Cache atualizado ou dependências satisfeitas |
| `package.xml` no prefixo Lyrical | `ros2cli` 0.40.8; `rclcpp` 32.0.2; `rclpy` 10.0.10; `rmw_fastrtps_cpp` 9.4.9; `rosbag2` 0.33.3; `rviz2` 15.2.5; `rqt_gui` 1.10.6 | Execução dessas bibliotecas e interfaces gráficas |
| Busca de manifests nesse mesmo prefixo | Não encontrados `rmw_cyclonedds_cpp`, `rmw_zenoh_cpp`, `micro_ros_agent`, `ros2trace`, `plotjuggler_ros` | Ausência em outros prefixos, containers ou máquinas |

**Não foram realizados:** build de workspace ROS, teste de pub/sub, depuração ROS via Vectis, execução de plugins, captura de tráfego, simulação Gazebo, testes em rede, medição de desempenho ou teste de hardware. Os testes futuros deste documento continuam pendentes. Não há capacidade ROS 2 promovida a “pronta”.

## 2. O que ROS 2 acrescenta ao modelo de uma IDE

### 2.1 Distribuição, plataforma e middleware são parte do projeto

A lista oficial consultada registra Lyrical Luth em maio de 2026, com suporte até maio de 2031; Jazzy até maio de 2029; Humble até maio de 2027; Kilted até dezembro de 2026. Rolling acompanha desenvolvimento e pode introduzir mudanças incompatíveis. Esses prazos são do projeto ROS, não compromissos de suporte da Vectis. [Distribuições, consulta 2026-09-24][releases]

| Combinação | Evidência oficial consultada em 2026-09-24 | Consequência proposta |
| --- | --- | --- |
| Lyrical + Ubuntu 26.04 amd64/arm64 | Tier 1; C++20, C17, Python 3.12–3.14; tabela de dependências de Ubuntu registra Python 3.14.3 [Plataformas Lyrical][platforms] | Candidata atual para um gate próprio; a presença local não comprova o gate |
| Jazzy + Ubuntu 24.04 amd64/arm64 | Tier 1; referência de linguagem C++17; plataforma Ubuntu usa Python 3.12.3 [REP-2000][rep2000] | Segunda combinação candidata, relevante para projetos existentes |
| Lyrical + Ubuntu 24.04 | Tier 3, compilação a partir das fontes segundo a página da distribuição [Plataformas Lyrical][platforms] | Não tratar como equivalente ao pacote oficial de 26.04 |
| Humble/Kilted | Ainda constam como suportadas nas datas acima [Distribuições][releases] | Medir quando encontradas; não ampliar a matriz prometida sem demanda e testes |

A REP-2000 atual informa que, de Lyrical em diante, plataformas são documentadas na página da distribuição. Consultar somente o antigo arquivo da REP deixaria essa avaliação incompleta. [REP-2000, consulta 2026-09-24][rep2000]

ROS 2 admite diferentes implementações RMW: Fast DDS, Cyclone DDS e também Zenoh, que não é DDS. A documentação não garante interoperabilidade universal entre fornecedores. A IDE deve registrar a implementação efetivamente selecionada e seus arquivos de configuração, sem substituir middleware por conveniência. [Middleware, consulta 2026-09-24][middleware]

### 2.2 Workspace, pacote e executável são entidades diferentes

`colcon` coordena um conjunto de pacotes; extensões acrescentam os comportamentos de CMake e ROS. `--packages-select` seleciona nomes; `--packages-up-to` inclui dependências recursivas. `--merge-install` e `--symlink-install` alteram a instalação produzida, não equivalem a escolher Debug ou Release. Esses argumentos foram conferidos na documentação e na ajuda local. [Build][colconbuild], [seleção de pacotes][colconselect], consultas 2026-09-24.

**Modelo proposto:** workspace contém pacotes com nome, caminho, build type, dependências e evidência de origem; pacote contém interfaces, executáveis e launches; artefato instalado mantém vínculo com fonte e perfil. Uma árvore de arquivos com ícone ROS não representa esse modelo.

Underlays e overlays formam uma cadeia ordenada. `setup` pode carregar os underlays associados; `local_setup` carrega o prefixo local. Sobrescrever um pacote de underlay tem consequências para dependentes. A Vectis deve exibir ordem, origem e sombreamento, e impedir que resultados de contextos diferentes sejam misturados. [Workspaces encadeados, consulta 2026-09-24][overlays]

### 2.3 O ambiente deve ser reproduzível sem se tornar global

O contexto ROS proposto guarda distribuição/prefixo, scripts escolhidos, overlays ordenados, diretório de trabalho, interpretador Python, RMW, domínio, opções de descoberta e referências aos arquivos de configuração. Mantém vínculos separados com kit de compilação e destino de execução. Valores herdados, declarados e medidos precisam aparecer como estados distintos.

O core deve obter esse ambiente em subprocesso controlado, com prazo, saída capturada e identificação dos scripts executados. Depois o aplica a build, LSP, run, debug e ferramentas externas. Não modifica `.bashrc`, não faz `source` em todas as distribuições encontradas e não devolve todo o ambiente ao QML ou aos logs: caminhos e variáveis pertinentes bastam; segredos não integram o resumo.

No diagnóstico, registrar `ROS_DISTRO`, `RMW_IMPLEMENTATION`, `ROS_DOMAIN_ID` e os prefixos pertinentes ao build/runtime. Ausência de `RMW_IMPLEMENTATION` significa seleção implícita a resolver, não middleware ausente. Para descoberta, `ROS_AUTOMATIC_DISCOVERY_RANGE` e `ROS_STATIC_PEERS` são opções documentadas, mas o próprio guia informa que não se aplicam a `rmw_zenoh`. Portanto, “somente localhost” precisa de configuração e prova específicas ao middleware, sem um checkbox que prometa isolamento universal. [Middleware][middleware], [descoberta configurável][dynamicdiscovery], consultas 2026-09-24/25.

Há uma razão concreta para separar descoberta estática de execução: o carregador de launch Python importa o módulo e chama sua descrição; a identificação de pacotes Python pelo colcon pode executar `setup.py`. Portanto, “inspecionar” com essas ferramentas não é sinônimo de apenas ler texto. A abertura de uma pasta pode reconhecer manifests; avaliar scripts depende da confiança/ação já concedida ao workspace. [Carregador de launch][launchpython], [identificação Python do colcon][colconpython], consultas 2026-09-24.

## 3. C/C++/Rust/Python: onde reaproveitar e onde qualificar

| Linguagem | Fluxo ROS documentado | Adaptação proposta para a Vectis |
| --- | --- | --- |
| C++ | `rclcpp`, mensagens geradas e pacotes usualmente `ament_cmake` [Bibliotecas][clients] | clangd sobre compilation database do pacote/contexto correto; CMake executado através de colcon; GDB/LLDB como processo |
| Python | `rclpy` usa bindings nativos; interpretador deve ser compatível com os binários ROS [Python][python] | Reaproveitar inteligência Python e debugpy, preservando prefixos, imports gerados, ABI e ambiente do processo |
| C | `rclc` complementa `rcl`, com uso relevante em micro-ROS [Bibliotecas][clients] | Mesmos serviços C/clangd/build; integração de firmware continua no domínio Embarcados |
| Rust com `rclrs` | README atual anuncia mensagens, pub/sub, serviços, actions, parâmetros e outras APIs; declara ausência de garantia de estabilidade; versão indicada 0.8 [rclrs][rclrs] | Opção prioritária de estudo para projetos Rust ROS; fixar versão e matriz; não repetir avaliações antigas de que actions necessariamente faltam |
| Rust com `r2r` | API assíncrona baseada em futures/streams, geração a partir do ambiente ROS instalado, dependência de libclang; README indica 0.9.7 [r2r][r2r] | Alternativa explícita para projetos existentes; Cargo e rust-analyzer com ambiente e geração corretos; sem migração automática entre bibliotecas |

Fontes da tabela consultadas em **2026-09-24**. As bibliotecas ROS do aplicativo são dependências do projeto do usuário. **Nenhuma delas deve entrar no core da IDE para transformar a Vectis em um cliente ROS embutido.** O fato de o core ser Rust não justifica adicionar `rclrs`, `r2r` ou uma implementação DDS para substituir `ros2` e ferramentas existentes.

Na conferência de licenças, `ros2_rust` apresenta Apache-2.0; o arquivo de `r2r` explicita MIT **AND** Apache-2.0, não uma escolha genérica “MIT OR Apache”. Arquivos e dependências transitivas continuam sujeitos a conferência antes de qualquer distribuição. [Licença rclrs][rclrslicense], [licença r2r][r2rlicense], consultas 2026-09-25 e 2026-09-24, respectivamente.

### 3.1 Consequências concretas para edição e build

1. **Interfaces geradas:** `.msg`, `.srv` e `.action` participam da geração de código. Antes do build correspondente, imports/headers podem faltar; o editor deve distinguir “ainda não gerado” de “biblioteca inexistente”, sem esconder diagnósticos reais. [Bibliotecas cliente, consulta 2026-09-24][clients]
2. **C++ multipacote:** obter `compile_commands.json` com exportação CMake e, quando disponível, o handler do colcon; validar arquivo, diretórios e perfil antes de fornecer ao clangd. A existência do handler foi medida localmente; geração e consumo não foram testados. Não inventar um include path global com todos os overlays.
3. **Python instalado:** `ros2 run` pode executar a cópia instalada, diferente do arquivo aberto em `src`. A documentação sugere `--symlink-install` como uma solução; isso não elimina a necessidade de conferir paths de breakpoints. [Guia de IDEs, consulta 2026-09-24][ides]
4. **Python/venv:** um venv não garante compatibilidade com `rclpy`. A documentação exige compatibilidade do interpretador com os binários e alerta para ambientes Conda incompatíveis. Registrar o Python usado por build, LSP e debug; não resolver imports instalando pacotes automaticamente. [Python, consulta 2026-09-24][python]
5. **Rust:** `rclrs` documenta integração com `colcon-cargo`/`colcon-ros-cargo`; `r2r` também permite fluxo Cargo. Detectar o que o projeto declara, sem impor um deles e sem anunciar toda a matriz ROS porque `cargo build` funcionou em um exemplo. [rclrs][rclrs], [r2r][r2r], consultas 2026-09-24.

### 3.2 Perfis de compilação e testes

Debug/Release deve ser um perfil explícito por contexto. Para pacotes CMake, o core pode propor `CMAKE_BUILD_TYPE`; para Cargo, precisa reconhecer o perfil Cargo correspondente; para Python, não há tradução automática para os mesmos flags. O comando real, cache e artefatos são a evidência. “Debug” não prova ausência de otimização, símbolos completos ou breakpoint verificável.

Proposta de armazenamento: separar bases de build/install/log por distribuição, toolchain e perfil, ou exigir reconfiguração explícita quando o projeto usa diretórios fixos. Não executar limpeza recursiva como correção automática. Registrar argumentos CMake, seleção de pacotes, limites de paralelismo e mixins locais efetivamente usados. A ajuda local mostrou suporte a mixins, mas nenhum mixin disponível para build.

Testar um workspace demanda execução e consolidação: `colcon test` e leitura dos resultados por `colcon test-result`; um processo que terminou não basta para declarar todos os testes aprovados. Pacotes ignorados, dependentes não construídos, testes falhos e resultados antigos precisam de estados separados. Usar artefatos de teste da execução atual, identificados por pacote. [Teste][colcontest], [resultados][colcontestresult], consultas 2026-09-25.

## 4. Ferramentas e plugins abertos: triagem de confiabilidade

“Confiável” aqui significa **candidato com proveniência e evidência examináveis**, e não software certificado, sem vulnerabilidades ou já aprovado no gate da Vectis. Licença, atividade, versão, testes, comportamento de rede e adequação arquitetural são verificações independentes.

**Classe A:** candidato preferencial como processo externo, ainda sujeito a gate funcional. **Classe B:** referência útil ou ferramenta opcional com condições adicionais. **Classe C:** histórico, escopo incompatível ou evidência insuficiente para recomendação de adoção. Essas classes pertencem à triagem deste documento; não alteram os modos A–D do projeto.

### 4.1 Ferramentas da cadeia principal

| Componente | Licença/evidência documental | Uso proposto e ressalva | Classe |
| --- | --- | --- | --- |
| `ros2cli` | Apache-2.0; upstream ROS 2 e CLI local executada [Fonte][ros2cli] | Inventário, consultas e operações explícitas. CLI extensível: medir verbos/flags instalados, sem presumir saída JSON universal | A |
| `colcon-core`, extensões CMake/ROS | Apache-2.0 no core; extensões são componentes separados [Licença][colconlicense], [build][colconbuild] | Build/teste multipacote por processo; não reimplementar scheduler nem executar CMake isolado ignorando a ordem do workspace | A |
| `rosdep` | BSD-3-Clause no arquivo de licença [Licença][rosdeplicense], [comandos][rosdepcommands] | `check`/`resolve` para diagnóstico e explicação de dependências. Não executar `install`, `init` ou `update` automaticamente; cache ausente/desatualizado permanece explícito | A |
| `vcstool` | Apache-2.0; metadados coletados indicam `pushed_at` de 2024-07-17 [Repositório][vcstool] | Reconhecer manifests `.repos`; importação/download não pertence à primeira fatia. Menor atividade exige revisão, mas sozinha não prova abandono | B |
| `rosbag2` | Apache-2.0; README descreve plugins MCAP e SQLite3 [Fonte][bags] | Inspecionar, gravar e reproduzir por CLI; preservar formato, QoS, relógio e resultado. Não embutir reader para substituir a ferramenta | A |
| RViz 2 | Arquivo LICENSE raiz: **BSD-3-Clause-Clear**; manifests podem usar o rótulo genérico BSD [Licença][rvizlicense], [fonte][rviz] | Abrir configuração no `rviz2` externo; visualização espacial e displays no host deles. Não incorporar Ogre/Qt/plugins RViz na UI QML | A |
| rqt + plugins | rqt: BSD-3-Clause; cada plugin exige conferência própria [rqt][rqt], [licença][rqtlicense] | Executar ferramentas de grafo, console, tópicos e parâmetros que existam na distribuição. Não hospedar plugins rqt na Vectis | A |
| Gazebo + `ros_gz` | Apache-2.0 nos repositórios consultados; matriz oficial associa Jazzy/Harmonic e Lyrical/Jetty [Gazebo][gazebo], [ros_gz][rosgz] | Simulação externa opcional, com versão/bridge explícitos; não presumir compatibilidade de qualquer par ROS/Gazebo | B |
| `ros2_tracing` | Apache-2.0; usa LTTng e documenta suporte Linux [Fonte][tracing] | Profiling por ferramenta existente; verificar instrumentação e permissões. A primeira fatia pode somente abrir resultado externo | B |

Fontes e licenças desta tabela consultadas em **2026-09-24**, exceto onde indicado no registro final. Licença da raiz não substitui auditoria de arquivos, binários distribuídos e dependências transitivas.

### 4.2 Plugins de IDE e aplicações: o que a pesquisa encontrou

| Projeto | Evidência verificada | Avaliação para a Vectis |
| --- | --- | --- |
| **Robot Developer Extensions for ROS 2 — RDE** | MIT; sucessor indicado pelo projeto após a extensão Microsoft; manifest 1.3.0; código de debug e testes examinados [Fonte][rde] | **B / MODE-B. Principal referência de comportamento:** ambiente, tarefas colcon, launch, seleção de processos, interfaces e testes. Não carregar VSIX/Extension Host |
| **ROS Qt Creator Plug-in** | Arquivos `ros_colcon_step.cpp` e `ros_run_step.cpp` têm cabeçalhos Apache-2.0; API GitHub não reconheceu licença global. Há implementação colcon, apesar de README enfatizar catkin/ROS 1 [Fonte][qtc], [colcon][qtccolcon] | **B / MODE-B.** Referência útil de build/kit/ambiente. Não classificá-lo como apenas ROS 1 nem prometer cobertura ROS 2 completa. Licença global e cobertura de execução ROS 2 permanecem pendentes |
| **Microsoft `ms-iot/vscode-ros`** | MIT; repositório marcado como arquivado na API [Fonte][oldros] | **C / MODE-D.** Histórico; não escolher como base de manutenção quando existe sucessor |
| **`rqt_graph`** | BSD-3-Clause em LICENSE; ROS Index diferencia releases por distribuição [Fonte][rqtgraph], [índice][rqtgraphindex] | **A / MODE-A externo; B como referência.** Grafo observado, filtros e seleção. Não transportar plugin Python/Qt para dentro do QML |
| **PlotJuggler** | MPL-2.0 no aplicativo atual [Fonte][plot] | **B / MODE-A externo.** Ferramenta candidata para sinais e séries temporais; compatibilidade da versão instalada precisa ser medida |
| **`plotjuggler-ros-plugins`** | LICENSE AGPL-3.0; manifest 2.3.1 declara `AGPLv3`; workflow Jazzy usa `action-ros-ci` [Licença][plotpluginslicense], [manifest][plotpluginspackage], [CI][plotpluginsci] | **B / externo, sujeito à política de licença.** Não herdar a licença MPL do aplicativo; não copiar nem linkar esses plugins ao core. Empacotamento e combinação com PlotJuggler 4.x não foram validados |
| **Lichtblick** | LICENSE MPL-2.0; aplicação de diagnóstico desktop/web; README descreve telemetria condicionada ao endpoint OTLP compilado [Fonte][licht], [licença][lichtlicense] | **B / MODE-A externo.** Candidato aberto para investigação visual; começar por arquivos locais. Verificar o build entregue e a rede antes de qualquer alegação “sem telemetria” |
| **Foxglove Studio histórico** | Repositório público `foxglove/studio` arquivado [Fonte][oldfoxglove] | **C / MODE-D.** O arquivo histórico não comprova que o produto comercial atual inteiro tenha a mesma licença ou esteja aberto; não o recomendar como tal |
| **Foxglove Bridge ROS 2 atual** | Fica em `foxglove/foxglove-sdk`, MIT. O antigo `ros-foxglove-bridge` agora aponta para ele e se descreve como ROS 1 [Bridge atual][foxbridge], [legado][oldbridge] | **B / processo opcional.** Usar somente quando já disponível no contexto escolhido. Não instalar bridge no robô; não adicionar o SDK ao core |
| **`rosbridge_suite`** | BSD-3-Clause segundo licença/metadados; interface JSON/WebSocket; declaração própria Quality Level 3 [Fonte][rosbridge] | **B / processo opcional.** Não é pré-requisito para a Vectis usar ROS 2. Também requer servidor ROS existente; sua presença não garante desempenho para câmera/LiDAR |
| **FKIE Multi Agent Suite** | MIT; README exige daemon em cada host e associa discovery ROS 2 a `rmw_fastrtps_cpp` [Fonte][fkie] | **B como referência; C para adoção de sua arquitetura.** UX distribuída é relevante; instalar daemon por host conflita com a premissa da Vectis |

Fontes consultadas em **2026-09-24**. Nenhum desses plugins foi instalado ou executado nesta pesquisa. “CI presente” significa workflow/teste encontrado; não significa execução verde conferida ou teste reproduzido pela Vectis.

### 4.3 Referências profissionais examinadas além do README

**RDE:** foram lidos o resolver de attach, o resolver de launch ROS 2, `telemetry-helper.ts`, manifest e testes de fronteira de workspace/conversão de caminhos. O attach delega a adaptadores do host, incluindo `cppdbg` e alternativa LLDB. Há dependência de telemetria e código que cria o reporter quando encontra `aiKey`; não houve medição de transmissão. Os testes examinados cobrem, por exemplo, caminhos com prefixo semelhante e descoberta dentro da árvore do workspace. [Código fixado][rdeattach], [telemetria][rdetelemetry], [testes][rdetests], consulta 2026-09-24.

**Lição para a Vectis:** manter caminhos e ambientes por contexto; distinguir processo de nó; reaproveitar seus próprios GDB/LLDB/debugpy. Não transportar dependências do VS Code, instalação de extensões, telemetria ou recursos MCP. A licença MIT da extensão não comprova a licença de cada adaptador/binário ao qual ela delega.

**Qt Creator:** o passo colcon usa ambiente do workspace, diretório do projeto, parsers de saída e toolchain do kit; o código fixa locale para parsing e deriva progresso de texto. Seu README informa dependência da versão da API do Qt Creator. [Implementação][qtccolcon], [README][qtc], consulta 2026-09-24.

**Lição para a Vectis:** conservar proveniência de kit/ambiente e saída bruta; parsers precisam de testes por versão e estados de falha. A afinidade “ambos usam Qt” não torna o plugin carregável em Qt/QML nem justifica importar sua arquitetura.

### 4.4 Revisões fixadas e manutenção observada

Pins consultados em **2026-09-24**; a data abaixo é a do commit retornado pela API, não uma promessa de compatibilidade. São pins de pesquisa, não versões aprovadas para distribuição.

| Projeto / branch | Revisão examinada | Data UTC do commit |
| --- | --- | --- |
| RDE / `main` | [4928feba8734707b5841a1f5db83b61ff3661a16][rdepin] | 2026-09-04 |
| Qt Creator ROS / `devel` | [3c4dcb2718f47ff71e484e9845fb6f36d547726e][qtcpin] | 2026-09-24 |
| PlotJuggler / `main-4.x` | [c3077496c37c85f0457eda8f4404c3dc3a42be3a][plotpin] | 2026-09-23 |
| Plugins ROS PlotJuggler / `main` | [9cb0b596bc94bde4fa4e9f3a462dcbe78d5a2377][plotpluginspin] | 2026-04-21 |
| Lichtblick / `develop` | [c72097a9c58129e4cbb84b898ed3d220ae6048a0][lichtpin] | 2026-09-24 |
| `ros2_rust` / `main` | [c36e7a3040c3a2e299521591747b23b7e8b62a18][rclrspin] | 2026-09-23 |
| `r2r` / `master` | [653167aadcc32b20cb13b6542460ee1926ed933c][r2rpin] | 2026-09-13 |
| Foxglove SDK / `main` | [dcbc66776e8704f5b4f9aa0c6e3ef695ed4c297b][foxpin] | 2026-09-24 |

A triagem também coletou `pushed_at` dos repositórios: `ros2cli`, `rosbag2`, RViz e tracing em 2026-09-23; colcon em 2026-09-17; rosdep em 2026-09-11; FKIE em 2026-09-20; `micro_ros_setup` em 2026-09-18; micro-ROS-Agent em 2026-09-10. Esse campo indica atividade de push, inclusive em outras branches; **não deve ser apresentado como último commit da versão selecionada**. Fonte: API pública dos respectivos repositórios, consulta 2026-09-24.

Antes de promover um candidato, faltam auditoria transitiva, advisories e issues pertinentes, teste do binário selecionado, comportamento de rede, cancelamento e ausência de órfãos. Esta pesquisa não afirma que esses itens passaram. O [registro de componentes](registro-de-componentes-abertos.json) permanece sem adoções ROS decorrentes deste documento.

## 5. Fluxo de execução e depuração

### 5.1 Executar um launch não equivale a depurar seus nós

O sistema launch descreve processos, argumentos e ações de execução. O formato Python pode construir essa descrição dinamicamente. `--show-args` ajuda a descobrir argumentos, mas carregar o arquivo pode executar Python; não é uma análise estática inofensiva. [Launch][launch], [carregador Python][launchpython], consultas 2026-09-24.

Fluxo de produto proposto:

1. Escolher contexto ROS e perfil; mostrar quais partes foram verificadas.
2. Selecionar pacote/executável ou launch, com origem em `src`/`install` visível.
3. Informar argumentos, remaps e arquivos de parâmetros como campos de intenção.
4. O core resolve esses dados e apresenta comando, diretório, contexto e pendências.
5. Executar pela infraestrutura comum de run/terminal; identificar processo iniciado, saída e encerramento.
6. Depurar um processo selecionado com adaptador e fontes comprovados; somente depois estudar sessões múltiplas.

O usuário pode manter scripts próprios nas configurações de execução existentes. Isso oferece uma saída operacional, mas não autoriza rotular um comando shell salvo como integração ROS 2 completa.

### 5.2 Nó, processo e sessão DAP

ROS 2 permite vários nós em um processo por composição. Um componente pode ser uma biblioteca carregada pelo container; o nome de nó no grafo não identifica necessariamente um PID ou executável independente. [Composição, consulta 2026-09-24][composition]

Consequências propostas:

| Situação | Comportamento exigido |
| --- | --- |
| Nó C++ executado individualmente | Resolver ELF, argumentos e ambiente; usar GDB/LLDB externo; breakpoint precisa ser confirmado pelo adaptador |
| Nó Python | Resolver interpretador e código efetivamente instalado; usar debugpy já disponível no contexto; não depurar somente o launcher Python |
| Vários componentes no mesmo container ROS | Selecionar processo/container, mostrar associação conhecida; parar no debugger pode suspender outros componentes daquele processo |
| Launch com vários processos | Preservar o launch como grupo de execução; não alegar multi-debug enquanto o serviço DAP continuar com uma sessão |
| Processo que a IDE iniciou | Parada/cancelamento devem acompanhar seus descendentes e produzir resultado observável |
| Processo existente ao qual a IDE fez attach | Desanexar preserva propriedade externa; não usar “parar tudo” para matar o robô |
| PID não identificável a partir da observação | Exibir “processo não associado”; não inferir PID do nome do nó |

A leitura local encontrou `session: Option<DapSession>` no gerenciador DAP e `debug.start` sem parâmetros públicos para PID, argumentos ou contexto ROS. `connect` identifica endpoint debugpy; não é um conector genérico ROS ou GDB. O attach GDB remoto já existente é governado pelo kit. [Gerenciador local](../../crates/kinein-core/src/dap/mod.rs), [contrato](../../crates/kinein-protocol/src/debug.rs), medição 2026-09-24.

**Critério da primeira fatia:** um nó/processo depurável com ambiente e fontes corretos. Attach por PID e múltiplas sessões exigem evolução real do DAP; não basta criar uma lista de nós na UI.

### 5.3 Logs e cancelamento

Proposta: preservar stdout/stderr por processo conhecido e agregar por sessão de execução, com horário e limites de retenção. Logs ROS publicados em `/rosout` são outra fonte: recebê-los exige observação ROS e pode depender de configuração/QoS. Console do processo, log da aplicação e trace de callbacks não devem ser rotulados como uma única evidência.

O contrato de cancelamento deve registrar solicitação, término observado e descendentes restantes. Timeout é resultado incompleto, não confirmação de parada. Não há prova nesta pesquisa de que o `run.stop` atual encerre corretamente toda árvore de um launch; esse é um teste obrigatório antes da promoção.

## 6. Observabilidade que ajuda a investigar um sistema

### 6.1 Grafo com contexto, validade e limitações

A CLI ROS usa um daemon de descoberta para acelerar consultas; ele pode ser iniciado automaticamente por comandos como listagem de nós/tópicos e opera por domínio. Não é o `roscore` do ROS 1. A ajuda local confirmou alternativa `--no-daemon` para `node list`. [CLI e daemon, consulta 2026-09-24][daemon]

Proposta inicial: snapshots solicitados pelo usuário, com prazo limitado, através de comandos existentes. Cada resultado inclui contexto, instante, duração de observação, ferramenta/versão, saída bruta e estado completo/parcial. “Nenhum nó observado neste intervalo” é uma resposta válida; “não existe nenhum nó” extrapola a medida.

O grafo deve separar nós, tópicos, publishers/subscribers, serviços e actions. Recursos que o comando selecionado não consegue informar permanecem desconhecidos. A IDE não mantém um daemon próprio no alvo nem assume posse de um daemon ROS iniciado pelo usuário. Alterar domínio/RMW invalida o snapshot; não se reinicia indiscriminadamente um daemon compartilhado.

### 6.2 QoS, frequência e mensagens

A documentação DDS/ROS exemplifica uma incompatibilidade decisiva: publisher `best_effort` não satisfaz subscriber `reliable`; o sentido inverso é compatível. Durabilidade e outras políticas também participam da compatibilidade. Um tópico pode aparecer no grafo e o observador não receber mensagens. [QoS, consulta 2026-09-24][qos]

Para a UI, isso implica mostrar perfil pedido pelo observador e, quando medido, oferecido pelos endpoints. Não alterar o publisher, o middleware ou as políticas do robô para fazer o monitor “funcionar”. A regra de diagnóstico precisa ser específica ao RMW: a documentação atual distingue Zenoh das implementações DDS. [Middleware, consulta 2026-09-24][middleware]

Uma operação `echo` é uma subscrição real. Proposta: selecionar tópico e campos, limitar taxa/bytes/tempo, permitir parar e informar truncamento/descarte. Imagens e nuvens de pontos devem abrir visualizadores especializados; não serializar todo esse tráfego no JSON-RPC QML/core. Frequência recebida pelo observador não comprova frequência de produção nem latência de ponta a ponta.

### 6.3 Parâmetros, serviços, actions e lifecycle

Nós com lifecycle têm estados e transições próprios; não se pode pressupor que todos os nós sejam gerenciados dessa forma. [Lifecycle, consulta 2026-09-25][lifecycle]

Proposta de evolução: começar por leitura de parâmetros e estado. Alterar parâmetro, chamar serviço, enviar/cancelar action ou pedir transição são operações explícitas, com destino, payload e resposta observáveis. Falha/timeout deve permanecer falha/timeout. Não converter automaticamente um timeout em repetição de comando que possa movimentar equipamento.

Publicação em tópicos de comando e controle de atuadores não entram como botões genéricos da primeira fatia. O foco é desenvolver e investigar o projeto, preservando seus próprios launches e interfaces de operação.

### 6.4 Bags, tempo e profiling

`rosbag2` documenta gravação/reprodução, overrides de QoS e armazenamento MCAP/SQLite3; no README Jazzy consultado, MCAP é o padrão. Com `--use-sim-time`, a gravação espera a primeira mensagem `/clock`. Não presumir essas opções ou plugins em toda instalação. [rosbag2, consulta 2026-09-24][bags]

Proposta para bags: exibir diretório/arquivo, formato reconhecido, tópicos/tipos, duração e contagens que a ferramenta informou. Para gravar, indicar tópicos, destino e contexto. Para reproduzir, apresentar domínio, relógio, remaps e efeito de publicação antes da execução. Abrir um arquivo para inspeção não inicia replay.

`ros2_tracing` observa eventos instrumentados e usa LTTng no Linux; o projeto distingue tracer userspace de tracing de kernel e recomenda iniciar a sessão antes da aplicação para coletar metadados necessários. [Tracing, consulta 2026-09-24][tracing]

Proposta: tratar trace como capacidade separada de DAP e de bag. Verificar instrumentação, ferramenta e resultado existentes. Não anunciar análise de callbacks, ausência de perda ou overhead aceitável sem captura e medição no alvo pertinente. Um perfilador genérico de CPU continua útil, mas não substitui sozinho a semântica de executores/callbacks ROS.

### 6.5 TF, URDF e Xacro

O grafo de comunicação e a árvore de coordenadas são observações diferentes. O conjunto `geometry2` fornece `tf2_echo`, `tf2_monitor` e `view_frames` para inspecionar transformações e estatísticas temporais. `view_frames` produz um artefato gráfico; o código consultado chama `dot` para gerar PDF. [Ferramentas TF][tftools], [implementação][tfframes], consulta 2026-09-25.

Proposta: ações de paleta para consultar transformação entre frames e gerar/abrir o relatório, sempre no contexto escolhido. Falta de frame, janela temporal insuficiente e ausência de `dot` devem permanecer diagnósticos distintos. Não criar buffer TF ou renderer 3D dentro da Vectis. A aprovação de um snapshot de tópicos não deve marcar a árvore TF como validada.

Xacro expande macros XML; sua fonte é mantida no repositório `ros/xacro`. [Fonte Xacro, consulta 2026-09-25][xacro] Para a Vectis, a proposta é editar URDF/Xacro como arquivos do projeto, executar o processador externo já instalado quando solicitado e visualizar o resultado em ferramenta existente. Editor visual de robô, resolução automática de meshes e correção de geometria não estão incluídos. Xacro/TF foram examinados como fluxo; não passaram pela triagem completa de licença, transitivas e execução exigida para adoção.

## 7. Remoto, Linux embarcado e micro-ROS

### 7.1 SSH não transporta automaticamente o grafo ROS

O caminho preferido para uma máquina remota é executar as ferramentas ROS **que já existem nela**, através do SSH existente, com seu ambiente identificado. Saída e resultados voltam pelos canais de execução da IDE. A comunicação DDS distribuída depende da rede e de sua descoberta; um encaminhamento TCP SSH não equivale a transportar multicast e todos os endpoints DDS. Essa é uma inferência de integração a partir do modelo de descoberta, não um teste de rede realizado aqui. [Descoberta, consulta 2026-09-24][discovery]

Alternativamente, uma bridge já disponível pode expor uma porta encaminhável. A bridge ROS 2 atual da Foxglove documenta porta 8765, bind padrão `0.0.0.0`, TLS desabilitado por padrão, capacidades de publicação/parâmetros/serviços e acesso remoto por plataforma opcional. Ser MIT não significa ser somente uma leitura local. [Bridge, consulta 2026-09-24][foxbridge]

**Decisão proposta:** bridges são opcionais e externas. Mostrar endereço e capacidades efetivas; preferir escopo local quando o usuário escolhe executar uma para uso local. Não habilitar acesso cloud, instalar servidor no alvo ou abrir portas como reparo silencioso. Se não houver ROS/bridge no destino, registrar a ausência e manter a capacidade indisponível.

Cross-compilation Linux exige ABI, sysroot, bibliotecas ROS, tipos gerados e instalação coerentes com o alvo. O kit de microcontrolador `arm-none-eabi` não representa um computador Linux ARM. Reaproveitar kits e SSH, mas testar deploy de um pacote/workspace ROS antes de prometer que `remote.deploy` cobre dependências de execução.

### 7.2 micro-ROS é um projeto de firmware, não uma descoberta automática

Na arquitetura usual micro-ROS, o cliente no dispositivo restrito usa Micro XRCE-DDS e se comunica com um Agent, que o representa no espaço DDS. A documentação descreve transportes como serial e UDP. [Arquitetura micro-ROS, consulta 2026-09-24][micromiddleware]

Isso cria três identidades que a Vectis precisa manter separadas:

| Identidade | Onde vive | O que a IDE pode medir |
| --- | --- | --- |
| Placa/probe e firmware | MCU e conexão de gravação/debug | Identificação disponível, artefato, operação de flash e resultado |
| Transporte e Agent | Host local ou computador Linux do sistema | Executável existente, versão, serial/endereço, processo e logs |
| Entidades ROS expostas | Grafo observado no contexto ROS | Nós/tópicos/tipos e troca de mensagens efetivamente observada |

O Agent pertence à aplicação do usuário; não é um agente da IDE a ser instalado. Nada neste documento autoriza provisionamento do alvo. Trabalhar com firmware micro-ROS pressupõe projeto e ferramentas já disponibilizados pelo usuário; gravar esse firmware continua sendo uma operação explícita do fluxo Embarcados.

`micro_ros_setup` apresenta ferramentas de build e encaminha a integrações específicas, como ESP-IDF e Zephyr. Parte dos exemplos STM32 depende de CubeMX/CubeIDE; a própria tabela contém versões antigas e itens comunitários. Logo, um nome de placa nessa tabela não comprova uma rota atual livre da cadeia ST. [micro_ros_setup, consulta 2026-09-24][microsetup]

**Adaptação recomendada:** estudar primeiro projeto micro-ROS já existente com ESP-IDF ou Zephyr; para STM32/GNU Arm/OpenOCD, exigir BSP/startup/linker/RTOS e biblioteca compatíveis fornecidos pelo projeto. Não gerar esse conjunto por dedução do MCU e não introduzir CubeMX como dependência oculta.

O fluxo ESP32/MicroPython existente não equivale a micro-ROS: `rclpy` depende do ambiente Python/bindings descrito acima; não foi verificado suporte a ele no MicroPython da placa. mpremote, esptool, serial e o futuro OpenOCD continuam reutilizáveis em seus papéis, sem anunciar que já fornecem transporte XRCE ou firmware ROS.

### 7.3 Simulação e frameworks de robótica

Gazebo/`ros_gz` oferecem uma frente de simulação externa. A matriz oficial consultada associa Jazzy a Harmonic e Lyrical a Jetty; usar um par diferente exige verificação específica. [Matriz ros_gz, consulta 2026-09-24][rosgz]

Nav2, MoveIt 2 e ros2_control merecem aparecer como **projetos e ferramentas do usuário**, não como três novas plataformas internas da IDE. As fontes oficiais identificam Nav2 como stack de navegação, MoveIt 2 como framework de planejamento de movimento e ros2_control como framework de controle. [Nav2][nav2], [MoveIt 2][moveit], [ros2_control][control], consultas 2026-09-24/25.

Proposta: abrir seus pacotes/launches, compilar, depurar processos e lançar RViz/Gazebo existentes. Assistentes de calibração, planejadores próprios, editor de comportamento, configuração automática de hardware e operação de frota ficam fora da primeira fatia. Compatibilidade e licença de cada plugin/driver continuam específicas ao pacote; nenhuma dessas stacks foi executada nesta pesquisa.

## 8. O que trazer / o que NÃO trazer

A tabela segue o idioma de comparação usado em [Remote-SSH UI/HUD, §3.1](../especificacoes/remote-ssh-ui-hud.md). São decisões propostas, derivadas das evidências das seções anteriores.

| Item de referência | O que trazer | O que NÃO trazer | Motivo |
| --- | --- | --- | --- |
| Ambiente ROS dos plugins de IDE | Contexto reproduzível e visível para todas as ferramentas | Source global, troca silenciosa de distro, exigência de iniciar a IDE de terminal preparado | A medição local já mostrou diferença entre instalado e visível no PATH |
| Workspaces colcon | Pacotes, dependências, seleção e bases de build/install | Reimplementação de colcon/ament ou tratar todo workspace como um CMake único | A unidade de orquestração é multipacote |
| Perfis | Configuração real, argumentos e artefatos separados | Botão “Debug” que promete símbolos sem medir | Linguagens e pacotes têm mecanismos diferentes |
| RDE ROS 2 | Fluxo de launch, configuração, interfaces e seleção de processo | Extension Host, VSIX, telemetria, instalação automática de adaptadores, MCP | A Vectis já tem serviços próprios e ferramentas externas |
| Qt Creator ROS | Relação entre ambiente, kit, processo e diagnóstico | Plugin binário ou ABI do Qt Creator | Qt em comum não cria compatibilidade de host |
| Introspecção ROS | Snapshot datado, QoS observado e saída bruta | Grafo apresentado como verdade instantânea/global | Descoberta é temporal e depende do contexto |
| Run/debug | Continuidade de ambiente, fontes e processo selecionado | “Depurar todos os nós” sem multi-DAP e associação de processos | Nós e processos não têm relação um-para-um |
| rqt/RViz | Ação de abrir ferramenta instalada e configuração | Seus plugins rodando dentro da UI Qt/QML | Evita duplicar host e acoplar versões Qt/ROS |
| PlotJuggler | Análise externa de sinais e bags | Plugins AGPL incorporados ao core | Licença e arquitetura distintas; combinação de versões precisa de prova |
| Lichtblick | Opção externa de visualização com build identificado | Electron/Node embutido, alegação não medida de ausência de rede | O aplicativo tem ciclo e dependências próprios |
| rosbag2 | Inspeção/gravação/replay com domínio, tempo e QoS explícitos | Reader/player reimplementado ou replay ao abrir arquivo | Reaproveita ferramenta madura e torna o efeito visível |
| Tracing | Sessão e artefatos de LTTng/ros2_tracing | Profiler próprio de executores ou promessa de tempo real | Instrumentação e impacto precisam ser medidos |
| SSH | Comandos no contexto remoto já existente | Instalar daemon Vectis, FKIE, ROS ou bridge no alvo | Respeita a premissa de medir e informar |
| micro-ROS | Vínculo firmware → transporte → Agent → grafo | Confundir MicroPython com micro-ROS ou instalar firmware para “detectar” | Há mudança real de aplicação e dependências |
| Gazebo | Processo externo, versão pareada e configuração do projeto | Simulador dentro da IDE ou prova simulada vendida como prova física | Simulação não verifica USB, rádio, timing elétrico ou atuação real |
| Rust ROS | Detecção de biblioteca/versão e pipeline de mensagens | Inserir cliente ROS no core ou garantir paridade automática com C++ | A estabilidade e a cobertura precisam de qualificação própria |

## 9. Superfícies na IDE

**Proposta de organização:** ROS 2 ganha superfície contextual própria para workspace Linux/robótica. O painel Embarcados mantém Placa · Projeto · Gravar · Kit, com vínculos para a aplicação ROS quando houver firmware micro-ROS. Layout e atalhos definitivos pertencem ao trabalho futuro de produto; não são alterações da 0.3.

```text
Barra de execução
  contexto ROS · perfil · configuração ativa
  Construir | Executar | Depurar processo | Parar execução

Área de edição
  C/C++/Rust/Python · package.xml · interfaces · launch · parâmetros

Painel ROS 2
  Ambiente | Pacotes | Execução | Grafo
  Detalhes: origem, status medido, argumentos, QoS, pendências

Superfícies comuns
  Problems · Tests · Jobs · Terminal/Console · Debug
  Bags e Trace: artefatos com ações; visões dedicadas só após os gates

Embarcados                         Remoto
  Placa · Projeto · Gravar · Kit     alvo SSH · probe · deploy · execução
          vínculo ao projeto micro-ROS e ao contexto ROS

Ferramentas externas
  RViz 2 · rqt · PlotJuggler · Lichtblick · Gazebo
```

| Superfície | Conteúdo e ações |
| --- | --- |
| **ROS 2 / Ambiente** | Prefixo/distribuição, overlays ordenados, Python, RMW/domínio, ferramentas detectadas, evidência e última inspeção; “Verificar ambiente” |
| **ROS 2 / Pacotes** | Pacotes da fonte e instalados, build type, dependências, interfaces e estado; seleção para build/teste; abrir manifesto |
| **ROS 2 / Execução** | Pacote/executável ou launch, argumentos e contexto; proposta do core; processos associados e estado de encerramento |
| **ROS 2 / Grafo** | Atualizar snapshot, filtrar namespace, detalhes de endpoints e QoS; consultas limitadas; estado parcial visível |
| **Ações primárias** | Construir seleção, Executar configuração, Depurar processo, Parar execução própria; mesma semântica das ações globais |
| **Paleta** | Verificar ambiente/dependências; listar pacotes; escolher launch; atualizar grafo; inspecionar tópico/bag; abrir ferramenta externa disponível |
| **Kit** | Compiladores/debugger, arquitetura, sysroot/toolchain file e alvo de debug. Referência ao contexto quando necessário; não usar `remoteTarget` como domínio ROS |
| **Contexto ROS** | Prefixos, scripts, Python, RMW/domínio/descoberta e destino de execução. Não guardar credenciais nem todo ambiente herdado |
| **Configuração de execução** | Intenção tipada, parâmetros, remaps, launch e referência ao contexto; comando renderizado pelo core |
| **Embarcados / Projeto e Kit** | Identificação do projeto micro-ROS e vínculo ao contexto; firmware/toolchain/Agent como capacidades separadas |
| **Embarcados / Gravar** | Artefato e operação de flash já previstos no domínio; não instalar Agent nem afirmar comunicação ROS após apenas gravar |

Nenhum botão deve pedir à UI para formar `source … && ros2 …`. QML envia escolhas; o core valida, compõe e apresenta a operação.

## 10. IPC: reaproveitamento e lacunas medidas

Os nomes novos abaixo são **propostas de contrato, não endpoints disponíveis**. A leitura de core/protocolo em 2026-09-24 não encontrou integração ROS/colcon/ament; os resultados ROS na UI eram referências de iconografia. Os pontos existentes permitem operar comandos manuais, mas não representam o contexto ROS nem um grafo multipacote. O estado `available` de uma proposta significa requisitos verificados para aquela operação; não homologa todo o ecossistema nem substitui o gate funcional.

### 10.1 O que já existe e pode ser reaproveitado

| Contrato/serviço atual | Parâmetros → resultado existente | Reuso e limite para ROS 2 |
| --- | --- | --- |
| `project.model` | Sem parâmetros → modelo, frameworks, SDKs, artefatos, alvo e hints | Base de descoberta; não contém o modelo de workspace ROS proposto [Fonte](../../crates/kinein-core/src/project/mod.rs) |
| `build.run` | `buildSystem?` → `jobId` | Jobs/diagnósticos reaproveitáveis; enum de build não contém colcon e request não seleciona pacotes [Build](../../crates/kinein-protocol/src/build.rs), [sistemas](../../crates/kinein-protocol/src/workspace.rs) |
| `test.run` | `filter?`, `buildSystem?` → `jobId` | Reaproveitar Tests/Jobs; seleção colcon e consolidação por pacote ainda não representadas [Handler](../../crates/kinein-core/src/handlers/build.rs) |
| `runConfig.save` / `setActive` | `id?`, `name`, `command` / `id?` → configurações e ativa | Um script/comando ROS escrito pelo usuário pode ser salvo; não há intenção ROS nem contexto tipado [Contrato](../../crates/kinein-protocol/src/runconfig.rs) |
| `run.start` / `run.stop` | `command?`, `device?` → `command`, `terminalId?` / parada | Reuso de execução interativa; saída por `event.terminal.render`, término por `event.terminal.closed`. Não é um Job ROS estruturado [Contrato](../../crates/kinein-protocol/src/run.rs) |
| `debug.start` e demais `debug.*` | `program?` ou `connect {host,port}` → `program`, `attached` | Reaproveitar DAP; `connect` é debugpy. Falta contexto/args/PID/múltiplas sessões [Contrato](../../crates/kinein-protocol/src/debug.rs) |
| `toolchain.get` / `toolchain.setKit` | Seleção/atributos de kit, incluindo sysroot, alvo e configuração de debug | Reutilizar toolchain e alvo de debug; não sobrecarregar `remoteTarget`/`debugServer` com configuração ROS [Contrato](../../crates/kinein-protocol/src/toolchain.rs) |
| `remote.probe` / `remote.deploy` | `name` / `name`, `source?`, `dest?` → `jobId`, `command` | SSH, diagnóstico e transferência; não medem por si só o ambiente ROS remoto nem instalam dependências [Contrato](../../crates/kinein-protocol/src/remote.rs) |
| `remote.command` | `name`, `kind`, `program?`, `port?` → `command`, `remoteTarget?`, `source` | Exemplo existente de composição no core; falta representação do ambiente/launch ROS [Handler](../../crates/kinein-core/src/handlers/remote_command.rs) |
| `container.*` | Estado/listagem, ações, abertura de shell/logs e Compose | Motores existentes como processos; não há prova de ROS, descoberta, GPU ou debug em container [Handler](../../crates/kinein-core/src/handlers/container.rs) |
| Serviços de LSP, Jobs, Problems, Tests e terminal | Infraestrutura comum já presente | Reaproveitar serviços; alimentar contexto e resultados ROS exige adapter e contratos, não outro host de extensões |

Não se propõem `ros2.build`, `ros2.test`, `ros2.flash` ou `ros2.debug` paralelos aos domínios existentes. Reusar uma infraestrutura não equivale a afirmar que seu contrato atual já contém os campos necessários.

### 10.2 Contratos novos candidatos

| Proposta | Parâmetros de intenção | Resultado proposto | Justificativa medida |
| --- | --- | --- | --- |
| `ros2.environmentInspect` | Destino de execução; prefixo/scripts escolhidos; overlays ordenados; RMW/domínio/Python opcionais; modo `static` ou `evaluate` | `jobId`; evento `event.ros2.environmentInspected` com `contextId`, fatos declarados/medidos, ferramentas, versões, capacidades por operação, pendências, fontes e erro | PATH inicial não encontrou `ros2`, mas o subprocesso com setup executou a CLI. O kit atual não representa essa cadeia nem seu ambiente |
| `ros2.workspaceInspect` | `contextId`; raízes dentro do workspace; modo `static` ou `colcon` | `jobId`; evento `event.ros2.workspaceInspected` com `snapshotId`, pacotes, caminhos, build types, dependências conhecidas, origens/sombreamento, incertezas e saída bruta referenciada | Modelo atual não possui colcon nem identidade de pacotes ROS; descoberta via colcon pode executar código, portanto seu modo precisa ser explícito |
| `runConfig.ros2Proposal` | `contextId`; operação e seleção tipadas: executar pacote, launch, consulta CLI ou bag; argumentos/remaps/arquivos de parâmetros pertinentes | `proposalId`, nome, comando renderizado, diretório, contexto, requisitos, evidências, efeitos e estado `available`/`partial`/`blocked`/`unverified` | `runConfig.save` só recebe shell. O precedente `runConfig.flashProposal` já fixa composição no core; falta uma proposta ROS equivalente |
| `ros2.graphSnapshot` | `contextId`; categorias solicitadas; filtro de namespace; prazo e limite de entidades | `jobId`; evento `event.ros2.graphObserved` com instante/intervalo, entidades, relações conhecidas, estado parcial, truncamento, erros por consulta e referências à saída | Nenhum contrato existente representa grafo/QoS; CLI de introspecção tem descoberta temporal e pode usar daemon |

Esses quatro candidatos cobrem as lacunas identificadas sem criar uma família de métodos para cada comando da CLI. No início, diagnóstico de dependências, detalhes de tópico e informações de bag podem ser propostas de execução com resultado textual. **Não prometer tabelas normalizadas desses resultados antes de acrescentar e testar seu schema.**

A escolha do contexto também precisa alcançar o editor. Propõe-se o contrato comum **novo** `workspace.selectContext`, com `contextId` → contexto ativo, serviços reconfigurados e pendências. A justificativa é a mesma diferença de ambiente medida na §1.1: selecionar ambiente apenas em run deixaria LSP/build divergentes. O core verifica vínculo ao workspace e reconfigura os serviços futuros; processos em execução preservam o contexto com que começaram. O nome e a persistência devem ser reconciliados com o desenho de RemoteContext antes de implementar, sem criar seletores concorrentes.

`static` significa leitura de arquivos sem avaliação de scripts. `evaluate`/`colcon` podem executar ferramentas e código do ambiente/projeto; obedecem à confiança e autorização já existentes para o workspace. Uma verificação não pode transformar a execução de `setup.py` ou launch Python em efeito oculto.

### 10.3 Extensões necessárias nos contratos comuns

| Contrato | Extensão proposta | Resultado/condição |
| --- | --- | --- |
| `build.run` | Acrescentar sistema colcon, `contextId`, perfil e seleção `{mode: all/selected/upTo, packages}` | Preservar `jobId`; emitir progresso/diagnóstico por pacote somente quando medido; desconhecidos e falhas de parsing não viram sucesso |
| `test.run` | Mesma seleção/contexto e vínculo à execução de build pertinente | Preservar Jobs/Tests; consolidar resultados atuais e identificar falha de teste mesmo quando o comando não a reflete no exit code |
| `runConfig.save` | Variante de configuração criada de `proposalId`, mutuamente exclusiva com shell manual | Core persiste intenção e referência de contexto; revalida ao executar. Não depender de um identificador efêmero após reinício |
| `run.start` | Executar proposta/configuração ROS validada, com contexto do core | Continuar no terminal/serviço comum; resultado identifica execução. O comando exibido não volta pela UI como fonte de verdade alterável |
| `debug.start` | Contexto, argumentos e diretório para o processo escolhido; seleção de alvo sem ambiguidade com `program`/`connect` | Reusar resultado/eventos DAP; capacidade depende do adaptador. Attach PID é evolução posterior com alvo discriminado, sem reutilizar `connect` |
| Serviço de execução remota | Aplicar contexto de destino também a build/run/debug | SSH não reutiliza paths locais como se fossem remotos; falha de setup é resultado visível |

Adicionar `sessionId` a toda a família DAP e suportar múltiplas sessões seria mudança transversal. **Fica fora da primeira fatia**; a limitação de sessão única foi medida e é suficiente para justificar estudo posterior, não implementação antecipada.

### 10.4 Regras comuns de evidência, erro e volume

Cada contexto/snapshot/proposta pertence ao workspace e destino que o produziram. Mudança de setup, overlay, distribuição, RMW, perfil ou arquivos relevantes invalida os resultados dependentes. O core rejeita referência vencida e explica o que precisa ser medido novamente. Identificadores não autorizam executar em outro alvo.

Toda operação longa usa cancelamento e eventos comuns. Propostas e inspeções não criam um segundo gerenciador de processos; execução interativa continua no terminal. Consultas periódicas só entram depois de medir custo e comportamento ao fechar o painel, desconectar ou trocar contexto.

Taxonomia mínima de falhas: ferramenta fora do contexto, setup falhou, comando/flag indisponível, pacote não resolvido, mensagens não geradas, Python incompatível, prazo de descoberta esgotado, saída não reconhecida, resultado truncado e destino desconectado. Deve haver mensagem humana e evidência; nunca completar campos desconhecidos por heurística silenciosa.

Saída bruta é preservada com limite e indicador de truncamento, sem segredos. Parsing fica no core, versionado e coberto por amostras reais. Se a CLI instalada não fornece dados suficientes, o resultado será parcial. Não corrigir essa limitação criando um cliente DDS, servidor auxiliar no alvo ou protocolo ROS próprio dentro da IDE.

## 11. Riscos e verificações exigidas

| Risco | Falha que o usuário perceberia | Tratamento proposto / prova necessária |
| --- | --- | --- |
| Ambiente/overlay errado | Build funciona no terminal e falha no editor; pacote errado executado | Testar dois overlays com pacote homônimo, ordem distinta e troca de contexto; conferir prefixo do artefato executado |
| Python incompatível | `rclpy` não importa, apesar de existir | Medir interpretador, biblioteca carregada e erro real; testar venv compatível/incompatível sem instalação corretiva |
| Flags e saídas mudam por versão | UI exibe lista vazia ou sucesso indevido | Fixtures capturadas por distro/versão, estado de parser incompatível e saída bruta acessível |
| Descoberta/RMW/rede | Grafo vazio, parcial ou diferente entre hosts | Medir domínio, RMW, interfaces e janela; ensaio com duas máquinas. Não alegar interoperabilidade universal |
| QoS incompatível | Tópico aparece, monitor/bag recebe zero mensagens | Testar publisher/subscriber com políticas compatíveis e incompatíveis; exibir perfil observado |
| Tempo simulado | Gravação não avança, TF/visualização divergem | Testar `/clock` ausente, pausa e salto; registrar fonte de tempo; não converter isso em desconexão |
| Launch dinâmico | Lista declarada difere dos processos reais | Identificar origem da informação e não confundir descrição com execução; testar includes/condições e falha na avaliação |
| Composição/múltiplos nós | Attach escolhe PID errado, breakpoint suspende mais de um nó | Provar associação processo/container e sessão única; nomes duplicados permanecem ambíguos |
| Cancelamento incompleto | Launch ou visualizador permanece aberto após parar | Ensaiar filho que termina, filho que resiste, desconexão e attach externo; declarar término só quando observado |
| Qt e plugins externos | Ferramenta não abre ou plugin não carrega | Medir aplicativo+plugin+distribuição juntos. Lyrical não permite presumir que todo visualizador continue usando Qt 5 |
| Licenças/telemetria | Pacote recomendado tem obrigações/dependências diferentes das esperadas | Fixar versão e conferir licença/build; casos PlotJuggler e Lichtblick já justificam essa separação |
| CPU/RAM/disco e tráfego | Observador interfere no sistema ou perde dados | Medir custo ocioso/ativo, saída, descarte e crescimento de bag; nenhum limite de desempenho foi provado nesta pesquisa |
| Bridge | “Visualização” também consegue publicar ou alterar parâmetros | Expor capacidades e endereço; usar servidor já disponível; não habilitar rede/cloud automaticamente |
| micro-ROS | Flash conclui, mas nenhuma mensagem chega | Verificar firmware, Agent, transporte e grafo separadamente; ensaio em placa obrigatório |

## 12. O que provar no host, na simulação, no QEMU e no hardware

**Todos os ensaios abaixo estão propostos e pendentes**, exceto a inspeção de disponibilidade/ajuda registrada na §1.1. O gate QEMU de embarcados já existente não contém, por inferência, um gate ROS 2.

| Ensaio de aceitação | Sem hardware dedicado | O que ainda exige equipamento/rede reais |
| --- | --- | --- |
| Detectar instalação fora do PATH | Host: caso já observado para Lyrical; futura UI/IPC ainda por testar | Destino remoto exige repetir nele |
| Build C++/Python com interface própria | Host/container com ROS previamente disponível; erro de compilação e dependente não construído | Toolchain/ABI de Linux embarcado precisam de alvo correspondente |
| Testes por pacote e resultado antigo | Host: suíte com aprovação, falha, pacote ignorado e rerun | Nenhum hardware para a semântica do runner |
| LSP com headers/imports gerados | Host: antes/depois da geração e troca de perfil/overlay | Biblioteca de driver específico pode exigir SDK disponível |
| GDB/LLDB e debugpy em um nó | Host: breakpoint verificado, variável, continue, stop/detach e fonte instalada | Attach remoto, permissões e timing do equipamento precisam de ensaio próprio |
| Pub/sub, serviços/actions e QoS | Dois processos no host; prova limitada a essa combinação | Rádio, multicast, firewall, perda e carga em duas máquinas não se provam no loopback |
| Snapshots de grafo e daemon | Host: domínio vazio, nó transitório, timeout e troca de RMW | Descoberta distribuída e ambientes de frota |
| Bag record/info/play | Host: mensagens sintéticas, contagens, replay e QoS; comparar resultado da ferramenta | Sustentação de câmera/LiDAR e armazenamento do alvo |
| Tempo e visualização robótica | Gazebo + `ros_gz` existentes; relógio, tópicos e RViz do par escolhido | Sensores físicos, calibração, latências e comportamento mecânico |
| Rust `rclrs` e `r2r` | Dois gates separados: mensagens próprias, serviços/actions usados pelo exemplo e debug nativo | Cross-compilation e runtime em ARM exigem alvo ou emulação específica, depois validação física |
| Firmware no QEMU | Build, símbolos, attach, memória e execução do modelo MCU suportado | Probe/SWD, reset elétrico, flash real, USB, SWO e periféricos não modelados |
| micro-ROS em emulação | Somente se existir imagem/RTOS/transporte suportado e Agent funcional; esse cenário não foi validado | Serial/USB/UDP reais, reset/reconexão, memória/timing do firmware e mensagens sob carga |
| Flash GNU Arm/OpenOCD | Reutilizar o estudo/gate de embarcados para o que ele mede | Modelo de placa/probe, mapa de flash e verificação após gravação |
| Encerramento e ausência de órfãos | Host: processo e descendentes sob controle da IDE | Queda de conexão e reinício de destino remoto |

Um QEMU de MCU não substitui Gazebo; Gazebo não substitui teste de placa. Uma VM Linux no QEMU pode testar uma instalação ROS emulada, mas isso não comprova um microcontrolador executando micro-ROS. As capacidades precisam nomear exatamente o cenário aprovado.

Cada execução do gate deve registrar: versão da Vectis, distribuição/OS/arquitetura, versões das ferramentas, RMW, perfil, exemplo e revisão, comandos/contexto, resultado esperado/observado, artefatos e pendências. Para desempenho, estabelecer orçamento a partir de medição; não inventar taxa de atualização, uso de RAM ou latência aceitável neste documento.

## 13. O que não entra

| Fora do escopo | Motivo |
| --- | --- |
| Qualquer entrega ROS 2 na 0.3 | Contraria o horizonte futuro solicitado |
| Instalar ROS, compilador, Python, RMW, Agent, bridge ou firmware automaticamente | O produto mede e informa; não provisiona por conta própria |
| Daemon da Vectis ou do FKIE em cada alvo | Viola a premissa de acesso por ferramentas existentes |
| ROS 1, `roscore`, catkin como fluxo principal ou ponte ROS 1/2 automática | Dilui a primeira matriz ROS 2 e introduz outro ciclo de compatibilidade |
| Implementação própria de colcon, ament, DDS, XRCE, OpenOCD, GDB ou ST-LINK | Ferramentas e protocolos permanecem nos processos apropriados |
| Bibliotecas cliente ROS no core só para introspecção | Tornariam o core dependente do runtime/ABI ROS para substituir ferramentas externas |
| Marketplace executável, plugins VS Code/Qt Creator/rqt/RViz dentro da Vectis | Arquitetura do projeto não possui esse host |
| Copiar plugins AGPL, transplantar Electron/Node ou adaptar código sem gate de licença | Incompatível com o modo de integração proposto e a política do repositório |
| Viewer 3D, simulador, plotter ou editor URDF visual próprios | Já há ferramentas candidatas; custo não justificado por lacuna medida na IDE |
| Garantia de multi-debug, debug distribuído automático ou PID inferido do nó | Contratos e provas ausentes |
| Painel genérico de operação/teleop, controle de frota ou envio automático de comandos físicos | Não é necessário para o ciclo de desenvolvimento inicial |
| Configuração automática de DDS Security/SROS2, certificados, VPN, firewall ou discovery server | Depende da implantação; deve ser respeitada e diagnosticada, não alterada pela IDE |
| Promessa de tempo real, certificação ou funcionamento de qualquer placa ROS | Exige evidência específica de hardware, aplicação e carga |
| Homologação indiscriminada de toda biblioteca Rust ROS | Bibliotecas, geração de tipos e estabilidade diferem |

## 14. Ordem recomendada para o trabalho futuro

| Etapa | Resultado que justificaria avançar |
| --- | --- |
| **R1 — Ambiente e workspace** | Contexto isolado, pacotes/interfaces e diagnóstico de ferramentas; nenhum script avaliado silenciosamente; caso “fora do PATH” resolvido pela inspeção |
| **R2 — Desenvolvimento diário C++/Python** | Colcon build/test, resultados por pacote, LSP e fontes instaladas coerentes; um processo depurável com o mesmo ambiente |
| **R3 — Execução e investigação** | Launch com encerramento comprovado, snapshots limitados, QoS e bags por ferramentas externas; abrir RViz/rqt disponíveis |
| **R4 — Remoto e simulação** | SSH com contexto de destino; bag e visualização opcionais; par ROS/Gazebo validado; nenhuma instalação no alvo |
| **R5 — Rust e micro-ROS qualificados** | Gates por cliente Rust e por firmware/placa/Agent; reaproveitamento da frente Embarcados já comprovada |
| **Estudos posteriores** | Multi-DAP, streaming volumoso e trace integrado somente quando os contratos e medições justificarem |

Essa ordem expressa dependências, não datas nem alocação automática de versões. A decisão de produto recomendada é investir na integração ROS 2, com foco inicial em C++/Python, usando **RDE e Qt Creator como referências de comportamento**, **ros2cli/colcon/rosdep como ferramentas da cadeia**, e **RViz/rqt/PlotJuggler/Lichtblick/Gazebo como aplicações externas qualificadas**. A promoção de cada capacidade depende dos ensaios da §12.

## 15. Registro das fontes e interpretação

As referências junto às afirmações apontam para fontes primárias. A data de consulta é **2026-09-24**, salvo teste/resultados colcon, lifecycle, ros2_control, licença rclrs, referência de comandos rosdep, descoberta configurável, TF e Xacro, conferidos em **2026-09-25**. A ajuda local de rosdep foi conferida também em 2026-09-25: `check` e `resolve` constavam entre os verbos; nenhum foi executado sobre um projeto nesta pesquisa.

| Grupo de fontes | Para que foi usado | Limite |
| --- | --- | --- |
| Documentação ROS 2 / fonte `ros2_documentation` | Distribuições, plataformas, linguagem, ambiente, launch, composição, QoS e daemon | Documentação upstream não é prova de comportamento da Vectis |
| Documentação colcon / rosdep e ajuda local | Argumentos, seleção, testes, dependências | Presença da CLI não comprova execução bem-sucedida de um workspace |
| RDE / Qt Creator, pins da §4.4 | Código de integração e referências de comportamento | Não houve execução dos plugins nem cópia de implementação |
| RViz, rqt, PlotJuggler/plugins, Lichtblick, bridges e FKIE | Natureza das ferramentas, licenças, dependências e restrições | Auditoria de binários, transitivas e rede continua pendente |
| ros2_rust / r2r | Fluxos e limites das bibliotecas Rust | Suporte declarado não equivale à matriz homologada da IDE |
| micro-ROS, Gazebo/ros_gz e frameworks | Relação entre firmware, Agent, simulação e aplicação | Nenhum ensaio de hardware ou simulação foi realizado |
| Core/protocolo local | Inventário IPC e justificativas dos contratos novos | Leitura estática; futuros endpoints não estão implementados |

Branches móveis foram usadas para panorama; os componentes estudados como referência profissional têm pins registrados. Uma atualização da pesquisa deve revisar datas, versões, arquivo de licença e escopo efetivo, especialmente quando README e código divergem. As decisões marcadas como propostas são inferências desta avaliação, não afirmações dos mantenedores.

[clients]: https://github.com/ros2/ros2_documentation/blob/jazzy/source/Concepts/Basic/About-Client-Libraries.rst "Consulta 2026-09-24"
[releases]: https://github.com/ros2/ros2_documentation/blob/rolling/source/Releases.rst "Consulta 2026-09-24"
[platforms]: https://github.com/ros2/ros2_documentation/blob/rolling/source/Releases/lyrical/supported-platforms.rst "Consulta 2026-09-24"
[rep2000]: https://reps.openrobotics.org/rep-2000/ "Consulta 2026-09-24"
[middleware]: https://github.com/ros2/ros2_documentation/blob/rolling/source/ROS-Framework/client-libraries/About-Different-Middleware-Vendors.rst "Consulta 2026-09-24"
[colconbuild]: https://colcon.readthedocs.io/en/released/reference/verb/build.html "Consulta 2026-09-24"
[colconselect]: https://colcon.readthedocs.io/en/released/reference/package-selection-arguments.html "Consulta 2026-09-24"
[overlays]: https://colcon.readthedocs.io/en/released/user/using-multiple-workspaces.html "Consulta 2026-09-24"
[launchpython]: https://github.com/ros2/launch/blob/lyrical/launch/launch/launch_description_sources/python_launch_file_utilities.py "Consulta 2026-09-24"
[colconpython]: https://github.com/colcon/colcon-python-setup-py/blob/master/colcon_python_setup_py/package_identification/python_setup_py.py "Consulta 2026-09-24"
[python]: https://github.com/ros2/ros2_documentation/blob/jazzy/source/How-To-Guides/Using-Python-Packages.rst "Consulta 2026-09-24"
[rclrs]: https://github.com/ros2-rust/ros2_rust/blob/c36e7a3040c3a2e299521591747b23b7e8b62a18/README.md "Consulta 2026-09-24"
[r2r]: https://github.com/sequenceplanner/r2r/blob/653167aadcc32b20cb13b6542460ee1926ed933c/README.md "Consulta 2026-09-24"
[rclrslicense]: https://github.com/ros2-rust/ros2_rust/blob/c36e7a3040c3a2e299521591747b23b7e8b62a18/LICENSE "Consulta 2026-09-25"
[r2rlicense]: https://github.com/sequenceplanner/r2r/blob/653167aadcc32b20cb13b6542460ee1926ed933c/LICENSE "Consulta 2026-09-24"
[ides]: https://github.com/ros2/ros2_documentation/blob/rolling/source/Developer-Tools/Debugging/ROS-2-IDEs.rst "Consulta 2026-09-24"
[colcontest]: https://colcon.readthedocs.io/en/released/reference/verb/test.html "Consulta 2026-09-25"
[colcontestresult]: https://colcon.readthedocs.io/en/released/reference/verb/test-result.html "Consulta 2026-09-25"
[ros2cli]: https://github.com/ros2/ros2cli "Consulta 2026-09-24"
[colconlicense]: https://github.com/colcon/colcon-core/blob/master/LICENSE "Consulta 2026-09-24"
[rosdeplicense]: https://github.com/ros-infrastructure/rosdep/blob/master/LICENSE "Consulta 2026-09-24"
[rosdepcommands]: https://github.com/ros-infrastructure/rosdep/blob/master/doc/commands.rst "Consulta 2026-09-25; complementada pela ajuda local"
[vcstool]: https://github.com/dirk-thomas/vcstool "Consulta 2026-09-24"
[bags]: https://github.com/ros2/rosbag2/blob/jazzy/README.md "Consulta 2026-09-24"
[rvizlicense]: https://github.com/ros2/rviz/blob/rolling/LICENSE "Consulta 2026-09-24"
[rviz]: https://github.com/ros2/rviz "Consulta 2026-09-24"
[rqt]: https://github.com/ros-visualization/rqt "Consulta 2026-09-24"
[rqtlicense]: https://github.com/ros-visualization/rqt/blob/rolling/LICENSE "Consulta 2026-09-24"
[gazebo]: https://github.com/gazebosim/gz-sim "Consulta 2026-09-24"
[rosgz]: https://github.com/gazebosim/ros_gz/blob/ros2/README.md "Consulta 2026-09-24"
[tracing]: https://github.com/ros2/ros2_tracing/blob/jazzy/README.md "Consulta 2026-09-24"
[rde]: https://github.com/Ranch-Hand-Robotics/rde-ros-2/tree/4928feba8734707b5841a1f5db83b61ff3661a16 "Consulta 2026-09-24"
[qtc]: https://github.com/ros-industrial/ros_qtc_plugin/blob/3c4dcb2718f47ff71e484e9845fb6f36d547726e/README.md "Consulta 2026-09-24"
[qtccolcon]: https://github.com/ros-industrial/ros_qtc_plugin/blob/3c4dcb2718f47ff71e484e9845fb6f36d547726e/src/project_manager/ros_colcon_step.cpp "Consulta 2026-09-24"
[oldros]: https://github.com/ms-iot/vscode-ros "Consulta 2026-09-24"
[rqtgraph]: https://github.com/ros-visualization/rqt_graph "Consulta 2026-09-24"
[rqtgraphindex]: https://index.ros.org/p/rqt_graph/ "Consulta 2026-09-24"
[plot]: https://github.com/PlotJuggler/PlotJuggler/tree/c3077496c37c85f0457eda8f4404c3dc3a42be3a "Consulta 2026-09-24"
[plotpluginslicense]: https://github.com/PlotJuggler/plotjuggler-ros-plugins/blob/9cb0b596bc94bde4fa4e9f3a462dcbe78d5a2377/LICENSE "Consulta 2026-09-24"
[plotpluginspackage]: https://github.com/PlotJuggler/plotjuggler-ros-plugins/blob/9cb0b596bc94bde4fa4e9f3a462dcbe78d5a2377/package.xml "Consulta 2026-09-24"
[plotpluginsci]: https://github.com/PlotJuggler/plotjuggler-ros-plugins/blob/9cb0b596bc94bde4fa4e9f3a462dcbe78d5a2377/.github/workflows/ros-jazzy.yaml "Consulta 2026-09-24"
[licht]: https://github.com/lichtblick-suite/lichtblick/blob/c72097a9c58129e4cbb84b898ed3d220ae6048a0/README.md "Consulta 2026-09-24"
[lichtlicense]: https://github.com/lichtblick-suite/lichtblick/blob/c72097a9c58129e4cbb84b898ed3d220ae6048a0/LICENSE "Consulta 2026-09-24"
[oldfoxglove]: https://github.com/foxglove/studio "Consulta 2026-09-24"
[foxbridge]: https://github.com/foxglove/foxglove-sdk/blob/dcbc66776e8704f5b4f9aa0c6e3ef695ed4c297b/ros/src/foxglove_bridge/README.md "Consulta 2026-09-24"
[oldbridge]: https://github.com/foxglove/ros-foxglove-bridge/blob/main/README.md "Consulta 2026-09-24"
[rosbridge]: https://github.com/RobotWebTools/rosbridge_suite/tree/ros2 "Consulta 2026-09-24"
[fkie]: https://github.com/fkie/fkie-multi-agent-suite "Consulta 2026-09-24"
[rdeattach]: https://github.com/Ranch-Hand-Robotics/rde-ros-2/blob/4928feba8734707b5841a1f5db83b61ff3661a16/src/debugger/configuration/resolvers/attach.ts "Consulta 2026-09-24"
[rdetelemetry]: https://github.com/Ranch-Hand-Robotics/rde-ros-2/blob/4928feba8734707b5841a1f5db83b61ff3661a16/src/telemetry-helper.ts "Consulta 2026-09-24"
[rdetests]: https://github.com/Ranch-Hand-Robotics/rde-ros-2/blob/4928feba8734707b5841a1f5db83b61ff3661a16/test/suite/build-env-utils.test.ts "Consulta 2026-09-24"
[rdepin]: https://github.com/Ranch-Hand-Robotics/rde-ros-2/commit/4928feba8734707b5841a1f5db83b61ff3661a16 "Consulta 2026-09-24"
[qtcpin]: https://github.com/ros-industrial/ros_qtc_plugin/commit/3c4dcb2718f47ff71e484e9845fb6f36d547726e "Consulta 2026-09-24"
[plotpin]: https://github.com/PlotJuggler/PlotJuggler/commit/c3077496c37c85f0457eda8f4404c3dc3a42be3a "Consulta 2026-09-24"
[plotpluginspin]: https://github.com/PlotJuggler/plotjuggler-ros-plugins/commit/9cb0b596bc94bde4fa4e9f3a462dcbe78d5a2377 "Consulta 2026-09-24"
[lichtpin]: https://github.com/lichtblick-suite/lichtblick/commit/c72097a9c58129e4cbb84b898ed3d220ae6048a0 "Consulta 2026-09-24"
[rclrspin]: https://github.com/ros2-rust/ros2_rust/commit/c36e7a3040c3a2e299521591747b23b7e8b62a18 "Consulta 2026-09-24"
[r2rpin]: https://github.com/sequenceplanner/r2r/commit/653167aadcc32b20cb13b6542460ee1926ed933c "Consulta 2026-09-24"
[foxpin]: https://github.com/foxglove/foxglove-sdk/commit/dcbc66776e8704f5b4f9aa0c6e3ef695ed4c297b "Consulta 2026-09-24"
[launch]: https://github.com/ros2/ros2_documentation/blob/jazzy/source/Concepts/Basic/About-Launch.rst "Consulta 2026-09-24"
[composition]: https://github.com/ros2/ros2_documentation/blob/jazzy/source/Concepts/Intermediate/About-Composition.rst "Consulta 2026-09-24"
[daemon]: https://github.com/ros2/ros2_documentation/blob/jazzy/source/Concepts/Basic/About-Command-Line-Tools.rst "Consulta 2026-09-24"
[qos]: https://github.com/ros2/ros2_documentation/blob/jazzy/source/Concepts/Intermediate/About-Quality-of-Service-Settings.rst "Consulta 2026-09-24"
[lifecycle]: https://design.ros2.org/articles/node_lifecycle.html "Consulta 2026-09-25"
[discovery]: https://github.com/ros2/ros2_documentation/blob/jazzy/source/Concepts/Basic/About-Discovery.rst "Consulta 2026-09-24"
[micromiddleware]: https://github.com/micro-ROS/micro-ROS.github.io/blob/master/_docs/concepts/middleware/Micro_XRCE-DDS/index.md "Consulta 2026-09-24"
[microsetup]: https://github.com/micro-ROS/micro_ros_setup/blob/kilted/README.md "Consulta 2026-09-24"
[nav2]: https://github.com/ros-navigation/navigation2/blob/main/navigation2/package.xml "Consulta 2026-09-24"
[moveit]: https://github.com/moveit/moveit2 "Consulta 2026-09-24"
[control]: https://control.ros.org/rolling/doc/getting_started/getting_started.html "Consulta 2026-09-25"
[dynamicdiscovery]: https://github.com/ros2/ros2_documentation/blob/rolling/source/Developer-Tools/Introspection-and-analysis/Improved-Dynamic-Discovery.rst "Consulta 2026-09-25"
[tftools]: https://github.com/ros2/geometry2/blob/rolling/tf2_ros/doc/cli_tools.rst "Consulta 2026-09-25"
[tfframes]: https://github.com/ros2/geometry2/blob/rolling/tf2_tools/tf2_tools/view_frames.py "Consulta 2026-09-25"
[xacro]: https://github.com/ros/xacro "Consulta 2026-09-25"
