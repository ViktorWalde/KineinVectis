#!/bin/sh
# R10 num Xvfb e numa sessão D-Bus próprios: a sessão gráfica e o barramento
# de quem roda ficam de fora. O display é um número livre, acima de 90.
set -eu
aqui="$(cd "$(dirname "$0")" && pwd)"
log="$aqui/../tmp/a11y-sessao.log"
n=93
while [ -e "/tmp/.X11-unix/X$n" ] || [ -e "/tmp/.X$n-lock" ]; do n=$((n + 1)); done
Xvfb ":$n" -screen 0 1280x800x24 -nolisten tcp >/dev/null 2>&1 &
xvfb=$!
trap 'kill "$xvfb" 2>/dev/null || true' EXIT
for _ in $(seq 50); do [ -e "/tmp/.X11-unix/X$n" ] && break; sleep 0.1; done
export DISPLAY=":$n"
unset WAYLAND_DISPLAY
# O barramento de acessibilidade sobe com o dbus-broker, que ativa serviços
# pelo systemd; a sessão isolada não tem systemd, então o registro (o
# org.a11y.atspi.Registry) sobe à mão, depois do barramento.
dbus-run-session -- sh -c '
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
