# dfu.* — interfaces USB anunciadas pelo dfu-util

Desde o protocolo **0.193.0** (2026-10-26).

## `dfu.list {}` → `{interfaces, toolAvailable, complete, tool?, rawOutput, hint?}`

Exige workspace; parâmetros desconhecidos são INVALID_PARAMS. Consulta adiada
pelo dfu-util detectado no PATH, somente `--list`, prazo de 5 s e até 64 KiB
de saída retida. A ferramenta abre USB para ler descritores, sem detach,
reset, claim de interface ou gravação. Fonte e desenho: [52 §14.51](../../roadmaps/52-arquitetura-executavel-da-0.4.md#1451-041h4b2b4n1--descoberta-usb-do-dfu-util-antes-da-gravação-nativa-desenho-2026-10-26).

Cada entrada de `interfaces` corresponde a uma **alternativa USB**, com:

- `mode`: `"dfu"` ou `"runtime"`;
- `vid`, `pid`, `release`: quatro algarismos hexadecimais;
- `deviceNumber`, `configuration`, `interface`, `alternate`: inteiros de 8 bits;
- `path`, `name`: descritores textuais da ferramenta;
- `serial?`: ausente se vazio ou UNKNOWN.

Não agrupa alternativas como placas distintas nem transforma caminho/número
USB em identidade estável. Nenhum campo valida origem do firmware,
compatibilidade, acesso exclusivo ou autorização de gravação.

`complete=true` exige cabeçalho dfu-util 0.11, processo bem-sucedido e todas as
linhas no formato conferido. Aviso de acesso (mesmo com exit zero), versão ou
formato desconhecido, prazo, cancelamento e saída excessiva devolvem
`complete=false`, `interfaces=[]` e uma dica; a saída retida vai sempre junto.
Só falha ao iniciar deixa `toolAvailable=false`. `tool` informa o executável
consultado quando encontrado. Uma lista completa vazia informa somente que
nenhuma interface DFU foi listada nessa consulta.

Nova consulta e qualquer ativação/desativação do workspace cancelam a anterior,
inclusive reabrir a mesma raiz. A continuação recusa o token antigo ou mudança
de ferramenta/workspace com INVALID_REQUEST. O consumidor Qt/QML correlaciona
sucesso e erro por id e invalida a tela na desconexão/mudança de workspace.
Sem evento novo. Gravação nativa dfu-util continua recusada nesta versão.
