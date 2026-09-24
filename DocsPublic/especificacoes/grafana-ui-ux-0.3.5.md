# Grafana — integração prática e ergonômica para a 0.3.5

> **Classe: ALVO / PLANO.** Análise escrita em 2026-09-22 sobre a
> implementação existente. Grafana entra obrigatoriamente no fechamento da
> série 0.3, na **0.3.5**. O código e os contratos medidos vencem este plano.

## 1. Decisão de produto

Grafana não entra na 0.3.5 apenas para receber a moldura visual que faltou na
E3-7. A integração precisa ser mais simples que reproduzir o fluxo suportado
com `curl`, variáveis de ambiente e o navegador.

O fluxo suportado continua deliberadamente pequeno:

```text
ligar um projeto a uma instância já existente
→ provar saúde e autenticação
→ descobrir fontes de dados e dashboards
→ mostrar quais bancos do projeto já são observados
→ localizar e abrir o dashboard certo no navegador
```

A IDE não embute nem redistribui o Grafana, não vira editor de dashboards e
não persiste token. A integração continua pela HTTP API e respeita a fronteira
de licença registrada em
[`37-banco-e-observabilidade.md`](../integracoes/37-banco-e-observabilidade.md).

## 2. Base existente que será preservada

- perfil por workspace com URL e política do token;
- token ausente, vindo de variável de ambiente ou pedido na sessão;
- token em memória separado do perfil e redigido no transporte/log;
- `/api/health`, `/api/datasources` e `/api/search?type=dash-db`;
- estados distintos para alcançável e autenticado;
- fontes de dados, dashboards e cruzamentos com os bancos do projeto;
- dashboard aberto externamente no navegador;
- `GrafanaController` e o domínio `grafana.*` como donos atuais;
- testes Rust do perfil/store e prova anterior contra Grafana real.

Não se cria outro cliente HTTP, outro armazenamento de segredo ou cópia da
regra de cruzamento no QML.

## 3. Diagnóstico medido da interface atual

`GrafanaPanel` apresenta, na mesma coluna:

```text
título e explicação
URL
três políticas de token
nome da variável ou prompt do token
Salvar · Sondar · Esquecer
veredito
cruzamentos · fontes de dados · dashboards
```

Isso torna visíveis as capacidades, mas mistura três momentos diferentes:

1. **conectar pela primeira vez**, que é raro;
2. **resolver autenticação**, que deve aparecer só quando necessária;
3. **usar observabilidade**, que é o trabalho cotidiano.

Outros atritos:

- a ação inicial é ambígua: salvar antes de sondar é exigência do desenho, não
  intenção do usuário;
- “Sem token”, “Variável de ambiente” e “Pedir na sessão” aparecem antes de a
  API dizer se autenticação é necessária;
- o formulário continua dominante depois de configurado;
- listas longas não têm filtro, seleção persistente nem navegação por teclado;
- o resultado principal — banco do projeto ligado a uma fonte — compete com
  inventários auxiliares;
- estados vazio, não testado, alcançável sem autenticação e autenticado não
  formam um próximo passo único;
- o token em memória morre ao fechar o painel, portanto reabrir pode pedir a
  credencial novamente na mesma sessão de trabalho;
- o painel não informa idade/origem da última prova e pode parecer atual sem
  que tenha sido consultado nesta abertura do workspace.

## 4. Régua de ergonomia

Para a integração já configurada, o usuário deve conseguir:

```text
abrir Grafana → entender o estado → filtrar → abrir dashboard
```

sem editar o perfil, escolher novamente a origem do token ou interpretar JSON.

Para a primeira configuração:

- `http://localhost:3000` aparece como candidato, não como conhecimento;
- URL é o único campo inicial;
- **Conectar** testa antes de transformar configuração em trabalho diário;
- autenticação só é pedida se o servidor exigir;
- configuração avançada de token aparece por progressive disclosure;
- o resumo final diz exatamente o que será guardado e o que ficará apenas em
  memória;
- erro sempre oferece ação seguinte: corrigir URL, tentar novamente ou
  fornecer token.

O aceite não é “a requisição funcionou”. É o caminho comum exigir menos
memória, menos gestos e menos conhecimento de API que o terminal.

## 5. Duas superfícies, não um formulário permanente

### 5.1 Primeiro uso — assistente curto

```text
1. Instância
   [ http://localhost:3000                         ] [ Conectar ]

2. Autenticação, somente se necessária
   ( ) usar uma vez nesta sessão
   ( ) ler da variável [ GRAFANA_TOKEN ]

3. Confirmação
   Grafana 13.x · autenticado · N fontes · M dashboards
   perfil salvo: URL + política; token não será salvo
```

“Salvar” deixa de ser pré-condição visual para “Sondar”. O controller pode
compor internamente salvar + sondar com os métodos atuais, desde que falhas
sejam distinguíveis e o perfil anterior não seja perdido silenciosamente.
Se isso não for possível sem ambiguidade, entra um método tipado de teste de
rascunho; não se falsifica o fluxo só para evitar contrato novo.

### 5.2 Uso diário — Overview

```text
Grafana · projeto atual                    [ Atualizar ] [ configurar… ]
autenticado em grafana.exemplo · medido agora

OBSERVA ESTE PROJETO
PostgreSQL produção  →  datasource-prod             [ abrir dashboards ]

[ filtrar dashboards, pasta ou fonte…                              ]
Dashboards
  Visão geral da API             Plataforma                    [ abrir ↗ ]
  Banco / latência               Dados                         [ abrir ↗ ]
```

- configuração fica recolhida em **configurar…**;
- atualizar é a única ação primária;
- cruzamentos aparecem antes dos inventários;
- dashboards e fontes usam `KvDataGrid`, seleção e teclado;
- filtro atua localmente sobre o resultado já recebido;
- abrir usa o navegador e deixa isso claro pelo ícone/rótulo;
- estados vazio, carregando, erro e sem autenticação possuem texto e ação
  próprios;
- detalhes de API ficam acessíveis, mas não dominam a tela.

## 6. Máquina de estados da apresentação

```text
setup       absent | editing | saved
probe       unknown | probing | reachable | failed
auth        not_required | required | authenticated | failed
content     unknown | loading | ready | empty | stale
```

Regras:

- `reachable` não implica `authenticated`;
- resultados pertencem à URL e ao workspace que originaram a requisição;
- trocar URL invalida visualmente o resultado anterior;
- “atualizado” exige resposta desta sessão e apresenta horário/idade;
- falha de atualização pode manter o último resultado como **desatualizado**,
  nunca como resultado novo;
- fechar/reabrir a tool window não transforma dado antigo em prova atual;
- o QML deriva apresentação apenas de respostas tipadas, não de texto de erro.

O controller atual já possui boa parte desses fatos. Idade, vínculo entre
requisição e rascunho e estado `stale` entram somente onde forem necessários
para não mentir.

## 7. Token: decisão de sessão e proteções obrigatórias

> **DECIDIDO em 2026-09-22:** o token digitado permanece somente em memória
> durante a sessão do workspace/aplicação. Fechar a tool window não o apaga.
> Trocar/fechar o workspace, mudar a instância confirmada, esquecer a
> credencial/instância ou encerrar a aplicação o invalida.

Invariantes preservados:

- token nunca entra no perfil, workspace, log, comando ou screenshot;
- variável de ambiente guarda somente o **nome** no perfil;
- token digitado fica apenas em memória e possui ação explícita **Esquecer
  credencial da sessão**;
- mudar de workspace, esquecer a instância ou encerrar a aplicação o apaga.

A implementação atual também apaga o token ao simplesmente fechar o painel.
Isso protege agressivamente, mas transforma abrir/fechar uma tool window em
novo login. A 0.3.5 muda essa fronteira de forma deliberada:

```text
fechar/reabrir painel       mantém o token da sessão
trocar/fechar workspace     apaga
confirmar outra URL         apaga antes de consultar a URL nova
esquecer credencial         apaga imediatamente
esquecer instância          apaga antes de remover o perfil
token rejeitado             apaga e pede outro
encerrar aplicação          memória deixa de existir com o processo
```

### 7.1 Vínculo que impede envio à instância errada

O token possui contexto de custódia, mesmo sem virar objeto persistido:

```text
credentialContext = workspaceRoot + profileUrl
```

- somente uma requisição para o mesmo par pode reutilizá-lo;
- editar a URL no rascunho não muda o contexto nem envia o token antigo;
- confirmar URL diferente primeiro invalida o token e o resultado anterior;
- resposta atrasada da URL antiga é descartada por identidade da requisição;
- token rejeitado por autenticação é invalidado antes de mostrar o novo prompt;
- variável de ambiente continua sendo política do perfil; o valor não é
  copiado para uma propriedade visual para ganhar conveniência.

### 7.2 Proteções obrigatórias

1. token somente em memória; nunca em perfil, workspace, settings, sessão de
   editor, log, comando, erro, screenshot ou telemetria;
2. vínculo rígido a workspace + URL confirmada;
3. limpeza nos eventos da tabela acima;
4. ação visível **Esquecer credencial da sessão**, disponível mesmo com a
   tool window autenticada;
5. campo do token com echo de senha, sem binding que devolva o valor à tela;
6. payload e headers continuam cobertos pela redação no `CoreClient` e core;
7. presença do token não inicia refresh ou requisição em segundo plano;
8. componentes da view recebem somente `authenticated/tokenRequired`, nunca o
   token;
9. testes de ciclo de vida e de ausência em persistência/log são bloqueadores
   da entrega.

### 7.3 Limite técnico declarado

Hoje `sessionToken` é uma `string` no `GrafanaController.qml`. Atribuir `""`
impede reuso funcional, mas uma string QML/JavaScript pode permanecer no heap
até coleta de memória; isso não é apagamento criptográfico garantido.

Para a primeira implementação, a revisão precisa medir o alcance dessa
propriedade e impedir bindings/logs. Se a custódia prolongada no QML ampliar o
alcance ou tornar a prova frágil, o token efêmero migra para uma pequena
custódia C++ não exposta como property. Isso não autoriza cofre, persistência ou
dependência nova. Mesmo em C++, cópias feitas pelo stack Qt/JSON precisam ser
minimizadas e redigidas; prometer zeroização perfeita seria incorreto.

O ganho aceito é um prompt por sessão em vez de um por abertura do painel. O
risco adicional é o maior tempo na memória do processo; não muda a fronteira
de disco, rede ou autorização do Grafana.

### 7.4 Caminho medido do segredo e custódia alvo

O caminho atual foi conferido em 2026-09-22:

```text
GrafanaTokenPrompt
→ GrafanaController.sessionToken                  string QML duradoura
→ GrafanaRequestRouter.onProbeRequested(token)   argumento QML transitório
→ CoreClient::grafanaProbe(QString)               QString + QJsonObject
→ JSON-RPC local                                  payload real contém token
→ GrafanaProbeParams.token                        String Rust
→ Secret                                          wrapper no core
→ Authorization: Bearer …                         requisição ao Grafana
```

Proteções que já existem:

- `GrafanaProfile` não possui campo token e rejeita campo desconhecido;
- store Rust testa que token/password/secret não chegam ao arquivo;
- `CoreClient::sendRequest` substitui por `***` campos `token`, `password`,
  `secret` e equivalentes em qualquer profundidade antes de `appendLog`;
- o core envolve a credencial no tipo `Secret` antes do cliente HTTP;
- a resposta não devolve o token.

Lacunas medidas:

- a propriedade QML é legível por quem tiver referência ao controller;
- o token é mantido no QML, embora só precise atravessá-lo no aceite do prompt;
- não existe hoje `scripts/qml-harness/tst_grafana.qml`, apesar de o roadmap 44
  ter registrado que esse harness cobria o controller;
- a redação C++ existe no código, mas não tem teste de regressão dedicado;
- vínculo workspace + URL, limpeza por rejeição e **Esquecer credencial** ainda
  não existem.

Como a nova política prolonga a custódia, o alvo da 0.3.5 é retirar a string
duradoura do QML. Um estado privado e específico do domínio no lado C++ da
bridge guarda token + contexto; não é property legível nem cofre genérico:

```text
prompt accepted(token)
→ CoreClient/GrafanaCredentialSession.remember(token, workspace, profileUrl)
→ QML descarta o argumento e mantém apenas credentialAvailable

probe(workspace, profileUrl)
→ bridge compara o contexto
→ somente se idêntico inclui token em grafana.probe
```

O nome/classe final acompanha a divisão real do `CoreClient`; não se cria uma
abstração universal de segredos. O protocolo `grafana.probe { token? }` não
precisa mudar. O token ainda terá cópias transitórias no QString/JSON/IPC/Rust,
mas deixa de ser uma property QML duradoura. Se essa mudança mostrar custo ou
acoplamento desproporcional na implementação, a alternativa QML só pode ser
aceita com alcance medido e os mesmos testes; conveniência não dispensa prova.

### 7.5 Matriz de testes bloqueadora

| caso | resultado obrigatório |
| --- | --- |
| fechar/reabrir tool window | reutiliza a credencial do mesmo contexto |
| trocar ou fechar workspace | credencial anterior indisponível |
| editar URL sem confirmar | não envia token à URL de rascunho |
| confirmar URL diferente | limpa antes da primeira consulta nova |
| resposta atrasada da URL anterior | descartada |
| autenticação rejeitada | limpa e volta a pedir token |
| Esquecer credencial | limpa sem remover o perfil |
| Esquecer instância | limpa antes de remover o perfil |
| reiniciar aplicação | pede token novamente |
| persistência | arquivos não contêm sentinela secreta |
| log da bridge/core | sentinela ausente; campo aparece somente como `***` |
| view/screenshot de harness | valor nunca aparece |

O harness QML novo cobre estado/transições; um teste C++ cobre a redação real
de objeto aninhado; os testes Rust existentes continuam cobrindo contrato e
store. A prova com Grafana real cobre 401 e token válido, mas não substitui os
testes de vazamento.

## 8. Descoberta sem varredura invasiva

A IDE não varre a rede. Pode oferecer candidatos locais de origem explicável:

- `http://localhost:3000`;
- containers Grafana já conhecidos pelo domínio de Containers e suas portas
  publicadas;
- URL já salva no workspace;
- variável de ambiente explicitamente suportada, se houver decisão de produto
  e contrato para isso.

Cada candidato mostra a origem (“padrão local”, “container X”, “perfil do
workspace”). Descoberta de container reutiliza os dados existentes; não cria
um segundo scanner. URL remota continua sendo entrada direta.

## 9. Arquitetura e fronteiras

```text
Grafana tool window
 ├─ GrafanaOverview          estado e gesto diário
 ├─ GrafanaConnectionSetup   primeira configuração/edição
 ├─ GrafanaMatches           resposta principal
 └─ GrafanaCatalog           filtro + grids de fontes/dashboards
              │
       GrafanaController     estado de apresentação e token efêmero
              │ signals
       CoreClient / grafana.*
              │
       core Rust HTTP API    valida, redige e cruza dados
```

- `GrafanaController` continua sendo a fachada; componentes não chamam o
  `CoreClient` diretamente;
- o catálogo de tool windows só monta/foca a superfície; não recebe estado do
  domínio;
- filtro, seleção e aba interna são estado efêmero da view;
- perfil, resultado e token continuam no controller/core atual;
- nenhuma store/DI/registry universal nasce desta fatia;
- novo método IPC só entra depois de provar uma lacuna que sinais atuais não
  conseguem representar honestamente.

## 10. Fatias para a 0.3.5

### G0 — baseline real

- capturar a UI atual em 1024×700 e 1280×800;
- medir gestos de primeira conexão e de reabertura;
- provar sem token, token de variável e token digitado;
- registrar comportamento de erro, reabertura e troca de workspace;
- repetir contra um Grafana real, não apenas falso.

### G1 — fluxo de conexão

- URL primeiro, autenticação sob demanda;
- ação Conectar com próximo passo único;
- resumo do que será e não será salvo;
- erros acionáveis, foco e teclado;
- implementar e provar o ciclo de vida do token decidido na §7.

### G2 — superfície diária

- `KvPanelHeader` e `KvVerdict` coerentes;
- setup recolhido depois de pronto;
- cruzamentos em primeiro plano;
- fontes e dashboards em `KvDataGrid` selecionável;
- filtro e abertura externa acessíveis por mouse e teclado;
- empty/loading/error/stale completos.

### G3 — integração ao shell

- contribuição ao modelo mínimo de tool window;
- command/paleta focam a mesma instância;
- estado não é duplicado no shell;
- se houver indicador compacto, ele mostra apenas fato medido e leva ao
  painel; não vira segunda HUD completa.

### G4 — prova e fechamento

- harness do controller e da máquina de estados;
- teste de tab order, Enter, Escape, setas e resize;
- nenhum token em perfil/log/screenshot de harness;
- teste real com URL inválida, sem token, token inválido e válido;
- foto e roteiro do fluxo diário;
- documentação descreve a integração somente de leitura existente.

## 11. Critérios de aceite

- primeira conexão não expõe configuração que a resposta ainda não pediu;
- instância já configurada abre direto no uso diário;
- fechar e reabrir segue a política de token escolhida, sem surpresa;
- trocar workspace/URL, esquecer ou receber rejeição invalida a credencial;
- URL alterada nunca deixa dados antigos parecerem atuais;
- origem e idade da última prova são visíveis;
- dashboard pode ser filtrado e aberto só com teclado;
- cruzamento com bancos do projeto é a informação de maior hierarquia;
- 1024×700 mantém estado, filtro e ação principal sem rolagem de descoberta;
- nenhum segredo é persistido ou exibido;
- a tarefa suportada é mais simples que compor chamadas da API no terminal;
- prova em Grafana real acompanha os falsos do gate;
- nada sugere edição, criação ou incorporação de dashboard que a IDE não faz.

## 12. Não entra por inferência

- editor de dashboard;
- provisioning de Grafana ou instalação silenciosa;
- Grafana embutido/webview;
- criação automática de datasource;
- armazenamento de token em texto ou cofre novo;
- alertas, Loki, Prometheus e métricas como novos domínios;
- varredura de rede;
- telemetria da IDE.

Essas possibilidades não são descartadas por incompreensão. Precisam de pedido,
contrato, segurança/licença e prova próprios; não são necessárias para tornar a
capacidade já existente realmente utilizável na 0.3.5.
