# Kinein Vectis 0.3.5 — Public Beta (Linux x86_64)

Fechamento da série 0.3. A série 0.x continua beta: interfaces, configurações e
fluxos podem mudar até a 1.0.

## Downloads

- `KV0.3.zip`: pasta `KV0.3/` com AppImage, `.sha256`, `SHA256SUMS`,
  instalador `instalar-kinein-vectis.sh`, `Tutorial.md` e estas notas.
- `Kinein-Vectis-0.3.5-x86_64.AppImage` e `Kinein-Vectis-0.3.5-x86_64.AppImage.sha256`
  avulsos.

```bash
sha256sum -c Kinein-Vectis-0.3.5-x86_64.AppImage.sha256
chmod +x Kinein-Vectis-0.3.5-x86_64.AppImage
./Kinein-Vectis-0.3.5-x86_64.AppImage
```

SHA-256 do AppImage: `c2710023928767f39b583c1e56fc946ee46f977985fbd08930229d991a33ecf7`

Requisitos: Linux x86_64, glibc 2.36+, Wayland ou X11. O pacote traz Qt/QML e o
core; compiladores, SDKs e ferramentas do projeto continuam sendo do sistema.

## O que muda desde a 0.2.0

- **Terminal:** ergonomia de shell/TUI validada, limpar histórico, Selecionar
  Tudo sobre o buffer retido.
- **Remote SSH:** aliases do `~/.ssh/config`, configurar servidor sem
  formulário, painel em seções, escolher a pasta remota **navegando desde a
  home** do alvo (inclusive pastas com espaço), espelho por `rsync` provado contra `sshd` real.
- **Projeto e arquivos:** comando `kinein` (abre a pasta atual; uma janela por
  workspace), árvore com teclado, seleção múltipla, menu por teclado,
  copiar/recortar/colar arquivos, transferências por Jobs, colisões sem
  sobrescrita, arrastar de e para o gerenciador de arquivos, lixeira do sistema.
- **Arquivo externo no editor:** soltar um arquivo de fora do projeto abre uma
  aba somente leitura, sem importar nem escrever na origem.
- **Abas e Markdown:** identidade estável das abas, preview Markdown lado a lado.
- **Editor:** símbolos do workspace e indentação pela gramática.
- **Observabilidade (Grafana):** painel refeito, cruzamento com os bancos do
  projeto, recusa de credencial com nome próprio.
- **Janela:** cabeçalho e bordas nativos (menu, maximizar, mover, redimensionar).
- Compatibilidade: workspace, sessão e configurações da 0.2 reabrem sem perda.

Detalhes, datas e provas: `CHANGELOG.md` e `DocsPublic/roadmaps/40-estado-e-continuidade.md`.

Comunidade e issues: https://discord.gg/cWRkUGUmQU · https://github.com/ViktorWalde/KineinVectis/issues
