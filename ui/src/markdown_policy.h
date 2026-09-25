#pragma once

#include <QString>

// A POLITICA DO PREVIEW DE MARKDOWN: o que um documento pode alcancar.
//
// A §5 da `especificacoes/markdown-preview-0.3.md` e' explicita: o preview NAO
// e' navegador. Sem JavaScript, sem cookies, sem WebEngine, sem execucao de
// bloco de codigo, e "sem leitura fora do workspace por URL construida no
// documento". Abrir documentacao nao pode virar acesso a' rede nem leitura de
// arquivo arbitrario — um `.md` e' conteudo de terceiros como qualquer outro.
//
// Aqui e' DECISAO PURA: entra texto, sai veredito. Nenhuma leitura de disco,
// nenhuma rede. Quem age e' a ponte do documento, e por isso isto tem teste.
//
// LIMITE DITO: normalizar caminho nao resolve symlink. Um link dentro do
// workspace apontando para fora passa por aqui como permitido; quem le' o
// arquivo confere o caminho CANONICO antes de entregar os bytes. Os dois
// controles existem porque cada um pega o que o outro nao pega.
namespace kinein::markdown {

enum class LinkKind
{
    Anchor,    // `#secao` — navega dentro do proprio preview
    LocalFile, // abre no editor da IDE
    Web,       // abre no navegador do sistema
    Refused,   // nada acontece, e o motivo e' dito
};

struct LinkDecision
{
    LinkKind kind = LinkKind::Refused;
    QString target; // ancora, caminho absoluto normalizado, ou URL
    QString reason; // preenchido so' quando recusado
};

// `href` como esta' escrito no documento; `documentPath` e' o `.md` aberto;
// `workspaceRoot` pode ser vazio (nenhum projeto aberto), e nesse caso
// qualquer caminho local e' recusado — sem projeto nao ha' escopo que autorize.
[[nodiscard]] LinkDecision decideLink(const QString& href, const QString& documentPath,
                                      const QString& workspaceRoot);

struct ResourceDecision
{
    bool allowed = false;
    QString path;   // caminho absoluto normalizado, quando permitido
    QString reason; // por que nao, quando negado — vai para o alt text
};

// Imagens. `remoteAllowed` existe para a preferencia futura da §4.2; na 0.3 e'
// sempre `false`, e o padrao e' bloquear para nao buscar recurso remoto ao
// apenas abrir um arquivo.
[[nodiscard]] ResourceDecision decideResource(const QString& href, const QString& documentPath,
                                              const QString& workspaceRoot, bool remoteAllowed);

// `true` quando o caminho normalizado esta' dentro da raiz — a mesma regra que
// o link e o recurso usam, exposta porque quem le' o arquivo a repete sobre o
// caminho canonico.
[[nodiscard]] bool insideRoot(const QString& absolutePath, const QString& root);

} // namespace kinein::markdown
