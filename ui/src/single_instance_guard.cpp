#include "single_instance_guard.h"

#include <QFileInfo>
#include <QSocketNotifier>

namespace {

/// O diretorio de runtime da sessao, ou vazio.
///
/// Vazio e' um caso real (container magro, sessao sem systemd) e nao um erro:
/// a IDE segue sem a coordenacao, abrindo janela como sempre abriu.
QString diretorioDeRuntime()
{
    return qEnvironmentVariable("XDG_RUNTIME_DIR");
}

} // namespace

namespace kinein {

SingleInstanceGuard::SingleInstanceGuard(QObject* parent) : QObject(parent) {}

SingleInstanceGuard::~SingleInstanceGuard()
{
    soltar();
}

QString SingleInstanceGuard::workspacePath() const
{
    return m_workspacePath;
}

void SingleInstanceGuard::setWorkspacePath(const QString& caminho)
{
    // CANONICO DOS DOIS LADOS. Quem chega pela linha de comando canoniza antes
    // de calcular o nome do socket; se aqui ficasse o caminho como veio, um
    // symlink no meio faria as duas pontas nomearem sockets diferentes e a
    // coordenacao simplesmente nao aconteceria — sem erro nenhum.
    const QString limpo = caminho.isEmpty() ? QString{} : QFileInfo{caminho}.canonicalFilePath();
    if (limpo == m_workspacePath) {
        return;
    }
    soltar();
    m_workspacePath = limpo;
    assumir();
    emit workspacePathChanged();
}

void SingleInstanceGuard::soltar()
{
    m_notificador.reset();
    if (m_listenFd >= 0) {
        kinein::releaseSocket(m_listenFd, m_socketPath);
    }
    m_listenFd = -1;
    m_socketPath.clear();
}

void SingleInstanceGuard::assumir()
{
    if (m_workspacePath.isEmpty()) {
        return;
    }
    m_socketPath = kinein::socketPathFor(diretorioDeRuntime(), m_workspacePath);
    if (m_socketPath.isEmpty()) {
        return;
    }
    m_listenFd = kinein::listenFor(m_socketPath);
    if (m_listenFd < 0) {
        // OUTRA JANELA JA' E' A DONA desta pasta. Nao e' erro e nao ha' o que
        // dizer: esta janela continua funcionando, so' nao responde por ela.
        m_socketPath.clear();
        return;
    }
    m_notificador = std::make_unique<QSocketNotifier>(m_listenFd, QSocketNotifier::Read);
    connect(m_notificador.get(), &QSocketNotifier::activated, this, &SingleInstanceGuard::atender);
}

void SingleInstanceGuard::atender()
{
    const std::optional<kinein::Request> pedido = kinein::acceptOne(m_listenFd, m_workspacePath);
    if (!pedido.has_value()) {
        return;
    }
    emit activationRequested(pedido->activationToken);
}

} // namespace kinein
