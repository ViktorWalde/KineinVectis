// O CONTRATO DA LINHA DE COMANDO da UI (P0 de
// `especificacoes/projetos-arquivos-e-integracao-desktop-0.3.md`).
//
// Por que existe como unidade propria: ate' 2026-09-24 o argumento de pasta era
// varrido dentro do `CoreClient::start()` — pegava o primeiro argumento que
// fosse uma pasta existente e IGNORAVA EM SILENCIO qualquer outra coisa. Um
// caminho errado de digitacao abria a IDE sem projeto e sem dizer por que, o que
// a §3 daquela especificacao lista como defeito ("path invalido/inacessivel ->
// erro util, sem criar pasta nem substituir o projeto atual silenciosamente").
//
// Aqui nao ha' Qt GUI, nao ha' QML e nao ha' IO na parte lexica: e' isso que
// torna esta unidade a primeira do projeto coberta por teste C++.
#pragma once

#include <QString>
#include <QStringList>

namespace kinein::cli {

/// O que a linha de comando pediu.
enum class Action
{
    /// Abrir `folder` como workspace.
    Open,
    /// Nenhum caminho foi dado.
    ///
    /// NAO e' o mesmo que "abrir a pasta atual": a §3 da especificacao avisa
    /// que "abertura do desktop sem path nao deve tratar um CWD arbitrario
    /// como projeto". O atalho do menu roda o binario sem argumento, de um
    /// diretorio qualquer. Quem transforma isto em CWD e' o comando curto
    /// `kinein`, conforme a sugestao da §7 — e' decisao DELE, nao do binario.
    NoFolder,
    /// Escrever `message` e sair com 0, sem subir UI nem core.
    Help,
    /// Idem, com a versao.
    Version,
    /// Escrever `message` em stderr e sair com erro, sem tocar em disco.
    Refusal,
};

/// A decisao, ja' tomada.
struct Arguments
{
    Action action = Action::Open;
    /// Caminho ABSOLUTO quando `action == Open`; vazio nas outras.
    QString folder;
    /// Texto para a pessoa: ajuda, versao ou o motivo da recusa.
    QString message;
    /// `--wait`/`-w`: o terminal fica preso ate' a janela fechar (como
    /// `code --wait`), para quem usa a IDE como editor de outro programa.
    bool waitForClose = false;
    /// `--verbose`: o terminal continua ligado a IDE e mostra o stderr dela,
    /// para diagnostico.
    bool verbose = false;
};

/// Le' os argumentos (SEM o nome do programa) e decide. Parte LEXICA: resolve
/// caminho relativo contra `diretorioAtual` e nao toca no disco.
///
/// Sem caminho, devolve `NoFolder` — e nao o diretorio atual. Ver o porque na
/// documentacao daquele valor.
[[nodiscard]] Arguments parse(const QStringList& arguments, const QString& currentDirectory);

/// Confere no DISCO se `folder` serve como workspace.
///
/// Devolve vazio quando serve, ou a frase que diz o que esta' errado. Separado
/// do `parse` porque so' esta metade precisa de sistema de arquivos.
[[nodiscard]] QString validateFolder(const QString& folder);

/// O texto de `--help`.
[[nodiscard]] QString helpText();

/// Se o processo do TERMINAL deve soltar a IDE e devolver o prompt, como
/// `code .` (roadmap 53 §5.1). So' quando quem chamou e' um terminal de verdade
/// (`stderrEhTerminal`): o atalho do menu, um script ou o gate, que leem o
/// stderr, continuam com o processo como sempre. `jaDesacoplado` impede um
/// segundo desacoplamento no filho.
[[nodiscard]] bool shouldDetach(const Arguments& request, bool stderrIsTerminal,
                                bool alreadyDetached);

} // namespace kinein::cli
