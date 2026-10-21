# Embarcados — rascunho de evolução da HUD e UX na 0.4.x

> **Classe: PLANO / RASCUNHO DE UX.** Pesquisa recebida em 2026-10-21.
> **Nenhuma das imagens é fonte da verdade ou design aprovado.** A direção
> visual depende da aprovação do autor, seguindo a interface atual da IDE.
> A imagem 5 é a aproximação preferida pelo autor, ainda sujeita a
> refinamento e aprovação visual. Não reproduzir os mockups literalmente.

## 1. O que está decidido e o que é proposta

**Decidido pelo autor:** melhorar/incrementar as HUDs e polir a experiência
sobre a UI atual, preservando organização da janela, editor prioritário,
identidade grafite/âmbar, componentes e donos existentes. Não redesenhar a
IDE nem transformar Embarcados em um dashboard de configurações.

**Em avaliação:** disposição interna, densidade, textos, agrupamentos,
expansões, hierarquia de ações e microinterações descritos abaixo. Os
estudos anteriores de frontend também são rascunhos de solução visual;
inclusão de uma frente no roadmap não aprova seu desenho final. A regra
comum está na [arquitetura de frontend §1.1](arquitetura-de-frontend-0.3-em-diante.md#11-aprovação-visual-e-referências--decisão-de-2026-10-21).

Código/gates continuam demonstrando **o que existe**; aprovação do autor
define **a direção visual aceita**. Estados e capacidades não podem ser
declarados implementados/detectados a partir de uma imagem ou aprovação de UX.

## 2. Referências recebidas

| Referência | Uso na pesquisa | Limite |
| --- | --- | --- |
| [Imagem 1](embarcados-hud-ux-referencia-1.png) | Seleção de ambiente/alvo e acesso a ferramentas | Excesso de informação, campos e cartões ainda em avaliação |
| [Imagem 2](embarcados-hud-ux-referencia-2.png) | Resumo, estado e ações por contexto | A segunda fileira de ecossistemas não é navegação aprovada |
| [Imagem 3](embarcados-hud-ux-referencia-3.png) | Relação entre board, runner, interface e dispositivo | Campos/ações dependem de suporte real, não da presença no mockup |
| [Imagem 4](embarcados-hud-ux-referencia-4.png) | Comparação funcional entre conteúdos das abas e Linux | Cinco estados lado a lado são comparação; não criar cinco painéis simultâneos |
| [Imagem 5](embarcados-hud-ux-referencia-5.png) | Aproximação mais próxima da UI atual, segundo o autor | Preferência de referência, ainda sem aprovação visual final ou prova nativa |

Todas são referências conceituais. Projeto, ROS, SDK, board, versões,
caminhos, datas, diagnósticos e estados nelas são ilustrativos. Não inferir
compatibilidade, Windows ativo, suporte a um framework ou nova feature do
editor a partir dessas imagens. Fotos de placas, pinouts e links técnicos
exigem origem/licença/identidade verificadas antes de entrar no produto.

## 3. Síntese funcional candidata

O painel deve ajudar a responder rapidamente: **qual contexto/alvo está
ativo, o que foi verificado, como está o build e qual ação faz sentido agora**.
Preferência de pesquisa: resumo operacional compacto, com detalhes
progressivos que permitam alcançar uma visão de engenharia mais densa.
Não há aprovação de modos separados “iniciante” e “profissional”.

| Contexto | Informação prioritária | Ações candidatas, conforme capacidades reais |
| --- | --- | --- |
| Bare metal | MCU, arquitetura, kit, linker, probe e artefato associado ao alvo | Compilar, revisar gravação, depurar |
| Zephyr/RTOS | Board, SDK, configuração, diretório de build e runner efetivo | Build, flash, debug e monitor no dono atual |
| PlatformIO | Environment declarado, plataforma/framework, board e upload | Build, upload, test/check e monitor por environment |
| Linux embarcado | Arquitetura, SDK/sysroot e conexão remota | Compilar, deploy, executar/debug remoto e logs |
| MicroPython/CircuitPython | Runtime correto, conexão, arquivos e capacidades distintas | Conectar, executar/transferir e REPL onde suportado |
| Emulação | Modelo, arquitetura, emulador e imagem/alvo | Iniciar, depurar e inspecionar onde integrado |
| Não identificado ou ambíguo | Evidências encontradas, limitações e escolha explícita | Abrir/configurar manualmente sem inventar contexto |

Priorizar o ecossistema efetivo, sem esconder os acessos já disponíveis.
Não obrigar o usuário a escolher novamente o tipo do projeto a cada abertura.
Em múltiplos alvos/environments, tornar a seleção explícita, persistente e
visível, distinguindo uma escolha de visualização da configuração usada pelo
runner. Exemplos aninhados não substituem o runtime principal.

## 4. Organização sobre os componentes atuais

Conferência estática em 2026-10-21: `ui/qml/embedded/EmbeddedPanel.qml` já
tem quatro abas com rótulos contextuais: **Visão geral, Placa/Conexões,
Gravar/Implantar e Ambiente**. `EmbeddedOverviewView.qml` já distingue
PlatformIO, MicroPython e Linux; `EmbeddedEnvironmentView.qml` trata kits
gerenciados. A pesquisa estende essa base; não prova que os novos fluxos
ou o desenho das imagens já existem.

- **Visão geral:** testar resumo compacto de projeto/contexto, estado do
  ambiente, último build pertinente e uma ação principal contextual. Detalhe
  técnico, saída completa e outras ações ficam acessíveis sob demanda.
- **Placa/Conexões:** distinguir alvo configurado, board de catálogo,
  dispositivo conectado, probe e porta. Ausência de USB é normal quando
  o fluxo usa SSH/emulador ou ainda não exige hardware.
- **Gravar/Implantar:** relacionar artefato, alvo, contexto de build e destino;
  apresentar o plano real antes de executar. SSH/deploy não vira gravação MCU.
- **Ambiente:** resumo e origem dos valores efetivos; configuração avançada
  sob demanda. Ausência de ferramenta abre o dono de
  [Instalar Ferramentas](instalar-ferramentas.md), sem novo catálogo/executor.

Logs, testes e diagnósticos detalhados usam Build, Terminal, Problemas,
Testes e Jobs já existentes. SSH e depuração usam seus gestores atuais;
Qualidade usa o [Quality Center](quality-center-e-gates.md). Informações
repetidas devem apontar ao dono do valor, sem criar estados concorrentes.

## 5. Estados, evidência e previsibilidade

Separar detectado, selecionado/configurado, validado, incompatível, ausente,
não verificado e obsoleto, sem depender só de cores. Cada problema relevante
explica observação, origem, impacto e ação fundamentada. Exemplo conceitual:
“SDK não validado para a arquitetura selecionada” leva a revisar o SDK;
não vira incompatibilidade confirmada apenas porque faltou informação.

**Artefatos:** um ELF encontrado é candidato, não prova de firmware gravável.
Conferir vínculo com target/environment, arquitetura, build e operação
compatível antes de oferecê-lo para gravação. Reproduzir o caso de projeto
Rust de desktop com ELFs no workspace; não apresentar um fluxo MCU só pela
extensão do arquivo. Alteração de alvo/kit invalida evidências anteriores.

**Operação:** revisão identifica binário, destino/dispositivo e efeitos.
Apagar flash, sobrescrever, reiniciar ou desconectar não recebem defaults
indiscriminados dos mockups. Respeitar o plano da ferramenta, capacidades,
confirmação e proteções existentes. Reconhecer projeto/SDK não executa
scripts desconhecidos; preservar Project Health e confiança no workspace.

Atualização/desconexão mantém foco, seleção e posição quando possível;
resposta antiga não repõe dados de outro contexto. Progresso vem de eventos
reais; uma ação pode abrir o diagnóstico/log sem repetir a operação.

## 6. Prioridades e encaixe na versão

| Prioridade | Aplicação | Etapa da sequência vigente |
| --- | --- | --- |
| P0 — contexto e operação | Contexto correto, associação de artefatos, ações pertinentes, confirmação e recuperação | h4b2 e integrações 0.4.1–0.4.3, conforme responsabilidade; consolidação 0.4.6 |
| P1 — HUD/UX | Resumo, hierarquia, densidade, formulários progressivos e integração sem duplicação | Junto de cada fluxo; auditoria integrada 0.4.7 |
| P2 — acabamento | Microinterações, superfícies, transições e otimização medida | 0.4.8 e edição especial após 0.4.9 |

Foco visível, teclado, legibilidade, estados compreensíveis e movimento
reduzido acompanham o aceite funcional; não são adiados apenas pelo rótulo P2.
Tempos como 110–180 ms são sugestões a conferir nos tokens/medição, não
defaults aprovados. Não animar largura do editor ou mover painéis por eventos.

Ordem única no [52 §11.1](../roadmaps/52-arquitetura-executavel-da-0.4.md#111-fluxo-de-execução-reorganizado--2026-10-21).
Nada neste rascunho antecipa ROS 2, amplia automaticamente o suporte a
ferramentas ou altera a sequência **0.4.9 → edição especial → AppImage**.

## 7. Refinamento e aceite

Comparar uma captura nativa da UI atual com uma proposta concreta derivada
dela, nas mesmas tarefas/larguras. Apresentar o resultado revisável ao autor
e registrar sua aprovação ou ajustes antes de dar o desenho como aceito.
A revisão visual pendente não bloqueia correções funcionais já autorizadas
que independem desse desenho. Não usar a imagem 5 como aprovação implícita.

Provar projetos bare metal, Zephyr, PlatformIO e Linux, além de runtimes
Python distintos, projeto incompleto, workspace não confiável, sem hardware,
múltiplos alvos, ferramenta ausente, cancelamento e resultado obsoleto.
Conferir teclado, escala, textos longos e português/inglês, posição/foco em
atualizações e resposta sob build/análise. Fixtures, prova nativa do agente
e aprovação visual do autor são evidências diferentes e devem ser registradas.
