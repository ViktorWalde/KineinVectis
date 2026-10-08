#include "typing_perf_harness.h"

#include "core_client.h"

#include <QChar>
#include <QCoreApplication>
#include <QDebug>
#include <QElapsedTimer>
#include <QEvent>
#include <QGuiApplication>
#include <QKeyEvent>
#include <QList>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QQuickWindow>
#include <QString>
#include <QStyleHints>
#include <QTimer>
#include <QVariant>

#include <atomic>
#include <functional>
#include <utility>

namespace kinein {
namespace {

// Alfabeto da digitacao. So letras, de proposito: parenteses e aspas acionam
// o auto-close de pares, e uma tecla viraria DUAS edicoes — mediria o recurso
// de pares, nao a digitacao.
constexpr QLatin1StringView kAlphabet("abcdefghijklmnopqrstuvwxyz");

int envInt(const char* name, int fallback)
{
    bool ok = false;
    const int envValue = qEnvironmentVariableIntValue(name, &ok);
    return ok ? envValue : fallback;
}

void fail(const QString& reason)
{
    // A3.2 item 3: ausencia produz motivo explicito, nunca sucesso falso.
    qInfo().noquote().nospace() << "KINEIN_PERF typing_error=" << reason;
    // A instalacao tambem pode falhar antes de app.exec(): exit direto se perde.
    QMetaObject::invokeMethod(QCoreApplication::instance(), "exit", Qt::QueuedConnection,
                              Q_ARG(int, 1));
}

// Mede tecla -> frame apresentado no editor real, com arquivo grande aberto.
//
// POR QUE NAO DA PARA MEDIR ISSO NUM HARNESS QML. A rota `qml -I build/.../ui`
// esta fechada com evidencia (DocsPublic/roadmaps/21): o qmldir gerado aponta para
// caminhos qrc:, que so existem dentro do binario compilado, entao o runner
// nunca carrega EditorTextSurface. Por isso o harness vive aqui, no processo
// real, atras de env.
//
// POR QUE DIRIGIR O FLUXO NORMAL. Sem workspace o editor nao aceita tecla, e
// medir num TextEdit vazio mediria o Qt, nao a Kinein. O harness percorre
// workspace.open -> fs.read -> foco antes de cronometrar; e essa orquestracao,
// nao a cronometragem, que custa.
class TypingHarness : public QObject
{
    Q_OBJECT

public:
    TypingHarness(QQuickWindow* window, CoreClient* client, QObject* editor, QObject* parent)
        : QObject(parent), m_window(window), m_client(client), m_editor(editor),
          m_workspace(qEnvironmentVariable("KINEIN_PERF_TYPING_WORKSPACE")),
          m_file(qEnvironmentVariable("KINEIN_PERF_TYPING_FILE")),
          m_keys(envInt("KINEIN_PERF_TYPING_KEYS", 40)),
          m_quietMs(envInt("KINEIN_PERF_TYPING_QUIET_MS", 150)),
          m_timeoutMs(envInt("KINEIN_PERF_TYPING_TIMEOUT_MS", 120000))
    {
    }

    void begin();

signals:
    void sampleCollected(double /*ms*/);

private:
    void onConnected();
    void onWorkspaceOpened();
    void onFileLoaded();
    void scheduleNextKey();
    void sendKey();
    void recordSample(double ms);
    void report();

    // Espera a UI ficar QUIETA (nenhum frame por m_quietoMs) antes da proxima
    // tecla. Sem isso o numero mente: o realce de sintaxe volta do core ~280 ms
    // depois da tecla anterior (A3.1) e produz um frame proprio; se a tecla
    // seguinte sair antes dele, esse frame seria creditado a ela e a medicao
    // reportaria uma latencia que ninguem viveu.
    void waitForQuiet(std::function<void()> then);

    [[nodiscard]] QString editorText() const;

    QQuickWindow* m_window = nullptr;
    CoreClient* m_client = nullptr;
    QObject* m_editor = nullptr;
    QString m_workspace;
    QString m_file;
    int m_keys = 0;
    int m_quietMs = 0;
    int m_timeoutMs = 0;

    // m_clock corre a sessao inteira e nunca reinicia: os carimbos vem de
    // threads diferentes (GUI e render) e precisam da mesma origem.
    QElapsedTimer m_clock;
    std::atomic<qint64> m_keySentNs{-1};
    std::atomic<qint64> m_lastFrameNs{0};

    QList<double> m_samples;
    int m_sentCount = 0;
    int m_initialChars = 0;
    bool m_workspaceRequested = false;
    std::function<void()> m_afterQuiet;
    QTimer m_quietTimer;
};

void TypingHarness::begin()
{
    if (m_workspace.isEmpty() || m_file.isEmpty()) {
        fail(QStringLiteral("KINEIN_PERF_TYPING_WORKSPACE e _FILE sao obrigatorios"));
        return;
    }

    // O cursor piscando produz frames que NENHUMA tecla causou. Como a medicao
    // credita a proxima tecla o primeiro frame que chegar, um pisca no momento
    // errado reportaria latencia menor que a real. Zero desliga o pisca.
    QGuiApplication::styleHints()->setCursorFlashTime(0);

    m_clock.start();
    m_quietTimer.setInterval(10);
    connect(&m_quietTimer, &QTimer::timeout, this, [this] {
        const qint64 sinceFrame =
            m_clock.nsecsElapsed() - m_lastFrameNs.load(std::memory_order_acquire);
        if (sinceFrame < static_cast<qint64>(m_quietMs) * 1000000) {
            return;
        }
        m_quietTimer.stop();
        const auto then = m_afterQuiet;
        m_afterQuiet = nullptr;
        if (then) {
            then();
        }
    });

    connect(this, &TypingHarness::sampleCollected, this, &TypingHarness::recordSample,
            Qt::QueuedConnection);

    // O carimbo do frame TEM de sair na render thread, no instante do swap.
    // frameSwapped e emitido la; uma conexao queued mediria de brinde a fila
    // de eventos da GUI thread — ruido irrelevante nos 250 ms do startup, mas
    // nao numa metrica de poucos milissegundos.
    connect(
        m_window, &QQuickWindow::frameSwapped, this,
        [this] {
            const qint64 now = m_clock.nsecsElapsed();
            m_lastFrameNs.store(now, std::memory_order_release);
            const qint64 sentAt = m_keySentNs.exchange(-1, std::memory_order_acq_rel);
            if (sentAt < 0) {
                return;
            }
            const double ms = static_cast<double>(now - sentAt) / 1000000.0;
            emit sampleCollected(ms);
        },
        Qt::DirectConnection);

    QTimer::singleShot(m_timeoutMs, this, [this] {
        fail(QStringLiteral("timeout apos %1 ms (amostras=%2)")
                 .arg(m_timeoutMs)
                 .arg(m_samples.size()));
    });

    connect(m_client, &CoreClient::workspaceChanged, this, [this] { onWorkspaceOpened(); });
    connect(m_client, &CoreClient::fileLoaded, this, [this](const QString& path, const QString&) {
        if (path == m_file) {
            onFileLoaded();
        }
    });

    // `start()` do Main.qml so DISPARA o processo: QProcess::start e assincrono
    // e o sendRequest DESCARTA em silencio o que chega antes de Running. Abrir o
    // workspace aqui direto nao falharia — simplesmente nao aconteceria nada.
    // Espera-se `connected` (o ping do core respondeu).
    connect(m_client, &CoreClient::statusChanged, this, [this] { onConnected(); });
    onConnected();
}

void TypingHarness::onConnected()
{
    if (m_workspaceRequested || !m_client->isConnected()) {
        return;
    }
    m_workspaceRequested = true;
    m_client->openWorkspace(m_workspace);
}

void TypingHarness::onWorkspaceOpened()
{
    if (m_sentCount > 0 || m_client->workspaceRoot().isEmpty()) {
        return;
    }
    disconnect(m_client, &CoreClient::workspaceChanged, this, nullptr);
    // Abre a aba. O jump/foco definitivo vem depois, ja com o texto carregado.
    QMetaObject::invokeMethod(m_editor, "openDiagnostic", Q_ARG(QVariant, m_file),
                              Q_ARG(QVariant, 1), Q_ARG(QVariant, 1));
}

void TypingHarness::onFileLoaded()
{
    waitForQuiet([this] {
        const QString text = editorText();
        if (text.isEmpty()) {
            fail(QStringLiteral("editor vazio apos fs.read"));
            return;
        }
        // Conta quebras de linha, igual ao `fixture_lines` do medir-core.py:
        // duas contagens diferentes da MESMA fixture so gerariam duvida.
        const int lineCount = static_cast<int>(text.count(QLatin1Char('\n')));
        // Guarda o tamanho de partida: e com ele que se prova, no fim, que as
        // teclas entraram de fato no documento.
        m_initialChars = static_cast<int>(text.size());
        // Digitar na PRIMEIRA linha nao seria representativo: seria o pior caso
        // de invalidacao. O meio do arquivo e o gesto comum e e deterministico
        // dado que a fixture e deterministica.
        const int targetLine = envInt("KINEIN_PERF_TYPING_LINE", lineCount / 2);
        qInfo().noquote().nospace() << "KINEIN_PERF typing_file_lines=" << lineCount;
        qInfo().noquote().nospace() << "KINEIN_PERF typing_file_bytes=" << text.toUtf8().size();
        qInfo().noquote().nospace() << "KINEIN_PERF typing_line=" << targetLine;

        QMetaObject::invokeMethod(m_editor, "openDiagnostic", Q_ARG(QVariant, m_file),
                                  Q_ARG(QVariant, targetLine), Q_ARG(QVariant, 1));
        QMetaObject::invokeMethod(m_editor, "focusEditor");
        scheduleNextKey();
    });
}

void TypingHarness::waitForQuiet(std::function<void()> then)
{
    m_afterQuiet = std::move(then);
    m_lastFrameNs.store(m_clock.nsecsElapsed(), std::memory_order_release);
    m_quietTimer.start();
}

void TypingHarness::scheduleNextKey()
{
    if (m_sentCount >= m_keys) {
        report();
        return;
    }
    waitForQuiet([this] { sendKey(); });
}

void TypingHarness::sendKey()
{
    const char c = kAlphabet.at(m_sentCount % kAlphabet.size()).toLatin1();
    const int key = Qt::Key_A + (c - 'a');
    const QString text = QString(QChar::fromLatin1(c));
    m_sentCount++;

    // O carimbo sai ANTES do sendEvent: o que o usuario sente comeca na tecla,
    // nao no fim do handler.
    m_keySentNs.store(m_clock.nsecsElapsed(), std::memory_order_release);
    QKeyEvent press(QEvent::KeyPress, key, Qt::NoModifier, text);
    QCoreApplication::sendEvent(m_window, &press);
    QKeyEvent release(QEvent::KeyRelease, key, Qt::NoModifier, text);
    QCoreApplication::sendEvent(m_window, &release);
}

void TypingHarness::recordSample(double ms)
{
    m_samples.append(ms);
    // Amostra crua na saida: mediana e p95 sao calculadas pelo runner
    // (medir-performance.sh), e o numero fica auditavel em vez de so resumido.
    qInfo().noquote().nospace() << "KINEIN_PERF typing_sample_ms=" << QString::number(ms, 'f', 2);
    scheduleNextKey();
}

void TypingHarness::report()
{
    // A MESMA armadilha que o A3.3 item 3 caiu: la, o eco do shell fazia o
    // marcador aparecer sem que a rajada tivesse rodado, e a medicao reportou
    // 50 mil linhas em 1,5 ms. Aqui o gemeo seria medir frames que tecla nenhuma
    // causou — o numero sairia igualmente bonito. Entao o documento tem de
    // provar que recebeu: N teclas => N caracteres a mais.
    const int inserted = static_cast<int>(editorText().size()) - m_initialChars;
    qInfo().noquote().nospace() << "KINEIN_PERF typing_chars_inserted=" << inserted;
    if (inserted != m_sentCount) {
        fail(QStringLiteral("teclas perdidas: %1 enviadas, %2 inseridas")
                 .arg(m_sentCount)
                 .arg(inserted));
        return;
    }
    qInfo().noquote().nospace() << "KINEIN_PERF typing_keys=" << m_samples.size();
    QCoreApplication::quit();
}

QString TypingHarness::editorText() const
{
    QVariant returned;
    if (!QMetaObject::invokeMethod(m_editor, "editorText", Q_RETURN_ARG(QVariant, returned))) {
        return {};
    }
    return returned.toString();
}

} // namespace

void installTypingPerfHarness(QGuiApplication& app, QQmlApplicationEngine& engine)
{
    if (!qEnvironmentVariableIsSet("KINEIN_PERF_TYPING")) {
        return;
    }
    if (engine.rootObjects().isEmpty()) {
        fail(QStringLiteral("engine sem root object"));
        return;
    }
    auto* window = qobject_cast<QQuickWindow*>(engine.rootObjects().constFirst());
    if (window == nullptr) {
        fail(QStringLiteral("root object nao e QQuickWindow"));
        return;
    }
    // Alcanca os ids do Main.qml sem exigir objectName na UI de producao:
    // instrumentacao nao deve deixar marca no codigo que ela mede.
    QQmlContext* ctx = qmlContext(window);
    if (ctx == nullptr) {
        fail(QStringLiteral("sem contexto QML no root"));
        return;
    }
    auto* client = qobject_cast<CoreClient*>(ctx->objectForName(QStringLiteral("coreClient")));
    QObject* domains = ctx->objectForName(QStringLiteral("domains"));
    QObject* editor =
        domains == nullptr ? nullptr : domains->property("editorController").value<QObject*>();
    if (client == nullptr || editor == nullptr) {
        fail(QStringLiteral("coreClient/domains.editorController nao encontrados no contexto"));
        return;
    }
    // Dono e o parent-child do Qt: `app` destroi o harness junto com a
    // aplicacao. Nao ha owner alternativo — o harness precisa sobreviver a esta
    // funcao e morrer com o processo.
    // NOLINTNEXTLINE(cppcoreguidelines-owning-memory)
    auto* harness = new TypingHarness(window, client, editor, &app);
    harness->begin();
}

} // namespace kinein

// O .moc e' codigo GERADO e recebe a mesma isencao que o mocs_compilation.cpp
// tem no CMakeLists: o moc do Qt 6.10 monta os QtMocHelpers por CTAD, que o
// -Wctad-maybe-unsupported (Clang 21, medido em 2026-09-17) reprova sob
// -Werror. A isencao fica presa ao include gerado; o resto do arquivo segue
// integralmente sob os warnings rigorosos.
#ifdef __clang__
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wctad-maybe-unsupported"
#endif
#include "typing_perf_harness.moc"
#ifdef __clang__
#pragma clang diagnostic pop
#endif
