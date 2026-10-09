#include "single_instance_guard.h"

#include <QFileInfo>

namespace kinein {

SingleInstanceGuard::SingleInstanceGuard(QObject* parent) : QObject(parent) {}

SingleInstanceGuard::~SingleInstanceGuard()
{
    release();
}

QString SingleInstanceGuard::workspacePath() const
{
    return m_workspacePath;
}

void SingleInstanceGuard::setWorkspacePath(const QString& path)
{
    // CANONICO DOS DOIS LADOS. Quem chega pela linha de comando canoniza antes
    // de calcular o nome do socket; se aqui ficasse o caminho como veio, um
    // symlink no meio faria as duas pontas nomearem sockets diferentes e a
    // coordenacao simplesmente nao aconteceria — sem erro nenhum.
    const QString canonical = path.isEmpty() ? QString{} : QFileInfo{path}.canonicalFilePath();
    if (canonical == m_workspacePath) {
        return;
    }
    release();
    m_workspacePath = canonical;
    claim();
    emit workspacePathChanged();
}

void SingleInstanceGuard::release()
{
    m_notifier.reset();
    if (m_listenFd >= 0) {
        kinein::releaseSocket(m_listenFd, m_socketPath);
    }
    m_listenFd = -1;
    m_socketPath.clear();
}

void SingleInstanceGuard::claim()
{
    if (m_workspacePath.isEmpty()) {
        return;
    }
    m_socketPath = kinein::socketPathFor(runtimeDirectory(), m_workspacePath);
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
    m_notifier = kinein::watchIncoming(m_listenFd, [this] { serveIncoming(); });
}

void SingleInstanceGuard::serveIncoming()
{
    const std::optional<kinein::Request> request = kinein::acceptOne(m_listenFd, m_workspacePath);
    if (!request.has_value()) {
        return;
    }
    emit activationRequested(request->activationToken);
}

} // namespace kinein
