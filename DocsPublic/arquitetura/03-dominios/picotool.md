# picotool.* — consulta de RP2040 em BOOTSEL

Desde o protocolo **0.196.0** (2026-10-27).

## `picotool.list {}` → `{devices, complete, toolAvailable, tool?, version?, rawOutput, hint?}`

Exige workspace; campos desconhecidos são INVALID_PARAMS. Consulta explícita,
adiada e cancelável, apenas picotool 2.2.0/2.3.1, RP2040 BOOTSEL 2e8a:0003.
Executa `version`, `info --device --vid 0x2e8a --pid 0x0003` e confirma cada
candidato com `--ser ID`, sem -f/-F, load, erase, reboot ou escrita OTP.

Prazo total de 5 s, 64 KiB de saída retida no conjunto, até 16 dispositivos.
Cada entrada: `{chip:"rp2040", serialSelector, flashSizeBytes}`. O seletor
são 16 hexadecimais maiúsculos, não zero/ffff; confirmação precisa retornar
um único RP2040 com mesmo ID e tamanho. Tamanho reportado pelo picotool, até
16 MiB. Não inventa bus/address quando a ferramenta não os informa. Seletor
não é autenticação ou prova de identidade imutável; não equivale a tty.

`complete=true` só após versão, saída e todas as confirmações válidas. Avisos,
permissão, dados/versão desconhecidos, duplicatas, prazo, cancelamento e erro
invalidam o conjunto (`devices=[]`, complete=false, hint). Ferramenta ausente ou
falha de lançamento reportada pelo runner usa toolAvailable=false; true não
prova execução correta ou acesso USB. Saída sempre retida. Ausência de
BOOTSEL acessível é erro da ferramenta, sem lista vazia falsamente verificada.

Como a consulta faz claim da interface e usa comandos de leitura picoboot,
reserva seleção automática pelo registro existente; conflito pode retornar
SERIAL_PORT_BUSY. Consulta nova, workspace (inclusive mesma raiz) e shutdown
cancelam a anterior. Continuação antiga ou ferramenta/raiz alterada responde
INVALID_REQUEST. Substitutas compartilham a reserva e uma seção exclusiva USB;
espera com prazo/cancelamento fora do laço impede sobreposição, inclusive com
três ou mais pedidos rápidos. Qt/QML correlacionam sucesso e erro por id e limpam estado
na desconexão/workspace. Nenhum evento novo. A gravação nativa da 0.197.0 usa runConfig/run,
com o [contrato UF2](runConfig.md#picotool-nativo--uf2-conferido-com-elf-cmake-01970-2026-10-27). Desenho/fontes: [52 §14.54](../../roadmaps/52-arquitetura-executavel-da-0.4.md#1454-041h4b2b4o1--consulta-picotool-rp2040-antes-da-gravação-desenho-2026-10-27).
