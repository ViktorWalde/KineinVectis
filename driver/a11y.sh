#!/bin/sh
# R10 num Xvfb, numa sessão D-Bus e num XDG_RUNTIME_DIR próprios: a sessão
# gráfica, o barramento e os sockets de quem roda ficam de fora. O display é
# um número livre, acima de 90.
set -eu
aqui="$(cd "$(dirname "$0")" && pwd)"
log="$aqui/../tmp/a11y-sessao.log"
mkdir -p "$aqui/../tmp"
# O barramento de acessibilidade e os sockets de cada aplicação vão para o
# XDG_RUNTIME_DIR; o de quem roda não recebe nada.
runtime="$(mktemp -d /tmp/kr-a11y-XXXXXXXX)"
n=93
while [ -e "/tmp/.X11-unix/X$n" ] || [ -e "/tmp/.X$n-lock" ]; do n=$((n + 1)); done
Xvfb ":$n" -screen 0 1280x800x24 -nolisten tcp >/dev/null 2>&1 &
xvfb=$!
trap 'kill "$xvfb" 2>/dev/null || true; rm -rf "$runtime"' EXIT
for _ in $(seq 50); do [ -e "/tmp/.X11-unix/X$n" ] && break; sleep 0.1; done
export DISPLAY=":$n" XDG_RUNTIME_DIR="$runtime"
unset WAYLAND_DISPLAY
# A sessão não ativa serviço nenhum (a11y-sessao.conf, sem pastas de
# serviço): sem isso, o pedido de configuração do Qt ativava o portal do
# sistema dentro dela. O barramento de acessibilidade e o registro (o
# org.a11y.atspi.Registry) sobem à mão, nessa ordem.
dbus-run-session --config-file="$aqui/a11y-sessao.conf" -- sh -c '
    /usr/lib/at-spi-bus-launcher --launch-immediately &
    launcher=$!
    sleep 1
    /usr/lib/at-spi2-registryd &
    registry=$!
    sleep 1
    python3 -I "$0/${A11Y_SCRIPT:-a11y.py}"
    rc=$?
    kill "$registry" "$launcher" 2>/dev/null || true
    exit $rc
' "$aqui" 2>"$log"
