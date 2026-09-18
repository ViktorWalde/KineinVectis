// O que o core manda SEM SER PERGUNTADO: as notificacoes `event.*`.
//
// Separado do `core_client_dispatch.cpp` em 2026-09-03, e a costura nao foi
// inventada aqui — o `arquitetura/04` §5 ja a nomeia com todas as letras:
// "resposta e' consequencia de um PEDIDO; evento e' consequencia do MUNDO".
// Dois fatos diferentes, dois arquivos.
//
// A diferenca tem consequencia pratica registrada na §6 do mesmo documento: um
// evento pode chegar ENTRE um pedido e a resposta dele, e por isso o cliente
// nao pode supor que a proxima linha apos um pedido e' a resposta. Quem casa
// por `id` e o dispatch; quem so' reage e este arquivo.
#include "core_client.h"

#include <QJsonArray>

namespace kinein {

void CoreClient::handleNotification(const QString& method, const QJsonObject& params)
{
    if (handleFileSystemNotification(method, params)) {
        return;
    }
    if (method == QStringLiteral("event.datasource.introspected")) {
        emit dataSourceIntrospected(
            params.value(QStringLiteral("name")).toString(),
            params.value(QStringLiteral("ok")).toBool(false),
            params.value(QStringLiteral("schemas")).toArray().toVariantList(),
            params.value(QStringLiteral("collections")).toArray().toVariantList(),
            params.value(QStringLiteral("message")).toString(),
            params.value(QStringLiteral("secretRequired")).toBool(false));
        return;
    }
    if (method == QStringLiteral("event.datasource.queried")) {
        // columns, rows (celulas em texto, null = NULL), affected, truncated,
        // elapsedMs, message e secretRequired viajam juntos: e' uma tabela.
        emit dataSourceQueried(params.toVariantMap());
        return;
    }
    if (method == QStringLiteral("event.datasource.tested")) {
        emit dataSourceTested(params.value(QStringLiteral("name")).toString(),
                              params.value(QStringLiteral("ok")).toBool(false),
                              params.value(QStringLiteral("serverVersion")).toString(),
                              params.value(QStringLiteral("message")).toString(),
                              params.value(QStringLiteral("secretRequired")).toBool(false));
        return;
    }
    if (method == QStringLiteral("event.datasource.created")) {
        emit dataSourceCreated(params.value(QStringLiteral("success")).toBool(false),
                               params.value(QStringLiteral("profile")).toObject().toVariantMap(),
                               params.value(QStringLiteral("message")).toString());
        return;
    }
    if (method == QStringLiteral("event.grafana.probed")) {
        // Um mapa inteiro em vez de oito parametros: o resultado da sonda e'
        // composto, e desmontar aqui so' obrigaria a UI a remontar.
        emit grafanaProbed(params.toVariantMap());
        return;
    }
    if (method == QStringLiteral("event.index.progress")) {
        emit indexProgressed(params.value(QStringLiteral("files")).toInt(),
                             params.value(QStringLiteral("symbols")).toInt());
        return;
    }
    if (method == QStringLiteral("event.index.finished")) {
        emit indexFinished(params.toVariantMap());
        return;
    }
    if (method == QStringLiteral("event.serial.identified")) {
        // Mapa inteiro (identity, target, raw, error viajam juntos): a tela
        // mostra o que o esptool leu e a sugestao de kit — aplicar e' clique.
        emit serialIdentified(params.toVariantMap());
        return;
    }
    if (method == QStringLiteral("event.serial.files")) {
        // Idem: a listagem, o erro do mpremote e a saida crua viajam juntos.
        emit serialFilesResolved(params.toVariantMap());
        return;
    }
    if (method == QStringLiteral("event.project.changed")) {
        // O core recomputa o modelo ao abrir o workspace e ao fim de um
        // configure/build; a tela SEGUE o modelo em vez de perguntar.
        emit projectChanged(params.toVariantMap());
        return;
    }
    if (method == QStringLiteral("event.test.discovered")) {
        emit testsDiscovered(params.toVariantMap());
        return;
    }
    if (method == QStringLiteral("event.toolchain.installed")) {
        emit toolchainInstalled(params.toVariantMap());
        return;
    }
    if (method == QStringLiteral("event.python.finished")) {
        // jobId, success, tool, command e path viajam juntos: a tela diz o que
        // rodou e se o ambiente existe, e pede o status de novo.
        emit pythonEnvironmentFinished(params.toVariantMap());
        return;
    }
    if (method == QStringLiteral("event.coverage.finished")) {
        // success, tool, path, files (resumo por arquivo) e error viajam juntos.
        emit coverageFinished(params.toVariantMap());
        return;
    }
    if (method == QStringLiteral("event.remote.probed")) {
        // name, success, arch, kernel, tools[{id,found,path}], error e raw.
        emit remoteProbed(params.toVariantMap());
        return;
    }
    if (method == QStringLiteral("event.remote.deployed")) {
        // name, success, source, dest, command e error.
        emit remoteDeployed(params.toVariantMap());
        return;
    }
    if (method == QStringLiteral("event.remote.synced")) {
        // name, direction, success, command, changed[], error e mirror.
        emit remoteSynced(params.toVariantMap());
        return;
    }
    if (method == QStringLiteral("event.python.stubs")) {
        // jobId, success, package, command e target: a tela diz e pede o status.
        emit pythonStubsFinished(params.toVariantMap());
        return;
    }
    if (method == QStringLiteral("event.container.finished")) {
        // Mapa inteiro pelo mesmo motivo do grafana.probed: jobId, acao, alvo,
        // ok e a mensagem viajam juntos, e a UI decide o que mostrar.
        emit containerFinished(params.toVariantMap());
        return;
    }
    if (method == QStringLiteral("event.git.remoteFinished")) {
        emit gitRemoteOperationFinished(params.value(QStringLiteral("operation")).toString(),
                                        params.value(QStringLiteral("success")).toBool(false),
                                        params.value(QStringLiteral("message")).toString());
        return;
    }
    if (handleRunnerNotification(method, params)) {
        return;
    }
}

/// Eventos dos RUNNERS: build, test, quality e run.
///
/// Separado do `handleNotification` em 2026-09-04, e quem mandou separar foi o
/// gate: acrescentar o evento do `datasource` levou a funcao a complexidade
/// cognitiva 26 contra um teto de 25. O numero so' mandou OLHAR — o corte e'
/// por responsabilidade, e a responsabilidade estava na cara: os quatro
/// runners repetem a mesma forma (`started` / `output` / `diagnostic` /
/// `finished`) e nao tem nada a ver com git, disco ou banco.
bool CoreClient::handleRunnerNotification(const QString& method, const QJsonObject& params)
{
    if (method == QStringLiteral("event.build.started")) {
        const QString command = params.value(QStringLiteral("command")).toString();
        appendLog(QStringLiteral("build iniciado: %1").arg(command));
        emit buildStarted(command);
        return true;
    }
    if (method == QStringLiteral("event.build.output")) {
        emit buildOutput(params.value(QStringLiteral("line")).toString());
        return true;
    }
    if (method == QStringLiteral("event.build.diagnostic")) {
        emit buildDiagnostic(params.toVariantMap());
        return true;
    }
    if (method == QStringLiteral("event.build.finished")) {
        const bool success = params.value(QStringLiteral("success")).toBool();
        appendLog(QStringLiteral("build finalizado: %1")
                      .arg(success ? QStringLiteral("sucesso") : QStringLiteral("falha")));
        setBuilding(false);
        m_buildJobId.clear();
        emit buildFinished(success, params.value(QStringLiteral("exitCode")).toInt(-1),
                           params.value(QStringLiteral("diagnostics")).toInt(0));
        return true;
    }
    if (method == QStringLiteral("event.test.started")) {
        const QString command = params.value(QStringLiteral("command")).toString();
        appendLog(QStringLiteral("testes iniciados: %1").arg(command));
        emit testStarted(command);
        return true;
    }
    if (method == QStringLiteral("event.test.output")) {
        emit testOutput(params.value(QStringLiteral("line")).toString(),
                        params.value(QStringLiteral("stream")).toString());
        return true;
    }
    if (method == QStringLiteral("event.test.case")) {
        emit testCase(params.value(QStringLiteral("name")).toString(),
                      params.value(QStringLiteral("status")).toString());
        return true;
    }
    if (method == QStringLiteral("event.test.finished")) {
        setTesting(false);
        m_testJobId.clear();
        // `error` so' vem quando o runner nem correu (emit_run_error no core):
        // a UI mostra a mensagem em vez de um "passou: 0".
        emit testFinished(params.value(QStringLiteral("success")).toBool(),
                          params.value(QStringLiteral("passed")).toInt(0),
                          params.value(QStringLiteral("failed")).toInt(0),
                          params.value(QStringLiteral("ignored")).toInt(0),
                          params.value(QStringLiteral("error")).toString());
        return true;
    }
    if (method == QStringLiteral("event.quality.started")) {
        const QString command = params.value(QStringLiteral("command")).toString();
        appendLog(QStringLiteral("analise iniciada: %1").arg(command));
        emit qualityStarted(command);
        return true;
    }
    if (method == QStringLiteral("event.quality.diagnostic")) {
        emit qualityDiagnostic(params.toVariantMap());
        return true;
    }
    if (method == QStringLiteral("event.quality.output")) {
        // Ate' 2026-09-18 era DESCARTADO aqui (40 §8.1): a saida bruta da
        // analise — o clang-tidy dizendo que nao achou a CDB, o ruff sem
        // config — nao chegava a tela nenhuma. Vai para o painel de Build,
        // como a do build.
        emit qualityOutput(params.value(QStringLiteral("line")).toString(),
                           params.value(QStringLiteral("stream")).toString());
        return true;
    }
    if (method == QStringLiteral("event.quality.finished")) {
        setAnalyzing(false);
        m_qualityJobId.clear();
        emit qualityFinished(params.value(QStringLiteral("success")).toBool(),
                             params.value(QStringLiteral("exitCode")).toInt(-1),
                             params.value(QStringLiteral("diagnostics")).toInt(0));
        return true;
    }
    if (handleCmakeNotification(method, params)) {
        return true;
    }
    if (handleDebugNotification(method, params)) {
        return true;
    }
    if (method == QStringLiteral("event.run.started")) {
        const QString command = params.value(QStringLiteral("command")).toString();
        appendLog(QStringLiteral("execucao iniciada: %1").arg(command));
        setRunning(true);
        emit runStarted(command);
        return true;
    }
    if (method == QStringLiteral("event.run.output")) {
        emit runOutput(params.value(QStringLiteral("line")).toString(),
                       params.value(QStringLiteral("stream")).toString());
        return true;
    }
    if (method == QStringLiteral("event.run.finished")) {
        setRunning(false);
        emit runFinished(params.value(QStringLiteral("success")).toBool(),
                         params.value(QStringLiteral("exitCode")).toInt(-1));
        return true;
    }
    if (handleTerminalNotification(method, params)) {
        return true;
    }
    if (handleLspNotification(method, params)) {
        return true;
    }
    if (handleEnvironmentNotification(method, params)) {
        return true;
    }
    if (handleJobNotification(method, params)) {
        return true;
    }
    return false;
}

bool CoreClient::handleFileSystemNotification(const QString& method, const QJsonObject& params)
{
    if (method == QStringLiteral("event.fs.changed")) {
        emit filesChanged(params.value(QStringLiteral("changes")).toArray().toVariantList());
        return true;
    }
    if (method == QStringLiteral("event.fs.watchError")) {
        const QString message = params.value(QStringLiteral("message")).toString();
        appendErrorLog(QStringLiteral("watcher do workspace: %1").arg(message));
        emit fileWatchFailed(message);
        return true;
    }
    return false;
}

bool CoreClient::handleTerminalNotification(const QString& method, const QJsonObject& params)
{
    if (method == QStringLiteral("event.terminal.render")) {
        // D2 (DocsPublic/roadmaps/24): grid do emulador (cores/cursor/spans) — a UI só desenha.
        emit terminalRender(params.toVariantMap());
        return true;
    }
    if (method == QStringLiteral("event.terminal.closed")) {
        // D2.3: eventos atrasados de workspaces anteriores não podem alterar o
        // estado das sessões atuais; por isso removemos o ID exato.
        const QString id = params.value(QStringLiteral("id")).toString();
        m_terminalIds.remove(id);
        setTerminalActive(!m_terminalIds.isEmpty());
        emit terminalClosed(id, params.value(QStringLiteral("exitCode")).toInt(-1));
        return true;
    }
    return false;
}

bool CoreClient::handleDebugNotification(const QString& method, const QJsonObject& params)
{
    if (method == QStringLiteral("event.debug.started")) {
        const QString program = params.value(QStringLiteral("program")).toString();
        appendLog(QStringLiteral("debug iniciado: %1").arg(program));
        setDebugging(true);
        emit debugStarted(program, params.value(QStringLiteral("attached")).toBool());
        return true;
    }
    if (method == QStringLiteral("event.debug.output")) {
        emit debugOutput(params.value(QStringLiteral("category")).toString(),
                         params.value(QStringLiteral("line")).toString());
        return true;
    }
    if (method == QStringLiteral("event.debug.stopped")) {
        emit debugStopped(params.value(QStringLiteral("reason")).toString(),
                          params.value(QStringLiteral("file")).toString(),
                          params.value(QStringLiteral("line")).toInt(0),
                          params.value(QStringLiteral("threadId")).toInt(0));
        return true;
    }
    if (method == QStringLiteral("event.debug.continued")) {
        emit debugContinued();
        return true;
    }
    if (method == QStringLiteral("event.debug.finished")) {
        setDebugging(false);
        emit debugFinished(params.value(QStringLiteral("exitCode")).toInt(-1));
        return true;
    }
    return false;
}

bool CoreClient::handleEnvironmentNotification(const QString& method, const QJsonObject& params)
{
    if (method == QStringLiteral("event.environment.started")) {
        setScanningEnvironment(true);
        appendLog(QStringLiteral("scan de ambiente iniciado"));
        emit environmentScanStarted(params.value(QStringLiteral("tools")).toInt(0));
        return true;
    }
    if (method == QStringLiteral("event.environment.tool")) {
        emit environmentTool(params.value(QStringLiteral("tool")).toObject().toVariantMap());
        return true;
    }
    if (method == QStringLiteral("event.environment.finished")) {
        setScanningEnvironment(false);
        m_environmentJobId.clear();
        const QVariantList tools = params.value(QStringLiteral("tools")).toArray().toVariantList();
        appendLog(QStringLiteral("scan de ambiente finalizado"));
        emit environmentScanFinished(params.value(QStringLiteral("success")).toBool(),
                                     params.value(QStringLiteral("total")).toInt(0),
                                     params.value(QStringLiteral("detected")).toInt(0),
                                     params.value(QStringLiteral("missing")).toInt(0),
                                     params.value(QStringLiteral("failed")).toInt(0), tools);
        emit toolsListed(tools);
        return true;
    }
    return false;
}

bool CoreClient::handleJobNotification(const QString& method, const QJsonObject& params)
{
    if (method == QStringLiteral("event.job.created")) {
        emit jobCreated(params.toVariantMap());
        return true;
    }
    if (method == QStringLiteral("event.job.progress")) {
        emit jobProgress(params.value(QStringLiteral("jobId")).toString(),
                         params.value(QStringLiteral("status")).toString(),
                         params.value(QStringLiteral("progress")).toDouble(0.0),
                         params.value(QStringLiteral("message")).toString());
        return true;
    }
    if (method == QStringLiteral("event.job.output")) {
        emit jobOutput(params.value(QStringLiteral("jobId")).toString(),
                       params.value(QStringLiteral("line")).toString());
        return true;
    }
    if (method == QStringLiteral("event.job.finished")) {
        emit jobFinished(params.value(QStringLiteral("jobId")).toString(),
                         params.value(QStringLiteral("status")).toString());
        return true;
    }
    return false;
}

} // namespace kinein
