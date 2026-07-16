// Native window operations used by the client-side Kinein title bar.

#pragma once

#include <QObject>
#include <QPointer>
#include <QQuickWindow>
#include <QtQml/qqmlregistration.h>

namespace kinein {

class WindowChromeController : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    Q_PROPERTY(QQuickWindow* window READ window WRITE setWindow NOTIFY windowChanged)
    Q_PROPERTY(bool maximized READ isMaximized NOTIFY maximizedChanged)

public:
    explicit WindowChromeController(QObject* parent = nullptr);

    [[nodiscard]] QQuickWindow* window() const;
    void setWindow(QQuickWindow* window);
    [[nodiscard]] bool isMaximized() const;

    Q_INVOKABLE void minimize();
    Q_INVOKABLE void toggleMaximized();
    Q_INVOKABLE void closeWindow();
    Q_INVOKABLE bool startSystemMove();
    Q_INVOKABLE bool startSystemResize(int edges);

signals:
    void windowChanged();
    void maximizedChanged();

private:
    QPointer<QQuickWindow> m_window;
};

} // namespace kinein
