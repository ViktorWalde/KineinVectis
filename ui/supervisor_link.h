// O supervisor (o papel do core) como processo filho da UI.
//
// A UI só recebe dele o endereço do socket do `nvim` e os eventos de vida. O
// stdin do supervisor é o fio da posse: quando a UI sai ou cai, o stdin fecha
// e o supervisor encerra o resto (rascunho, servidores, `nvim`).
#pragma once

#include <QObject>
#include <QProcess>
#include <QStringList>
#include <QVariantList>

namespace prova {

class SupervisorLink : public QObject
{
    Q_OBJECT
    Q_PROPERTY(QString socketPath READ socketPath NOTIFY socketPathChanged)
    Q_PROPERTY(QString error READ error NOTIFY errorChanged)

public:
    explicit SupervisorLink(QObject* parent = nullptr);

    void start(const QStringList& command);
    void stop();

    [[nodiscard]] QString socketPath() const { return m_socketPath; }
    [[nodiscard]] QString error() const { return m_error; }
    [[nodiscard]] QVariantList events() const { return m_events; }
    [[nodiscard]] qint64 pid() const { return m_process.processId(); }

signals:
    void socketPathChanged();
    void errorChanged();
    void nvimReady(qint64 elapsedMs);

private:
    void onReadyRead();

    QProcess m_process;
    QByteArray m_buffer;
    QString m_socketPath;
    QString m_error;
    QVariantList m_events;
};

} // namespace prova
