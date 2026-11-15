// O canal de controle do driver dos testes, dentro da UI.
//
// O driver manda teclas e eventos de IME pelo mesmo caminho de um teclado
// (QCoreApplication::sendEvent na janela), lê a grade, captura a tela e mede
// a latência da tecla ao quadro com o método do medir-performance.sh (A3.3-1).
// Nada disto existe no produto: é instrumento da prova.
#pragma once

#include <QElapsedTimer>
#include <QJsonObject>
#include <QJsonValue>
#include <QLocalServer>
#include <QObject>
#include <QPointer>
#include <QTimer>

#include <atomic>
#include <functional>

class QLocalSocket;
class QQuickWindow;

namespace prova {

class NvimView;
class SupervisorLink;

class ProvaControl : public QObject
{
    Q_OBJECT

public:
    ProvaControl(QQuickWindow* window, NvimView* view, SupervisorLink* link, const QElapsedTimer* clock,
                 QObject* parent = nullptr);

    bool listen(const QString& path);

signals:
    void sampleReady(double ms);

private:
    struct Waiter {
        QPointer<QLocalSocket> socket;
        QJsonValue id;
        std::function<bool()> done;
        std::function<QJsonValue()> result;
        QElapsedTimer elapsed;
        qint64 timeoutMs = 10000;
    };

    void onConnection();
    void handle(QLocalSocket* socket, const QJsonObject& req);
    void reply(QLocalSocket* socket, const QJsonValue& id, bool ok, const QJsonValue& result,
               const QString& error = {});
    void addWaiter(QLocalSocket* socket, const QJsonValue& id, std::function<bool()> done,
                   std::function<QJsonValue()> result, qint64 timeoutMs);
    void checkWaiters();

    QJsonObject state() const;
    void sendKey(int key, Qt::KeyboardModifiers mods, const QString& text);
    void sendSequence(const QString& portable);
    void typeText(const QString& text);
    [[nodiscard]] bool gridContains(const QString& text) const;

    // A medida da tecla ao quadro e da abertura até o quadro.
    void startTyping(QLocalSocket* socket, const QJsonValue& id, int keys, int quietMs, qint64 timeoutMs);
    void typingNextKey();
    void startOpening(QLocalSocket* socket, const QJsonValue& id, const QString& path,
                      const QString& expected, qint64 timeoutMs);
    void armFrameProbe(std::function<bool()> match, std::function<void(double)> done);
    void afterQuiet(int quietMs, std::function<void()> then);

    QQuickWindow* m_window;
    NvimView* m_view;
    SupervisorLink* m_link;
    const QElapsedTimer* m_clock;
    QLocalServer m_server;
    QList<Waiter> m_waiters;
    QTimer m_waiterTimer;

    // Os carimbos vêm da thread da GUI e da de renderização: mesma origem.
    std::atomic<qint64> m_lastFrameNs{0};
    std::atomic<qint64> m_firstFrameNs{-1};
    std::atomic<qint64> m_editorFrameNs{-1};
    std::atomic<bool> m_editorFlushSeen{false};
    qint64 m_nvimReadyMs = -1;

    // A sonda de quadro: 1 = esperando a grade certa; 2 = grade certa, falta
    // sincronizar; 3 = sincronizada, falta o swap.
    std::atomic<int> m_probeStage{0};
    std::atomic<qint64> m_probeSentNs{0};
    std::atomic<qint64> m_probeAnyFrameNs{-1};
    std::function<bool()> m_probeMatch;
    std::function<void(double)> m_probeDone;
    QList<double> m_anyFrameSamples;

    QTimer m_quietTimer;
    int m_quietMs = 150;
    std::function<void()> m_afterQuiet;

    struct Typing {
        QPointer<QLocalSocket> socket;
        QJsonValue id;
        int keys = 0;
        int sent = 0;
        int quietMs = 150;
        int row = 0;
        int col = 0;
        QString original;
        QString typed;
        QList<double> samples;
        bool active = false;
    } m_typing;
};

} // namespace prova
