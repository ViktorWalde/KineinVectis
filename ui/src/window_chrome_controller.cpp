#include "window_chrome_controller.h"

#include <QWindow>

namespace {

// Converts the raw edge mask sent by QML (Qt.LeftEdge, Qt.TopEdge, ...) into a
// validated Qt::Edges. A resize may span at most one horizontal and one
// vertical edge; anything else (opposite edges together, stray bits) is
// rejected as an empty set so the caller does nothing.
[[nodiscard]] Qt::Edges resizeEdgesFromInt(int edges)
{
    const Qt::Edges requested = Qt::Edges::fromInt(static_cast<Qt::Edges::Int>(edges));
    const Qt::Edges valid =
        requested & (Qt::LeftEdge | Qt::TopEdge | Qt::RightEdge | Qt::BottomEdge);

    if ((valid & (Qt::LeftEdge | Qt::RightEdge)) == (Qt::LeftEdge | Qt::RightEdge)) {
        return {};
    }
    if ((valid & (Qt::TopEdge | Qt::BottomEdge)) == (Qt::TopEdge | Qt::BottomEdge)) {
        return {};
    }
    return valid;
}

} // namespace

namespace kinein {

WindowChromeController::WindowChromeController(QObject* parent) : QObject(parent) {}

QQuickWindow* WindowChromeController::window() const
{
    return m_window.data();
}

void WindowChromeController::setWindow(QQuickWindow* window)
{
    if (m_window == window) {
        return;
    }

    if (!m_window.isNull()) {
        disconnect(m_window.data(), nullptr, this, nullptr);
    }

    m_window = window;
    if (!m_window.isNull()) {
        connect(m_window.data(), &QWindow::windowStateChanged, this,
                [this](Qt::WindowState) { emit maximizedChanged(); });
        connect(m_window.data(), &QObject::destroyed, this, [this]() {
            // QObject::destroyed chega depois de o Qt limpar os QPointer.
            emit windowChanged();
            emit maximizedChanged();
        });
    }

    emit windowChanged();
    emit maximizedChanged();
}

bool WindowChromeController::isMaximized() const
{
    return !m_window.isNull() && m_window->windowStates().testFlag(Qt::WindowMaximized);
}

void WindowChromeController::minimize()
{
    if (!m_window.isNull()) {
        m_window->showMinimized();
    }
}

void WindowChromeController::toggleMaximized()
{
    if (m_window.isNull()) {
        return;
    }

    if (isMaximized()) {
        m_window->showNormal();
    }
    else {
        m_window->showMaximized();
    }
}

void WindowChromeController::closeWindow()
{
    if (!m_window.isNull()) {
        m_window->close();
    }
}

bool WindowChromeController::startSystemMove()
{
    return !m_window.isNull() && m_window->startSystemMove();
}

bool WindowChromeController::startSystemResize(int edges)
{
    if (m_window.isNull() || m_window->windowStates().testFlag(Qt::WindowMaximized) ||
        m_window->windowStates().testFlag(Qt::WindowFullScreen))
    {
        return false;
    }

    const Qt::Edges resizeEdges = resizeEdgesFromInt(edges);
    return resizeEdges != Qt::Edges{} && m_window->startSystemResize(resizeEdges);
}

} // namespace kinein
