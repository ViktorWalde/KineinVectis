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
enum class Acao
{
    /// Abrir `pasta` como workspace.
    Abrir,
    /// Nenhum caminho foi dado.
    ///
    /// NAO e' o mesmo que "abrir a pasta atual": a §3 da especificacao avisa
    /// que "abertura do desktop sem path nao deve tratar um CWD arbitrario
    /// como projeto". O atalho do menu roda o binario sem argumento, de um
    /// diretorio qualquer. Quem transforma isto em CWD e' o comando curto
    /// `kinein`, conforme a sugestao da §7 — e' decisao DELE, nao do binario.
    SemPasta,
    /// Escrever `mensagem` e sair com 0, sem subir UI nem core.
    Ajuda,
    /// Idem, com a versao.
    Versao,
    /// Escrever `mensagem` em stderr e sair com erro, sem tocar em disco.
    Recusa,
};

/// A decisao, ja' tomada.
struct Argumentos
{
    Acao acao = Acao::Abrir;
    /// Caminho ABSOLUTO quando `acao == Abrir`; vazio nas outras.
    QString pasta;
    /// Texto para a pessoa: ajuda, versao ou o motivo da recusa.
    QString mensagem;
};

/// Le' os argumentos (SEM o nome do programa) e decide. Parte LEXICA: resolve
/// caminho relativo contra `diretorioAtual` e nao toca no disco.
///
/// Sem caminho, devolve `SemPasta` — e nao o diretorio atual. Ver o porque na
/// documentacao daquele valor.
[[nodiscard]] Argumentos interpretar(const QStringList& argumentos, const QString& diretorioAtual);

/// Confere no DISCO se `pasta` serve como workspace.
///
/// Devolve vazio quando serve, ou a frase que diz o que esta' errado. Separado
/// do `interpretar` porque so' esta metade precisa de sistema de arquivos.
[[nodiscard]] QString validarPasta(const QString& pasta);

/// O texto de `--help`.
[[nodiscard]] QString textoDeAjuda();

} // namespace kinein::cli
