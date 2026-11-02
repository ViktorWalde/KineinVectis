# Plano executável antes das próximas etapas — 0.3.5

> **Classe: LOG.** Critérios históricos da entrega 0.3.5, definidos em
> 2026-10-01. O estado vigente está no [40](40-estado-e-continuidade.md);
> os resultados datados estão no [40.7](40.7-registro-das-entregas.md).

## Escopo e estado de entrada

A entrega abrangia navegação de pastas remotas, integração desktop,
validação do AppImage e documentação de uso. O protocolo anterior ao
recorte Remote era `0.142.0`; um pacote produzido antes da última alteração
de código não comprova a versão final.

## Etapa 1 — fechar a escolha da pasta remota começando na home

`remote.directories` permite iniciar na home do alvo, consultar filhas,
subir ao diretório pai e abrir a pasta escolhida como espelho. O contrato
reutiliza `remote::ssh_args`, quoting POSIX, jobs canceláveis e o controller
Remote existente; inclui `requestedPath` para rejeitar respostas atrasadas.
A saída é delimitada por NUL e limitada pelo core.

**Aceite:** escolher uma pasta desde a home, abrir o espelho e apresentar
falhas SSH sem trocar a seleção por resposta atrasada. Testes simulados
cobrem parsing e contexto; `scripts/testar-remote-ssh.sh` valida `sshd` real.

**Resultado de 2026-10-01 (40.7 §7.143):** o teste real navegou desde
`/home/kinein` e abriu o espelho por `rsync`. A validação corrigiu o
`ControlPath` longo. Quando o caminho do socket ultrapassa a margem, o core
usa SSH comum; os dois ramos têm testes em `remote/mirror.rs`.

## Etapa 2 — prova final de integração P3/H0

**Critérios:** drop externo abre uma aba somente leitura, preserva o arquivo
fonte e não importa conteúdo silenciosamente; arrasto para o gerenciador de
arquivos e recuperação pela Lixeira exigem provas próprias.

**Resultado de 2026-10-01 (40.7 §7.144–§7.147):** drop externo provado em
X11 e Wayland, com receptores Qt e GTK4. Corrigido o indicador de modificação
na troca de aba, com harness por mutação. O arrasto para Nautilus e a
recuperação pela Lixeira foram validados; Remote passou a aceitar pastas
com espaços no protocolo `0.144.0`.

## Etapa 3 — gates e artefato final V8

1. Executar o gate completo depois da última edição de código: Rust, C++,
   QML, fiação, arquitetura, documentação e primeiro frame Debug/Release.
2. Gerar o AppImage pelo empacotador portátil e validar o artefato com
   `scripts/testar-appimage.sh` e `scripts/testar-appimage-portatil.sh`.
3. Conferir SHA-256, conteúdo, versões Cargo/CMake/AppStream/Sobre,
   documentação de uso e notas da entrega.

**Aceite:** gate verde, artefato gerado a partir do código validado, smoke
em Debian mínimo sem SDK e regressões Remote/0.2 verificadas.

## Etapa 4 — pasta `KV0.3`, commit e push

A estrutura histórica `KV0.3/` agrupava AppImage, checksum, instalador,
tutorial e notas da entrega. Checksums devem ser conferidos depois de cada
cópia. Fontes versionadas e artefatos de distribuição têm ciclos distintos;
`target/`, `build/` e `dist/` permanecem fora do código versionado.

## Etapa 5 — release pública e site

Uma distribuição precisa de versão identificável, notas coerentes,
AppImage e checksum verificáveis. Links de download e documentação do site
devem apontar para o artefato validado. Este registro histórico não atesta
disponibilidade atual de release ou hospedagem.

## Ordem e disciplina de documentação

Requisitos e contratos precedem implementação; o resultado técnico registra
método, cobertura e limitações. O plano vigente da série está nos roadmaps
[57](57-mapa-de-versoes-ate-a-1.0.md) e [52](52-arquitetura-executavel-da-0.4.md).
