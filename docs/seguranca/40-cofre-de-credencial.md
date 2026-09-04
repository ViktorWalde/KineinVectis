# 40 — O cofre de credencial: o que foi medido, e a decisão que falta

> **Classe: DECISÃO PENDENTE** (`docs/README.md`). Este documento **não**
> descreve código que existe. Ele existe porque o
> [`../roadmaps/35`](../roadmaps/35-ambiente-cpp-embarcados-simulacao.md) §7.3
> registrou uma regra dura:
>
> > *"Nenhuma linha de conexão a banco entra antes dessa pergunta ter dono."*
>
> Toda medição abaixo foi feita em **2026-09-04**, com versão e fonte citadas,
> como o contrato exige de qualquer afirmação sobre comportamento de terceiros.

## 1. O risco, dito sem rodeio

Hoje o `.kinein/` do projeto guarda **rascunho e toolchain em texto puro**
(`crates/kinein-core/src/db/mod.rs` — SQLite sem cifra, por desenho: o conteúdo
ali é o próprio código do autor, que já está em texto no disco).

Senha de banco no mesmo lugar **não é a mesma coisa**. Seria:

```text
um segredo de OUTRO sistema, em texto puro, dentro de um diretorio que o
autor versiona por engano com facilidade, num arquivo que a IDE cria sozinha.
```

Isso é **regressão de segurança**, não feature. O projeto tem
[`23-rede-de-seguranca.md`](23-rede-de-seguranca.md) (rede contra *perda de
dado*) e **não tem** rede contra *vazamento de segredo*.

## 2. O que os profissionais fazem, e o modo de falha que eles pagam

A `integracoes/README.md` obriga estudar uma referência profissional antes de
cada feature de IDE, e **importar invariante, modo de falha e estratégia de
teste — nunca código**.

### 2.1 Code OSS / VS Code

Usa o **Secret Service** do freedesktop (GNOME Keyring, KWallet, KeePassXC) por
D-Bus, selecionável por `"password-store": "gnome-libsecret"` em `argv.json`.
Quando o keyring não está disponível, cai para um armazenamento "basic" cuja
chave é **codificada no fonte do Chromium**.

**A invariante que vale importar:** o sistema operacional é o dono do segredo; a
IDE guarda apenas um identificador.

**O modo de falha que vale importar, e é o mais caro:** o caminho "sem keyring"
não foi tratado como caminho de primeira classe. O resultado são anos de relatos
de usuário do tipo *"You're running in a GNOME environment but the OS keyring is
not available for encryption"* (microsoft/vscode
[#194530](https://github.com/microsoft/vscode/issues/194530),
[#189672](https://github.com/microsoft/vscode/issues/189672)) e detecção de
ambiente que ignora a escolha explícita do usuário
([#204552](https://github.com/microsoft/vscode/issues/204552)). O fallback
"basic" é pior que o erro: ele **parece** cifrado e não é.

**A estratégia de teste que isso implica:** o caminho SEM Secret Service tem de
ser exercitado no gate, não descoberto pelo autor. E ele nunca pode fingir
cifra.

### 2.2 DBeaver

Continua sendo a referência funcional de cliente de banco deste projeto
(registrada em [`../integracoes/37`](../integracoes/37-banco-e-observabilidade.md)
§4 — o IntelliJ Community **não** tem Database Tools; isso é Ultimate, verificado
em 2026-07-17).

### 2.3 O próprio PostgreSQL já resolve isto, e quase ninguém lembra

**Fonte: documentação oficial do PostgreSQL 18 (18.6, 2026-08-13),**
[`libpq-pgpass`](https://www.postgresql.org/docs/current/libpq-pgpass.html).

```text
~/.pgpass          hostname:port:database:username:password
PGPASSFILE         sobrepoe o caminho; parametro de conexao `passfile`
permissao 0600     OBRIGATORIA no Unix
```

**E o modo de falha é excelente, porque é seguro:** se a permissão for mais
frouxa que `0600`, o arquivo é **ignorado inteiro** — não é lido "com um aviso".
Falha fechada, que é o comportamento certo para segredo.

Ou seja: **existe um lugar padrão, com permissão verificada pelo próprio
driver, que a IDE não precisa inventar.**

## 3. O custo real de embutir um cofre, medido

Medido em **2026-09-04**, com a *toolchain* pinada do repositório (1.96.1) e o
`deny.toml` **deste** projeto:

```bash
cargo add secret-service --features rt-async-io-crypto-rust
cargo deny check licenses
cargo tree --prefix none --format "{p}" | sed 's/ (\*)//' | sort -u
```

```text
secret-service      5.2.0    MIT OR Apache-2.0   atualizado em 2026-08-29
keyring             4.2.0    MIT OR Apache-2.0   atualizado em 2026-08-29
                             (o wrapper multiplataforma; no Linux usa o de cima)

licencas             OK      as 97 crates da arvore passam no deny.toml do
                             repositorio, sem acrescentar uma linha a lista

workspace hoje       84 crates
arvore do cofre      97 crates
NOVAS de verdade     87 crates      -> o workspace vai a 171, mais que DOBRA
```

**As 87 novas não são "umas caixinhas de cripto".** São um runtime assíncrono
inteiro (`async-io`, `async-executor`, `async-task`, `async-process`,
`async-signal`, `blocking`, `polling`…), o `zbus` e a pilha `aes`/`cbc`/`hkdf`/
`sha2`. O core hoje é **síncrono por desenho** — `Core::new()` puro roda sem GUI
e sem runtime.

**A boa notícia técnica:** o `secret-service` 5.2.0 tem módulo `blocking`
(seguindo o `zbus`), então **não** seria preciso adotar `tokio` no core. A conta
de dependência continua igual, mas a arquitetura síncrona sobrevive.

**A ressalva de licença que evitamos por sorte, e vale registrar:** a variante
`dbus-secret-service` (mesma organização, API bloqueante) liga o **libdbus-1**
do sistema, que é `AFL-2.1 OR GPL-2.0+`. Nenhuma das duas está na lista do
`deny.toml`, e a GPL-2.0 linkada seria violação direta da política do projeto
(*"copyleft é EXECUTADO como processo, nunca linkado"*). A escolha do
`secret-service` puro-Rust não é preferência estética.

## 4. As três saídas, e o que cada uma custa

### (a) Não guardar segredo nenhum — a IDE delega

A IDE guarda o **perfil de conexão** (host, porta, banco, usuário) em
`.kinein/`, que **não é segredo**, e obtém a senha de:

```text
1. ~/.pgpass ou PGPASSFILE      mecanismo do proprio Postgres, 0600 verificado
2. variavel de ambiente         o que CI e container ja usam
3. prompt na hora, em MEMORIA   vive a sessao, nunca toca o disco
```

```text
CUSTO      zero dependencia nova, zero cripto escrita aqui
GANHO      a IDE nunca vira alvo: nao ha' arquivo dela para roubar
PREÇO      sem .pgpass configurado, o autor digita a senha por sessao
```

### (b) Secret Service do freedesktop, com o caminho vazio de primeira classe

```text
CUSTO      +87 crates (84 -> 171), licencas OK sem mexer no deny.toml
GANHO      conveniencia real: o sistema lembra, e o segredo fica no keyring
PREÇO      onde nao ha' Secret Service (WSL, servidor, WM minimo, container),
           o caminho vazio TEM que existir e NAO pode fingir cifra — e' o
           erro que o Code OSS paga ha' anos (§2.1)
```

### (c) Cofre próprio, cifrado com senha-mestra

```text
CUSTO      escrever/gerir cripto de segredo neste repositorio
GANHO      independe do ambiente
PREÇO      e' a pior das tres: cripto caseira e' risco que este projeto nao
           precisa correr, e senha-mestra so' move o problema um degrau
```

## 5. A recomendação, e ela não decide nada

**(a) primeiro, (b) depois como conveniência opcional.** A ordem importa:

1. **(a) desbloqueia a etapa 26 hoje**, sem dependência nova e sem risco. O
   perfil de conexão é dado comum; o segredo fica com quem já sabe guardá-lo.
2. **(b) fica melhor construído depois de (a)**, porque (a) obriga a existir o
   caminho "sem segredo guardado" — que na saída (b) sozinha é justamente o
   caminho esquecido, e é onde o Code OSS sangra.
3. **(c) está descartada**, salvo decisão explícita em contrário.

**O que a recomendação NÃO resolve, e por isso a decisão é sua:** com (a) pura,
o autor que **não** tem `.pgpass` digita a senha a cada sessão. Se isso for
atrito inaceitável no uso diário, (b) entra junto e as +87 crates entram com
ela.

## 6. O registro da decisão

```text
DECISAO   (a) DELEGAR agora, Secret Service depois — autor, 2026-09-04
```

**A escolha do autor foi a (a), com a (b) explicitamente adiada, não
descartada.** O que isso fixa, e vale como contrato para quem escrever a etapa
26:

```text
A IDE GUARDA      perfil de conexao: nome, host, porta, banco, usuario
A IDE NUNCA       guarda senha em disco. Nao ha' campo de senha no perfil.
A SENHA VEM DE    1. o prompt da sessao, vivendo SO' em memoria
                  2. a variavel de ambiente
                  3. o ~/.pgpass / PGPASSFILE do proprio Postgres
```

**O preço aceito, dito na hora de aceitar:** sem `.pgpass` configurado, o autor
digita a senha uma vez por sessão. Isso é atrito conhecido, não descuido — e é
o gatilho medido para a saída (b) entrar depois: se o registro de saídas do
dogfooding (`docs-privada/diario/19`) mostrar esse atrito, as +87 crates passam
a ter justificativa de uso, que hoje não têm.

**Por que a ordem importa, e não é só cautela:** construir (a) primeiro
*obriga* a existir o caminho "sem segredo guardado". Na saída (b) sozinha esse
é justamente o caminho esquecido — é onde o Code OSS sangra há anos (§2.1).
Quando a (b) entrar, ela entra como conveniência sobre um caminho que já
funciona e já é testado, não como o caminho único com um remendo embaixo.

**Critério de aceite, seja qual for a escolha** (ele não depende dela):

```text
1. nenhum segredo em `.kinein/` em texto puro — verificavel por teste
2. o caminho SEM segredo guardado e' exercitado no gate, nao descoberto no uso
3. nada neste repositorio finge cifrar o que nao cifrou
4. `cargo deny check licenses` continua verde SEM acrescentar licenca a lista
```
