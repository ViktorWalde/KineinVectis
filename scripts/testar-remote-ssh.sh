#!/usr/bin/env bash
# Fluxo REAL de Remote SSH contra um sshd de verdade, num container.
#
# Por que NAO esta' no `verificar.sh`: ele exige podman e, na primeira vez,
# REDE para montar a imagem. O gate e' offline por principio. Segue a mesma
# separacao que o AppImage ja' usa no repo: `verificar-*` e' a catraca barata
# e hermetica; `testar-*` e' a prova pesada que se roda de proposito.
#
# O que ele prova esta' em scripts/testar_remote_ssh.py. Em uma frase: que o
# `remote.*` funciona contra OpenSSH de verdade — host key, BatchMode recusando
# sem chave, `ssh -G`, `ssh-copy-id` instalando a chave com senha, e rsync.
#
# ISOLAMENTO: uma HOME temporaria com par de chaves e `~/.ssh/config` proprios.
# A `~/.ssh` do autor nao e' lida nem escrita. O container e' removido no fim,
# inclusive se algo falhar.
#
# Uso: bash scripts/testar-remote-ssh.sh
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

IMAGEM=kinein-sshd-teste
CONTAINER=kinein-sshd-$$
ALIAS=kinein-lab
USUARIO=kinein

if ! command -v podman >/dev/null 2>&1; then
    echo "erro: este teste precisa do podman (o repo ja' o usa no AppImage)." >&2
    exit 1
fi
if [ ! -x target/release/kinein-core ]; then
    echo "erro: falta target/release/kinein-core. Rode:" >&2
    echo "      cargo build --release -p kinein-core" >&2
    exit 1
fi

HOME_TESTE="$(mktemp -d)"
PROJETO="$(mktemp -d)"
limpar() {
    podman rm -f "$CONTAINER" >/dev/null 2>&1 || true
    rm -rf "$HOME_TESTE" "$PROJETO"
}
trap limpar EXIT

echo "== alvo SSH de verdade (podman) =="
if ! podman image exists "$IMAGEM"; then
    echo "montando a imagem $IMAGEM (precisa de rede uma vez)..."
    podman build -q -t "$IMAGEM" -f packaging/testes/Containerfile.sshd packaging/testes >/dev/null
fi

# Porta efemera escolhida pelo kernel: rodar o teste nao pode colidir com um
# sshd local nem com outra copia deste mesmo teste.
PORTA="$(python3 -c 'import socket;s=socket.socket();s.bind(("127.0.0.1",0));print(s.getsockname()[1]);s.close()')"
podman run -d --name "$CONTAINER" -p "127.0.0.1:$PORTA:22" "$IMAGEM" >/dev/null
echo "container $CONTAINER em 127.0.0.1:$PORTA"

# Esperar o sshd ACEITAR conexao, nao apenas o container existir.
pronto=""
for _ in $(seq 1 40); do
    if python3 -c "
import socket,sys
s=socket.socket(); s.settimeout(0.5)
sys.exit(0 if s.connect_ex(('127.0.0.1', $PORTA)) == 0 else 1)
"; then pronto=1; break; fi
    sleep 0.5
done
[ -n "$pronto" ] || { echo "erro: o sshd do container nao subiu." >&2; exit 1; }

install -d -m 700 "$HOME_TESTE/.ssh"
ssh-keygen -q -t ed25519 -N '' -f "$HOME_TESTE/.ssh/id_ed25519"
# O alias que a DESCOBERTA vai achar. `StrictHostKeyChecking no` porque o host
# key de um container novo muda a cada corrida; num alvo real quem aceita e' a
# pessoa, uma vez, no terminal.
cat > "$HOME_TESTE/.ssh/config" <<CFG
Host $ALIAS
    HostName 127.0.0.1
    Port $PORTA
    User $USUARIO
    IdentityFile $HOME_TESTE/.ssh/id_ed25519
    StrictHostKeyChecking no
    UserKnownHostsFile $HOME_TESTE/.ssh/known_hosts

Host *
    StrictHostKeyChecking no
    UserKnownHostsFile $HOME_TESTE/.ssh/known_hosts
    IdentityFile $HOME_TESTE/.ssh/id_ed25519
CFG
chmod 600 "$HOME_TESTE/.ssh/config"

# SHIM DE ISOLAMENTO, e so' isso. Medido em 2026-09-24: o OpenSSH NAO honra
# `$HOME` para achar `~/.ssh/config` nem `~/.ssh/known_hosts` — usa a base de
# senhas. Sem o shim, o `ssh-copy-id` desta prova pararia no prompt de host key
# e, ao responder, escreveria no `known_hosts` REAL do autor.
#
# O shim NAO altera a linha que o produto compoe: ele executa o `ssh` de
# verdade, acrescentando apenas onde guardar o host key. Um container novo tem
# host key nova a cada corrida; num alvo real quem aceita e' a pessoa, uma vez,
# no terminal — que e' exatamente o que a IDE promete.
install -d "$HOME_TESTE/bin"
cat > "$HOME_TESTE/bin/ssh" <<SHIM
#!/bin/sh
exec /usr/bin/ssh -o UserKnownHostsFile="$HOME_TESTE/.ssh/known_hosts" \
    -o StrictHostKeyChecking=no "\$@"
SHIM
chmod +x "$HOME_TESTE/bin/ssh"
export PATH="$HOME_TESTE/bin:$PATH"

python3 -u scripts/testar_remote_ssh.py "$HOME_TESTE" "$PORTA" "$PROJETO"
