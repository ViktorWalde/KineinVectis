# CLAUDE.md — para o agente de IA que vai trabalhar aqui

Kinein Vectis é uma IDE desktop (core em Rust, UI em Qt/QML com uma camada
C++) para C/C++, Rust, Python e embarcados. A documentação é em português, e
os identificadores do código em inglês (gate). Este arquivo só aponta o
caminho; o conteúdo mora nos donos, e eles vencem se divergirem daqui.

## Leia, nesta ordem (e só isto antes da fatia)

1. [`DocsPublic/00-comece-aqui.md`](DocsPublic/00-comece-aqui.md) §2: o que
   é o projeto e as regras.
2. O handoff e o §4 do
   [`DocsPublic/roadmaps/40-estado-e-continuidade.md`](DocsPublic/roadmaps/40-estado-e-continuidade.md):
   o estado de hoje, a próxima fatia e o que espera o autor; o §5 traz as
   decisões dele, que não se reabrem.
3. A linha do contexto da mudança no 00 §5.1: a seção dele no
   `arquitetura/01`, os documentos da linha e só os domínios dela no
   `arquitetura/03`.
4. O roadmap da versão em curso (o 40 diz qual) e
   [`DocsPublic/contribuindo/05-com-um-agente-de-ia.md`](DocsPublic/contribuindo/05-com-um-agente-de-ia.md).

O registro (`roadmaps/40.7` e `roadmaps/registro/`) só se lê pela entrada
citada. Ler tudo gasta a sessão antes do código; começar pelo código repete o
que já foi descartado.

## As regras que não se "otimizam" (o detalhe e a origem estão no 05)

```text
- todo contexto vai para a documentação ANTES do código
- medir antes de afirmar; número com data; foto antes e depois na tela
- contrato primeiro: PROTOCOL_VERSION sobe e o arquitetura/03 muda antes do core
- catracas de tamanho: Rust 500 / QML visual 300 / controller-host 400 / ui/src 500
- um commit por fatia, com o gate rápido verde e `cargo fmt`; a mensagem
  termina com "Provou: ..." e "Não fez: ..."
- NUNCA `git checkout <arquivo>` para desfazer: cópia, stash ou edição
- nada de push, tag, release ou AppImage sem o autor pedir
- sem sudo e sem instalar nada em silêncio: dê o comando ao autor
- nunca gravar firmware na placa do autor sem pedido explícito
- não compilar a UI enquanto o verificar-cpp.sh roda
- decisão do autor não se reabre; proposta não é decisão
```

## Os comandos

```text
export PATH="$HOME/.cargo/bin:$PATH"
bash scripts/verificar.sh --rapido        o gate de cada fatia (~35 min)
bash scripts/verificar.sh --completo      antes de fechar uma versão
python3 scripts/module_map.py             regenera o arquitetura/01
python3 scripts/check_identifier_language.py   identificadores em inglês
```

Os demais, e o ambiente: `DocsPublic/build/comandos-de-build-e-verificacao.md`
e `DocsPublic/contribuindo/02-preparar-o-ambiente.md`.
