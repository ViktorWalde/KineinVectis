# Kinein Vectis — Embedded, Remote, SSH e QEMU

Ícones especializados para Linux embarcado, bare metal, deploy, serial, QEMU e debug remoto.


## 1. `kv.embedded.target` — Target

**Prioridade:** P0  
**Superfícies:** Embedded panel; top bar  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Representar destino de execução selecionado.

### Metáfora
Mira técnica.

### Construção geométrica
Dois círculos concêntricos e ponto central quadrado; quatro marcas cardeais curtas.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Local, SSH, QEMU, MCU ou container como modifier.

### Tooltip
```text
Target ativo
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 2. `kv.embedded.local_linux` — Linux local

**Prioridade:** P0  
**Superfícies:** Target selector  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Representar host Linux local.

### Metáfora
Monitor/terminal com pequeno kernel/pinguim abstrato não marcário.

### Construção geométrica
Monitor simples com prompt; pequeno ponto local no canto inferior.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Linux local
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 3. `kv.embedded.remote_linux` — Linux remoto

**Prioridade:** P0  
**Superfícies:** Target selector  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Representar target Linux acessado por rede.

### Metáfora
Monitor remoto ligado por dois nós.

### Construção geométrica
Monitor à direita e nó/host pequeno à esquerda conectados por linha.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Lock para host verificado; warning para host key.

### Tooltip
```text
Linux remoto
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 4. `kv.embedded.ssh` — SSH

**Prioridade:** P0  
**Superfícies:** Target; terminal; remote toolbar  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Abrir ou indicar conexão SSH.

### Metáfora
Terminal com chave.

### Construção geométrica
Janela de terminal; pequena chave de 6–7 px no canto inferior direito.

### Estados
disconnected, connecting, connected, reconnecting e failed.

### Modifiers e badges
Lock/open lock ou warning.

### Tooltip
```text
Conexão SSH
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 5. `kv.embedded.deploy` — Deploy

**Prioridade:** P0  
**Superfícies:** Remote toolbar; Run Config  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Enviar artefatos ao target.

### Metáfora
Pacote/binário com seta para host remoto.

### Construção geométrica
Cubo/arquivo à esquerda, seta horizontal e pequeno target à direita.

### Estados
idle, transferring, succeeded, failed e cancelled.

### Modifiers e badges
Progress ring ou check/x.

### Tooltip
```text
Enviar para o target
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 6. `kv.embedded.remote_run` — Executar remotamente

**Prioridade:** P0  
**Superfícies:** Remote toolbar  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Executar comando/binário no target remoto.

### Metáfora
Play dentro de host remoto.

### Construção geométrica
Retângulo de monitor com triângulo central e pequeno nó de rede.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Executar no target remoto
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 7. `kv.embedded.gdbserver` — gdbserver

**Prioridade:** P0  
**Superfícies:** Debug Profile; Remote toolbar  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Iniciar ou indicar servidor de debug remoto.

### Metáfora
Bug/debug ligado a porta de rede.

### Construção geométrica
Ícone debug à esquerda; tomada/porta circular à direita com linha curta.

### Estados
stopped, starting, listening, attached e failed.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
gdbserver remoto
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 8. `kv.embedded.serial` — Serial Monitor

**Prioridade:** P0  
**Superfícies:** Embedded panel; bottom panel  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Abrir monitor serial.

### Metáfora
Conector serial/USB com linhas de texto.

### Construção geométrica
Plug de 4 pinos simplificado à esquerda; três linhas de log à direita.

### Estados
closed, opening, connected, paused e error.

### Modifiers e badges
Número da porta apenas como texto próximo.

### Tooltip
```text
Monitor serial
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 9. `kv.embedded.qemu` — QEMU

**Prioridade:** P0  
**Superfícies:** Target selector; Embedded panel  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Representar máquina emulada pelo QEMU sem usar marca/logotipo.

### Metáfora
Máquina virtual dentro de uma moldura de chip.

### Construção geométrica
Chip retangular com pequeno monitor/CPU interno; bordas técnicas.

### Estados
stopped, booting, running, paused e failed.

### Modifiers e badges
GDB, serial ou display.

### Tooltip
```text
Target emulado
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 10. `kv.embedded.qemu_display` — Display QEMU

**Prioridade:** P0  
**Superfícies:** QEMU controls  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Abrir a janela/display da máquina emulada.

### Metáfora
Monitor com pequeno chip no canto.

### Construção geométrica
Monitor convencional; chip quadrado de 5–6 px como modifier.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Abrir display da máquina emulada
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 11. `kv.embedded.cross_compile` — Cross-compilation

**Prioridade:** P0  
**Superfícies:** Toolchain selector; Build  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Indicar compilação host → arquitetura alvo.

### Metáfora
Dois chips diferentes ligados por seta.

### Construção geométrica
Chip x86/genérico à esquerda, seta curta, chip alvo à direita; formas levemente diferentes.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Arquitetura deve aparecer em texto: ARM64, ARM, RISC-V.

### Tooltip
```text
Toolchain de cross-compilation
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 12. `kv.embedded.sysroot` — Sysroot

**Prioridade:** P0  
**Superfícies:** SDK/Toolchain settings  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Representar raiz de headers/libs do target.

### Metáfora
Árvore de diretórios dentro de target.

### Construção geométrica
Pasta raiz com dois ramos internos; pequeno alvo no canto.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Sysroot do target
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 13. `kv.embedded.sdk` — SDK

**Prioridade:** P0  
**Superfícies:** SDK profiles; First Run  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Representar SDK Yocto/Buildroot/vendor carregado.

### Metáfora
Caixa de ferramentas com chip.

### Construção geométrica
Maleta baixa; chip quadrado central; alça curta.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Check para ambiente carregado; warning para script inválido.

### Tooltip
```text
SDK
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 14. `kv.embedded.board` — Placa

**Prioridade:** P0  
**Superfícies:** Targets; Project templates  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Representar placa de desenvolvimento.

### Metáfora
PCB retangular com chip central e headers laterais.

### Construção geométrica
Retângulo 16×12; chip 5×5; três pinos em cada lado.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
USB, Wi-Fi, lock ou warning.

### Tooltip
```text
Placa de desenvolvimento
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 15. `kv.embedded.mcu` — Microcontrolador

**Prioridade:** P0  
**Superfícies:** Targets; Memory map  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Representar MCU/bare metal.

### Metáfora
Chip quadrado com núcleo central.

### Construção geométrica
Quadrado de 10–12 px, pinos nos quatro lados e pequeno quadrado interno.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Microcontrolador
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 16. `kv.embedded.probe` — Debug probe

**Prioridade:** P0  
**Superfícies:** Debug Profile; Devices  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Representar ST-Link, J-Link, CMSIS-DAP ou probe-rs sem usar marcas.

### Metáfora
Dongle USB ligado a ponta de probe.

### Construção geométrica
Corpo retangular curto, conector à esquerda e dois fios/pinos à direita.

### Estados
missing, detected, ready, busy e failed.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Probe de debug
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 17. `kv.embedded.flash` — Gravar firmware

**Prioridade:** P0  
**Superfícies:** Bare-metal toolbar  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Programar firmware na memória do target.

### Metáfora
Raio/seta entrando em chip.

### Construção geométrica
Chip MCU; seta diagonal preenchida entrando no centro.

### Estados
idle, erasing, writing, verifying, succeeded e failed.

### Modifiers e badges
Progress e check/x.

### Tooltip
```text
Gravar firmware
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 18. `kv.embedded.reset` — Resetar target

**Prioridade:** P0  
**Superfícies:** Bare-metal/QEMU controls  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Resetar dispositivo ou máquina virtual.

### Metáfora
Seta circular curta sobre chip.

### Construção geométrica
Chip pequeno com arco de 180–220° no topo; não usar refresh genérico isolado.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Resetar target
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 19. `kv.embedded.power` — Ligar/desligar target

**Prioridade:** P0  
**Superfícies:** Devices/QEMU controls  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Controlar energia virtual ou estado de sessão, não alimentação física arbitrária.

### Metáfora
Símbolo universal de power.

### Construção geométrica
Arco quase fechado e haste vertical central.

### Estados
off, on, transitioning e unavailable.

### Modifiers e badges
Nenhum por padrão.

### Tooltip
```text
Alternar estado do target
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---

## 20. `kv.embedded.port_forward` — Port forwarding

**Prioridade:** P0  
**Superfícies:** SSH advanced  
**Master:** 20/16 px  
**Implementação:** SVG simbólico próprio.

### Função
Criar túnel/encaminhamento de porta.

### Metáfora
Dois portais/portas ligados por seta.

### Construção geométrica
Dois colchetes retangulares; seta horizontal passando entre eles.

### Estados
default, hover, active, selected, focused e disabled.

### Modifiers e badges
L/R/D em texto adjacente para local, remote ou dynamic.

### Tooltip
```text
Encaminhamento de porta
```

### Atalho
```text
Nenhum atalho fixo.
```

### Evitar
Evitar detalhes decorativos, sombras e dependência exclusiva de cor.

---
