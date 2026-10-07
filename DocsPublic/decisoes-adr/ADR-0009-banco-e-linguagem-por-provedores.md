# ADR-0009 — Banco e linguagem por provedores independentes

> **Status:** decisão de arquitetura registrada; implementação pendente.
> **Data:** 2026-10-07. Base `33463a1`, protocolo atual `0.163.0`.

## Contexto

O autor tornou LSP SQL obrigatório no passo 7 e pediu ferramentas existentes
para MongoDB moderno, SQLite e InfluxDB 3 nativo, com expansão futura.
Instalação e atualização de banco/LSP ficam a cargo do usuário. Uma versão
moderna usada em projetos existentes e outra recente para projetos novos
devem poder coexistir; não há requisito de servidor MongoDB 3.x.

O código atual tem quatro motores em enum fechado e um cliente LSP funcional
para C/C++, Rust e Python. Seus processos são selecionados por linguagem/
extensão, não por conexão. Repetir o cliente ou gerar um analisador SQL na
IDE duplicaria ferramentas existentes; compartilhar uma conexão global
de linguagem misturaria catálogo, credenciais e respostas entre consoles.

## Decisão

Criar dois registros no domínio Banco: adaptadores de acesso e provedores
de linguagem. Compatibilidade é uma associação entre motor, dialeto,
capacidades e provedor, com preferência explícita do usuário. Um provedor
pode atender vários bancos; a instância de processo e seu estado continuam
isolados por workspace/conexão/configuração.

Reutilizar transporte, handshake, sincronização e apresentação LSP atuais.
Executar a ferramenta original como processo externo, instalado/atualizado
pelo usuário, sem host VS Code/Neovim nem ABI Rust dinâmica. Descritores têm
IDs estáveis e operações tipadas. A migração preserva valores/perfis/vínculos
atuais; perfil de provedor indisponível permanece visível e intacto.

O core é dono da seleção, compatibilidade, snapshot de catálogo e ciclo das
instâncias. A UI apresenta descritores/estado e rejeita respostas antigas.
Banco conserva autorização, segredo, leases, jobs, impacto e confirmação.
LSP de Banco não executa comandos do usuário ou ações equivalentes.
Segredos e conexão dependente de catálogo exigem adaptação comprovada.

Atualização de ferramenta negocia recursos e invalida só a instância afetada.
Não prender versões de servidor/LSP ao número usado na bateria de aceitação.
Incompatibilidade de protocolo/driver deve produzir erro localizado, com
perfil e editor preservados. As bibliotecas nativas compiladas no core ainda
exigem atualização do core quando seu próprio suporte mudar; o desenho não
promete compatibilidade com mudanças arbitrárias de protocolos futuros.

InfluxDB 3 usa API nativa, com capacidades de SQL/InfluxQL separadas de escrita
e administração. Provedor de linguagem compatível ainda precisa ser
selecionado e provado; Flux LSP arquivado não atende esse alvo. PostgreSQL,
SQLite e MongoDB têm candidatos com provas externas delimitadas, ainda sem
integração no produto. Detalhes, fontes, licenças, donos e fatias estão no
[desenho 38](../arquitetura/38-provedores-de-banco-e-linguagem.md).

## Alternativas e consequências

Um LSP por marca duplicaria seleção sem provar dialeto. Um único LSP SQL
universal não cobre automaticamente MongoDB e InfluxQL. Completion própria
e um novo parser aumentariam manutenção que o autor pediu evitar. Um host
genérico de extensões importaria dependências desnecessárias ao domínio.

A opção adotada requer adaptadores pequenos para configuração e diferenças
de protocolo, testes de compatibilidade e gerenciamento de processos. Um
LSP que exige catálogo vivo pode abrir conexão de metadados própria; não
confundir isso com reutilizar um snapshot que ele não sabe importar.
Ferramenta sem contrato suficiente permanece indisponível, com motivo.

## Critérios e reversão

Provar conexões simultâneas, troca de ferramenta, respostas atrasadas,
encerramento dos processos auxiliares, preservação de perfis desconhecidos
e comportamento de duas versões modernas do banco. Extensão de suporte
deve acrescentar descritor/adaptador/prova sem ramos por marca no editor.
Cada integração exige contrato aditivo, gates e prova na IDE.

O registro inicia sobre os quatro motores atuais. Pode-se retirar um
provedor de linguagem sem retirar seu driver nem perder consoles. A
migração de perfil só grava depois da validação integral e não elimina
dados que um adaptador ausente ainda não entende. O passo 7 só fecha com
os critérios do [59](../roadmaps/59-fechamento-da-0.3.9.md), não com D0.
