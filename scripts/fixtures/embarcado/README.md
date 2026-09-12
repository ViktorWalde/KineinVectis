# Fixture de embarcado: o menor firmware que prova o ciclo

Tres arquivos e nada mais — um Cortex-M3 bare-metal para a maquina
`lm3s6965evb` do QEMU (Stellaris LM3S6965: 256 KiB de flash em `0x0`,
64 KiB de SRAM em `0x20000000`; e' a maquina que o livro do rust-embedded usa
no capitulo de QEMU, e existe no `qemu-system-arm` 10.2.2 desta maquina).

```text
link.ld     a memoria e as secoes; nada de placa real
startup.c   vector table minima (sp, Reset, NMI, HardFault) e o copy/zero
main.c      `volatile int contador` que so' incrementa
```

**E' NOSSA, nao do usuario.** A regra da frente F (`DocsPublic/roadmaps/35` §5.6)
e' que a IDE nao adivinha linker script nem startup — e continua nao
adivinhando: esta fixture existe para o `scripts/verificar-embarcado.sh`
provar que o ciclo `debug.start` -> `attach` a um servidor que a IDE sobe ->
breakpoint -> variavel funciona, sem placa.

Compila com:

```bash
arm-none-eabi-gcc -mcpu=cortex-m3 -mthumb -g -O0 -nostartfiles -ffreestanding \
  -T link.ld startup.c main.c -o fixture.elf
```
