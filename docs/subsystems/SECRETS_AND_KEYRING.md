# Kernwerk Studio — Secrets, Keyring e Dados Sensíveis

> Este documento define como o Kernwerk deve tratar senhas, tokens, chaves, credenciais e informações sensíveis.

---

## 1. Objetivo

O Kernwerk vai lidar com:

```text
SSH;
Remote targets;
Git;
API keys de IA;
tokens;
possíveis senhas;
ambientes de SDK;
comandos remotos.
```

Regra máxima:

```text
Nunca salvar segredo em JSON local simples.
```

---

## 2. O que é segredo

```text
senha SSH;
senha sudo;
API key OpenAI/Anthropic/OpenRouter;
tokens Git;
private keys;
passphrases;
credenciais de registry;
credenciais de target remoto;
cookies;
certificados privados.
```

---

## 3. Onde não salvar

Proibido salvar segredos em:

```text
.kernwerk/*.json;
logs;
quality reports;
workspace.json;
toolchains.json;
run-configs.json;
history;
crash reports;
prompt de IA;
stdout/stderr persistente sem máscara.
```

---

## 4. Onde guardar

Preferências:

```text
SSH agent para SSH;
system keyring para tokens;
environment variables para API keys;
KWallet em KDE;
GNOME Keyring/libsecret;
secret-tool;
1Password/Bitwarden CLI futuramente se configurado pelo usuário.
```

No MVP:

```text
não implementar storage próprio de segredo;
usar SSH agent e variáveis de ambiente.
```

---

## 5. API keys

Configuração recomendada:

```text
OPENAI_API_KEY
ANTHROPIC_API_KEY
OPENROUTER_API_KEY
```

A IDE pode detectar se existe, mas não deve mostrar valor.

Mostrar:

```text
OPENAI_API_KEY: configured
ANTHROPIC_API_KEY: missing
```

Nunca mostrar:

```text
sk-...
```

---

## 6. Logs com máscara

Antes de logar comando/env, mascarar:

```text
*_TOKEN
*_KEY
*_SECRET
PASSWORD
PASS
AUTH
```

Exemplo:

```text
OPENAI_API_KEY=***
```

---

## 7. Remote SSH

Regras:

```text
preferir SSH key;
não salvar senha;
não salvar passphrase;
usar ssh-agent;
respeitar known_hosts;
não autoaceitar host desconhecido sem confirmação.
```

---

## 8. IA

Antes de enviar contexto para IA externa:

```text
mostrar arquivos/trechos;
permitir remover arquivos;
avisar se contém .env, secrets ou keys;
nunca enviar .env por padrão;
nunca enviar private key;
nunca enviar projeto inteiro sem confirmação.
```

---

## 9. Arquivos sensíveis ignorados

Nunca incluir automaticamente:

```text
.env
.env.*
id_rsa
id_ed25519
*.pem
*.key
*.p12
*.pfx
secrets.*
credentials.*
.kube/config
```

---

## 10. Comandos perigosos

Se um comando contém segredo, não persistir comando completo.

Exemplo:

```text
curl -H "Authorization: Bearer TOKEN"
```

Log:

```text
curl -H "Authorization: Bearer ***"
```

---

## 11. Critérios de aceite

MVP pronto quando:

```text
nenhum segredo salvo em .kernwerk;
logs mascaram variáveis sensíveis;
IA externa pede confirmação;
SSH usa agente externo;
API keys vêm de env;
settings mostram configured/missing, não valores.
```

---

## 12. Decisão final

Kernwerk deve ser privacy-first.

Segredo nunca deve virar conveniência perigosa.
