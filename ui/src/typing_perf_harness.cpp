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
constexpr QLatin1StringView kAlfabeto("abcdefghijklmnopqrstuvwxyz");

int envInt(const char* nome, int padrao)
{
    bool ok = false;
    const int valor = qEnvironmentVariableIntValue(nome, &ok);
    return ok ? valor : padrao;
}

void erro(const QString& motivo)
{
    // A3.2 item 3: ausencia produz motivo explicito, nunca sucesso falso.
    qInfo().noquote().nospace() << "KINEIN_PERF typing_error=" << motivo;
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
          m_teclas(envInt("KINEIN_PERF_TYPING_KEYS", 40)),
          m_quietoMs(envInt("KINEIN_PERF_TYPING_QUIET_MS", 150)),
          m_timeoutMs(envInt("KINEIN_PERF_TYPING_TIMEOUT_MS", 120000))
    {
    }

    void iniciar();

signals:
    void amostraColetada(double /*ms*/);

private:
    void aoConectar();
    void aoAbrirWorkspace();
    void aoCarregarArquivo();
    void agendarProximaTecla();
    void enviarTecla();
    void registrarAmostra(double ms);
    void relatar();

    // Espera a UI ficar QUIETA (nenhum frame por m_quietoMs) antes da proxima
    // tecla. Sem isso o numero mente: o realce de sintaxe volta do core ~280 ms
    // depois da tecla anterior (A3.1) e produz um frame proprio; se a tecla
    // seguinte sair antes dele, esse frame seria creditado a ela e a medicao
    // reportaria uma latencia que ninguem viveu.
    void esperarQuietude(std::function<void()> entao);

    [[nodiscard]] QString textoDoEditor() const;

    QQuickWindow* m_window = nullptr;
    CoreClient* m_client = nullptr;
    QObject* m_editor = nullptr;
    QString m_workspace;
    QString m_file;
    int m_teclas = 0;
    int m_quietoMs = 0;
    int m_timeoutMs = 0;

    // m_clock corre a sessao inteira e nunca reinicia: os carimbos vem de
    // threads diferentes (GUI e render) e precisam da mesma origem.
    QElapsedTimer m_clock;
    std::atomic<qint64> m_teclaEnviadaNs{-1};
    std::atomic<qint64> m_ultimoFrameNs{0};

    QList<double> m_amostras;
    int m_enviadas = 0;
    int m_charsIniciais = 0;
    bool m_workspacePedido = false;
    std::function<void()> m_aposQuietude;
    QTimer m_quietoTimer;
};

void TypingHarness::iniciar()
{
    if (m_workspace.isEmpty() || m_file.isEmpty()) {
        erro(QStringLiteral("KINEIN_PERF_TYPING_WORKSPACE e _FILE sao obrigatorios"));
        return;
    }

    // O cursor piscando produz frames que NENHUMA tecla causou. Como a medicao
    // credita a proxima tecla o primeiro frame que chegar, um pisca no momento
    // errado reportaria latencia menor que a real. Zero desliga o pisca.
    QGuiApplication::styleHints()->setCursorFlashTime(0);

    m_clock.start();
    m_quietoTimer.setInterval(10);
    connect(&m_quietoTimer, &QTimer::timeout, this, [this]() {
        const qint64 desdeFrame =
            m_clock.nsecsElapsed() - m_ultimoFrameNs.load(std::memory_order_acquire);
        if (desdeFrame < static_cast<qint64>(m_quietoMs) * 1000000) {
            return;
        }
        m_quietoTimer.stop();
        const auto entao = m_aposQuietude;
        m_aposQuietude = nullptr;
        if (entao) {
            entao();
        }
    });

    connect(this, &TypingHarness::amostraColetada, this, &TypingHarness::registrarAmostra,
            Qt::QueuedConnection);

    // O carimbo do frame TEM de sair na render thread, no instante do swap.
    // frameSwapped e emitido la; uma conexao queued mediria de brinde a fila
    // de eventos da GUI thread — ruido irrelevante nos 250 ms do startup, mas
    // nao numa metrica de poucos milissegundos.
    connect(
        m_window, &QQuickWindow::frameSwapped, this,
        [this]() {
            const qint64 agora = m_clock.nsecsElapsed();
            m_ultimoFrameNs.store(agora, std::memory_order_release);
            const qint64 enviada = m_teclaEnviadaNs.exchange(-1, std::memory_order_acq_rel);
            if (enviada < 0) {
                return;
            }
            const double ms = static_cast<double>(agora - enviada) / 1000000.0;
            emit amostraColetada(ms);
        },
        Qt::DirectConnection);

    QTimer::singleShot(m_timeoutMs, this, [this]() {
        erro(QStringLiteral("timeout apos %1 ms (amostras=%2)")
                 .arg(m_timeoutMs)
                 .arg(m_amostras.size()));
    });

    connect(m_client, &CoreClient::workspaceChanged, this, [this]() { aoAbrirWorkspace(); });
    connect(m_client, &CoreClient::fileLoaded, this, [this](const QString& path, const QString&) {
        if (path == m_file) {
            aoCarregarArquivo();
        }
    });

    // `start()` do Main.qml so DISPARA o processo: QProcess::start e assincrono
    // e o sendRequest DESCARTA em silencio o que chega antes de Running. Abrir o
    // workspace aqui direto nao falharia — simplesmente nao aconteceria nada.
    // Espera-se `connected` (o ping do core respondeu).
    connect(m_client, &CoreClient::statusChanged, this, [this]() { aoConectar(); });
    aoConectar();
}

void TypingHarness::aoConectar()
{
    if (m_workspacePedido || !m_client->isConnected()) {
        return;
    }
    m_workspacePedido = true;
    m_client->openWorkspace(m_workspace);
}

void TypingHarness::aoAbrirWorkspace()
{
    if (m_enviadas > 0 || m_client->workspaceRoot().isEmpty()) {
        return;
    }
    disconnect(m_client, &CoreClient::workspaceChanged, this, nullptr);
    // Abre a aba. O jump/foco definitivo vem depois, ja com o texto carregado.
    QMetaObject::invokeMethod(m_editor, "openDiagnostic", Q_ARG(QVariant, m_file),
                              Q_ARG(QVariant, 1), Q_ARG(QVariant, 1));
}

void TypingHarness::aoCarregarArquivo()
{
    esperarQuietude([this]() {
        const QString texto = textoDoEditor();
        if (texto.isEmpty()) {
            erro(QStringLiteral("editor vazio apos fs.read"));
            return;
        }
        // Conta quebras de linha, igual ao `fixture_lines` do medir-core.py:
        // duas contagens diferentes da MESMA fixture so gerariam duvida.
        const int linhas = static_cast<int>(texto.count(QLatin1Char('\n')));
        // Guarda o tamanho de partida: e com ele que se prova, no fim, que as
        // teclas entraram de fato no documento.
        m_charsIniciais = static_cast<int>(texto.size());
        // Digitar na PRIMEIRA linha nao seria representativo: seria o pior caso
        // de invalidacao. O meio do arquivo e o gesto comum e e deterministico
        // dado que a fixture e deterministica.
        const int linha = envInt("KINEIN_PERF_TYPING_LINE", linhas / 2);
        qInfo().noquote().nospace() << "KINEIN_PERF typing_file_lines=" << linhas;
        qInfo().noquote().nospace() << "KINEIN_PERF typing_file_bytes=" << texto.toUtf8().size();
        qInfo().noquote().nospace() << "KINEIN_PERF typing_line=" << linha;

        QMetaObject::invokeMethod(m_editor, "openDiagnostic", Q_ARG(QVariant, m_file),
                                  Q_ARG(QVariant, linha), Q_ARG(QVariant, 1));
        QMetaObject::invokeMethod(m_editor, "focusEditor");
        agendarProximaTecla();
    });
}

void TypingHarness::esperarQuietude(std::function<void()> entao)
{
    m_aposQuietude = std::move(entao);
    m_ultimoFrameNs.store(m_clock.nsecsElapsed(), std::memory_order_release);
    m_quietoTimer.start();
}

void TypingHarness::agendarProximaTecla()
{
    if (m_enviadas >= m_teclas) {
        relatar();
        return;
    }
    esperarQuietude([this]() { enviarTecla(); });
}

void TypingHarness::enviarTecla()
{
    const char c = kAlfabeto.at(m_enviadas % kAlfabeto.size()).toLatin1();
    const int key = Qt::Key_A + (c - 'a');
    const QString texto = QString(QChar::fromLatin1(c));
    m_enviadas++;

    // O carimbo sai ANTES do sendEvent: o que o usuario sente comeca na tecla,
    // nao no fim do handler.
    m_teclaEnviadaNs.store(m_clock.nsecsElapsed(), std::memory_order_release);
    QKeyEvent press(QEvent::KeyPress, key, Qt::NoModifier, texto);
    QCoreApplication::sendEvent(m_window, &press);
    QKeyEvent release(QEvent::KeyRelease, key, Qt::NoModifier, texto);
    QCoreApplication::sendEvent(m_window, &release);
}

void TypingHarness::registrarAmostra(double ms)
{
    m_amostras.append(ms);
    // Amostra crua na saida: mediana e p95 sao calculadas pelo runner
    // (medir-performance.sh), e o numero fica auditavel em vez de so resumido.
    qInfo().noquote().nospace() << "KINEIN_PERF typing_sample_ms=" << QString::number(ms, 'f', 2);
    agendarProximaTecla();
}

void TypingHarness::relatar()
{
    // A MESMA armadilha que o A3.3 item 3 caiu: la, o eco do shell fazia o
    // marcador aparecer sem que a rajada tivesse rodado, e a medicao reportou
    // 50 mil linhas em 1,5 ms. Aqui o gemeo seria medir frames que tecla nenhuma
    // causou — o numero sairia igualmente bonito. Entao o documento tem de
    // provar que recebeu: N teclas => N caracteres a mais.
    const int inseridos = static_cast<int>(textoDoEditor().size()) - m_charsIniciais;
    qInfo().noquote().nospace() << "KINEIN_PERF typing_chars_inserted=" << inseridos;
    if (inseridos != m_enviadas) {
        erro(QStringLiteral("teclas perdidas: %1 enviadas, %2 inseridas")
                 .arg(m_enviadas)
                 .arg(inseridos));
        return;
    }
    qInfo().noquote().nospace() << "KINEIN_PERF typing_keys=" << m_amostras.size();
    QCoreApplication::quit();
}

QString TypingHarness::textoDoEditor() const
{
    QVariant retorno;
    if (!QMetaObject::invokeMethod(m_editor, "editorText", Q_RETURN_ARG(QVariant, retorno))) {
        return {};
    }
    return retorno.toString();
}

} // namespace

void installTypingPerfHarness(QGuiApplication& app, QQmlApplicationEngine& engine)
{
    if (!qEnvironmentVariableIsSet("KINEIN_PERF_TYPING")) {
        return;
    }
    if (engine.rootObjects().isEmpty()) {
        erro(QStringLiteral("engine sem root object"));
        return;
    }
    auto* window = qobject_cast<QQuickWindow*>(engine.rootObjects().constFirst());
    if (window == nullptr) {
        erro(QStringLiteral("root object nao e QQuickWindow"));
        return;
    }
    // Alcanca os ids do Main.qml sem exigir objectName na UI de producao:
    // instrumentacao nao deve deixar marca no codigo que ela mede.
    QQmlContext* ctx = qmlContext(window);
    if (ctx == nullptr) {
        erro(QStringLiteral("sem contexto QML no root"));
        return;
    }
    auto* client = qobject_cast<CoreClient*>(ctx->objectForName(QStringLiteral("coreClient")));
    QObject* domains = ctx->objectForName(QStringLiteral("domains"));
    QObject* editor =
        domains == nullptr ? nullptr : domains->property("editorController").value<QObject*>();
    if (client == nullptr || editor == nullptr) {
        erro(QStringLiteral("coreClient/domains.editorController nao encontrados no contexto"));
        return;
    }
    // Dono e o parent-child do Qt: `app` destroi o harness junto com a
    // aplicacao. Nao ha owner alternativo — o harness precisa sobreviver a esta
    // funcao e morrer com o processo.
    // NOLINTNEXTLINE(cppcoreguidelines-owning-memory)
    auto* harness = new TypingHarness(window, client, editor, &app);
    harness->iniciar();
}

} // namespace kinein

#include "typing_perf_harness.moc"
