#include "prova_control.h"

#include "msgpack.h"
#include "nvim_view.h"
#include "supervisor_link.h"

#include <QAccessible>
#include <QClipboard>
#include <QHash>
#include <QCoreApplication>
#include <QFontInfo>
#include <QGuiApplication>
#include <QImage>
#include <QInputMethodEvent>
#include <QJsonArray>
#include <QJsonDocument>
#include <QKeyEvent>
#include <QKeySequence>
#include <QLocalSocket>
#include <QQuickWindow>

#include <cmath>

namespace prova {
namespace {

QJsonValue toJson(const QVariant& v)
{
    if (v.metaType().id() == qMetaTypeId<NvimExt>()) {
        return QJsonValue(v.value<NvimExt>().handle);
    }
    switch (v.typeId()) {
    case QMetaType::QVariantList: {
        QJsonArray a;
        for (const QVariant& x : v.toList()) {
            a.append(toJson(x));
        }
        return a;
    }
    case QMetaType::QVariantMap: {
        QJsonObject o;
        const QVariantMap m = v.toMap();
        for (auto it = m.cbegin(); it != m.cend(); ++it) {
            o.insert(it.key(), toJson(it.value()));
        }
        return o;
    }
    case QMetaType::QByteArray:
        return QString::fromUtf8(v.toByteArray());
    case QMetaType::UnknownType:
        return QJsonValue::Null;
    default:
        return QJsonValue::fromVariant(v);
    }
}

// O JSON não separa inteiro de real; a API do Neovim separa. Número inteiro
// vai como inteiro.
QVariant fromJson(const QJsonValue& v)
{
    switch (v.type()) {
    case QJsonValue::Array: {
        QVariantList l;
        for (const QJsonValue& x : v.toArray()) {
            l.append(fromJson(x));
        }
        return l;
    }
    case QJsonValue::Object: {
        QVariantMap m;
        const QJsonObject o = v.toObject();
        for (auto it = o.begin(); it != o.end(); ++it) {
            m.insert(it.key(), fromJson(it.value()));
        }
        return m;
    }
    case QJsonValue::Double: {
        const double d = v.toDouble();
        if (std::floor(d) == d && std::abs(d) < 9.0e15) {
            return static_cast<qint64>(d);
        }
        return d;
    }
    case QJsonValue::Null:
    case QJsonValue::Undefined:
        return {};
    default:
        return v.toVariant();
    }
}

QString rstrip(QString s)
{
    while (!s.isEmpty() && s.back().isSpace()) {
        s.chop(1);
    }
    return s;
}

QJsonArray toArray(const QList<double>& values)
{
    QJsonArray a;
    for (double v : values) {
        a.append(v);
    }
    return a;
}

} // namespace

ProvaControl::ProvaControl(QQuickWindow* window, NvimView* view, SupervisorLink* link,
                           const QElapsedTimer* clock, QObject* parent)
    : QObject(parent), m_window(window), m_view(view), m_link(link), m_clock(clock)
{
    connect(&m_server, &QLocalServer::newConnection, this, &ProvaControl::onConnection);
    m_waiterTimer.setInterval(20);
    connect(&m_waiterTimer, &QTimer::timeout, this, &ProvaControl::checkWaiters);
    connect(m_view, &NvimView::flushed, this, [this] {
        if (m_view->attached() && !m_editorFlushSeen.exchange(true)) {
            // O primeiro quadro do editor é o que sai depois deste flush.
        }
        if (m_probeStage.load() == 1 && m_probeMatch && m_probeMatch()) {
            m_probeStage.store(2);
        }
        checkWaiters();
    });
    connect(m_view, &NvimView::popupChanged, this, &ProvaControl::checkWaiters);
    connect(m_link, &SupervisorLink::socketPathChanged, this, [this] {
        if (!m_link->socketPath().isEmpty() && m_nvimReadyMs < 0) {
            m_nvimReadyMs = m_clock->elapsed();
        }
    });

    // Os carimbos de quadro saem na thread de renderização, no instante do
    // sync e do swap (o mesmo cuidado do typing_perf_harness.cpp).
    connect(
        m_window, &QQuickWindow::afterSynchronizing, this,
        [this] {
            if (m_probeStage.load() == 2) {
                m_probeStage.store(3);
            }
        },
        Qt::DirectConnection);
    connect(
        m_window, &QQuickWindow::frameSwapped, this,
        [this] {
            const qint64 now = m_clock->nsecsElapsed();
            m_lastFrameNs.store(now);
            qint64 expected = -1;
            m_firstFrameNs.compare_exchange_strong(expected, now);
            if (m_editorFlushSeen.load()) {
                expected = -1;
                m_editorFrameNs.compare_exchange_strong(expected, now);
            }
            expected = -1;
            m_probeAnyFrameNs.compare_exchange_strong(expected, now);
            int stage = 3;
            if (m_probeStage.compare_exchange_strong(stage, 0)) {
                const double ms = static_cast<double>(now - m_probeSentNs.load()) / 1.0e6;
                emit sampleReady(ms);
            }
        },
        Qt::DirectConnection);
    connect(
        this, &ProvaControl::sampleReady, this,
        [this](double ms) {
            const qint64 any = m_probeAnyFrameNs.load();
            m_anyFrameSamples.append(any < 0 ? -1.0 : static_cast<double>(any - m_probeSentNs.load()) / 1.0e6);
            if (auto done = std::exchange(m_probeDone, nullptr)) {
                m_probeMatch = nullptr;
                done(ms);
            }
        },
        Qt::QueuedConnection);

    m_quietTimer.setInterval(5);
    connect(&m_quietTimer, &QTimer::timeout, this, [this] {
        const qint64 since = m_clock->nsecsElapsed() - m_lastFrameNs.load();
        if (since < static_cast<qint64>(m_quietMs) * 1000000) {
            return;
        }
        m_quietTimer.stop();
        if (auto then = std::exchange(m_afterQuiet, nullptr)) {
            then();
        }
    });
}

bool ProvaControl::listen(const QString& path)
{
    QLocalServer::removeServer(path);
    m_server.setSocketOptions(QLocalServer::UserAccessOption);
    return m_server.listen(path);
}

void ProvaControl::onConnection()
{
    while (QLocalSocket* socket = m_server.nextPendingConnection()) {
        connect(socket, &QLocalSocket::disconnected, socket, &QObject::deleteLater);
        connect(socket, &QLocalSocket::readyRead, this, [this, socket] {
            while (socket->canReadLine()) {
                const QJsonObject req = QJsonDocument::fromJson(socket->readLine()).object();
                handle(socket, req);
            }
        });
    }
}

void ProvaControl::reply(QLocalSocket* socket, const QJsonValue& id, bool ok, const QJsonValue& result,
                         const QString& error)
{
    if (socket == nullptr) {
        return;
    }
    QJsonObject o{{QStringLiteral("id"), id}, {QStringLiteral("ok"), ok}};
    if (ok) {
        o.insert(QStringLiteral("resultado"), result);
    } else {
        o.insert(QStringLiteral("erro"), error);
    }
    socket->write(QJsonDocument(o).toJson(QJsonDocument::Compact) + '\n');
    socket->flush();
}

void ProvaControl::addWaiter(QLocalSocket* socket, const QJsonValue& id, std::function<bool()> done,
                             std::function<QJsonValue()> result, qint64 timeoutMs)
{
    Waiter w{socket, id, std::move(done), std::move(result), {}, timeoutMs};
    w.elapsed.start();
    m_waiters.append(std::move(w));
    m_waiterTimer.start();
    checkWaiters();
}

void ProvaControl::checkWaiters()
{
    for (qsizetype i = 0; i < m_waiters.size();) {
        Waiter& w = m_waiters[i];
        const bool done = w.done();
        if (done || w.elapsed.elapsed() > w.timeoutMs) {
            QJsonObject r = w.result().toObject();
            r.insert(QStringLiteral("ms"), static_cast<double>(w.elapsed.nsecsElapsed()) / 1.0e6);
            r.insert(QStringLiteral("cumprido"), done);
            reply(w.socket, w.id, true, r);
            m_waiters.removeAt(i);
            continue;
        }
        ++i;
    }
    if (m_waiters.isEmpty()) {
        m_waiterTimer.stop();
    }
}

QJsonObject ProvaControl::state() const
{
    QJsonArray rows;
    for (const QString& r : m_view->gridRows()) {
        rows.append(r);
    }
    QJsonArray items;
    for (const QVariant& i : m_view->popupItems()) {
        items.append(toJson(i));
    }
    const auto ms = [this](qint64 ns) { return ns < 0 ? -1.0 : static_cast<double>(ns) / 1.0e6; };
    return {
        {QStringLiteral("anexado"), m_view->attached()},
        {QStringLiteral("socket"), m_view->socketPath()},
        {QStringLiteral("modo"), m_view->mode()},
        {QStringLiteral("grade"), rows},
        {QStringLiteral("cursor"), QJsonArray{m_view->cursorRow(), m_view->cursorCol()}},
        {QStringLiteral("popup_visivel"), m_view->popupVisible()},
        {QStringLiteral("popup_itens"), items},
        {QStringLiteral("popup_selecionado"), m_view->popupSelected()},
        {QStringLiteral("foco"), m_view->hasActiveFocus()},
        {QStringLiteral("a11y_linha"), m_view->a11yLine()},
        {QStringLiteral("a11y_coluna"), m_view->a11yColumn()},
        {QStringLiteral("primeiro_quadro_ms"), ms(m_firstFrameNs.load())},
        {QStringLiteral("editor_quadro_ms"), ms(m_editorFrameNs.load())},
        {QStringLiteral("nvim_pronto_ms"), static_cast<double>(m_nvimReadyMs)},
        {QStringLiteral("erro_supervisor"), m_link->error()},
        {QStringLiteral("eventos_supervisor"), QJsonArray::fromVariantList(m_link->events())},
        {QStringLiteral("erros_de_protocolo"), static_cast<double>(m_view->rpc()->protocolErrors())},
        {QStringLiteral("pid_supervisor"), static_cast<double>(m_link->pid())},
        {QStringLiteral("fonte"), QFontInfo(m_view->font()).family()},
        {QStringLiteral("fonte_fixa"), QFontInfo(m_view->font()).fixedPitch()},
        {QStringLiteral("celula"), QJsonArray{m_view->cellWidth(), m_view->cellHeight()}},
        {QStringLiteral("colunas"), m_view->gridCols()},
        {QStringLiteral("janela"), QJsonArray{m_window->width(), m_window->height(), m_window->devicePixelRatio()}},
    };
}

void ProvaControl::sendKey(int key, Qt::KeyboardModifiers mods, const QString& text)
{
    QKeyEvent press(QEvent::KeyPress, key, mods, text);
    QCoreApplication::sendEvent(m_window, &press);
    QKeyEvent release(QEvent::KeyRelease, key, mods, text);
    QCoreApplication::sendEvent(m_window, &release);
}

void ProvaControl::sendSequence(const QString& portable)
{
    const QKeySequence seq = QKeySequence::fromString(portable, QKeySequence::PortableText);
    if (seq.isEmpty()) {
        return;
    }
    const QKeyCombination combo = seq[0];
    const int key = combo.key();
    const Qt::KeyboardModifiers mods = combo.keyboardModifiers();
    QString text;
    const bool command = mods & (Qt::ControlModifier | Qt::AltModifier | Qt::MetaModifier);
    if (!command) {
        if (key == Qt::Key_Space) {
            text = QStringLiteral(" ");
        } else if (key >= Qt::Key_A && key <= Qt::Key_Z) {
            const QChar c(key);
            text = (mods & Qt::ShiftModifier) ? QString(c) : QString(c.toLower());
        } else if (key > 0x20 && key < 0x7f) {
            text = QString(QChar(key));
        }
    }
    // Shift+Tab chega do Qt como Backtab, igual ao teclado de verdade.
    sendKey(key == Qt::Key_Tab && (mods & Qt::ShiftModifier) ? Qt::Key_Backtab : key, mods, text);
}

void ProvaControl::typeText(const QString& text)
{
    for (const char32_t cp : text.toUcs4()) {
        const QString s = QString::fromUcs4(&cp, 1);
        if (cp == U'\n') {
            sendKey(Qt::Key_Return, Qt::NoModifier, QStringLiteral("\r"));
        } else if (cp == U'\t') {
            sendKey(Qt::Key_Tab, Qt::NoModifier, QStringLiteral("\t"));
        } else if (cp == U' ') {
            sendKey(Qt::Key_Space, Qt::NoModifier, s);
        } else if (cp >= U'a' && cp <= U'z') {
            sendKey(Qt::Key_A + static_cast<int>(cp - U'a'), Qt::NoModifier, s);
        } else if (cp >= U'A' && cp <= U'Z') {
            sendKey(Qt::Key_A + static_cast<int>(cp - U'A'), Qt::ShiftModifier, s);
        } else if (cp < 0x7f) {
            sendKey(static_cast<int>(cp), Qt::NoModifier, s);
        } else {
            // Um caractere que o layout produz direto (é, ç): tecla sem código.
            sendKey(0, Qt::NoModifier, s);
        }
    }
}

bool ProvaControl::gridContains(const QString& text) const
{
    for (const QString& r : m_view->gridRows()) {
        if (r.contains(text)) {
            return true;
        }
    }
    return false;
}

void ProvaControl::afterQuiet(int quietMs, std::function<void()> then)
{
    m_quietMs = quietMs;
    m_afterQuiet = std::move(then);
    m_lastFrameNs.store(m_clock->nsecsElapsed());
    m_quietTimer.start();
}

void ProvaControl::armFrameProbe(std::function<bool()> match, std::function<void(double)> done)
{
    m_probeMatch = std::move(match);
    m_probeDone = std::move(done);
    m_probeAnyFrameNs.store(-1);
    m_probeSentNs.store(m_clock->nsecsElapsed());
    m_probeStage.store(1);
}

void ProvaControl::startTyping(QLocalSocket* socket, const QJsonValue& id, int keys, int quietMs,
                               qint64 timeoutMs)
{
    m_typing = Typing{};
    m_typing.socket = socket;
    m_typing.id = id;
    m_typing.keys = keys;
    m_typing.quietMs = quietMs;
    m_typing.row = m_view->cursorRow();
    m_typing.col = m_view->cursorCol();
    m_typing.original = m_view->gridRows().value(m_typing.row);
    m_typing.active = true;
    m_anyFrameSamples.clear();
    QTimer::singleShot(timeoutMs, this, [this] {
        if (!m_typing.active) {
            return;
        }
        m_typing.active = false;
        m_probeStage.store(0);
        reply(m_typing.socket, m_typing.id, false, {},
              QStringLiteral("tempo esgotado com %1 amostras").arg(m_typing.samples.size()));
    });
    typingNextKey();
}

void ProvaControl::typingNextKey()
{
    if (!m_typing.active) {
        return;
    }
    if (m_typing.sent >= m_typing.keys) {
        m_typing.active = false;
        reply(m_typing.socket, m_typing.id, true,
              QJsonObject{{QStringLiteral("amostras_ms"), toArray(m_typing.samples)},
                          {QStringLiteral("primeiro_quadro_qualquer_ms"), toArray(m_anyFrameSamples)},
                          {QStringLiteral("linha_grade"), m_typing.row},
                          {QStringLiteral("coluna_grade"), m_typing.col},
                          {QStringLiteral("digitado"), m_typing.typed},
                          {QStringLiteral("linha_final"), m_view->gridRows().value(m_typing.row)}});
        return;
    }
    afterQuiet(m_typing.quietMs, [this] {
        const QChar c(QLatin1Char('a').unicode() + m_typing.sent % 26);
        m_typing.sent++;
        m_typing.typed.append(c);
        // A grade certa é a linha original com o texto digitado na coluna do
        // cursor: o quadro só conta quando mostra a tecla.
        const QString expected = rstrip(
            (m_typing.original.left(m_typing.col) + m_typing.typed + m_typing.original.mid(m_typing.col))
                .left(m_typing.original.size()));
        armFrameProbe(
            [this, expected] { return rstrip(m_view->gridRows().value(m_typing.row)) == expected; },
            [this](double ms) {
                m_typing.samples.append(ms);
                typingNextKey();
            });
        sendKey(Qt::Key_A + (c.unicode() - 'a'), Qt::NoModifier, QString(c));
    });
}

void ProvaControl::startOpening(QLocalSocket* socket, const QJsonValue& id, const QString& path,
                                const QString& expected, qint64 timeoutMs)
{
    QPointer<QLocalSocket> s(socket);
    auto finished = std::make_shared<bool>(false);
    QTimer::singleShot(timeoutMs, this, [this, s, id, finished] {
        if (*finished) {
            return;
        }
        *finished = true;
        m_probeStage.store(0);
        reply(s, id, false, {}, QStringLiteral("tempo esgotado na abertura"));
    });
    armFrameProbe([this, expected] { return gridContains(expected); },
                  [this, s, id, finished](double ms) {
                      if (*finished) {
                          return;
                      }
                      *finished = true;
                      reply(s, id, true, QJsonObject{{QStringLiteral("ms"), ms}});
                  });
    m_view->rpc()->request(QStringLiteral("nvim_exec_lua"),
                           {QStringLiteral("return require('kinein').open(...)"), QVariantList{path}});
}

void ProvaControl::handle(QLocalSocket* socket, const QJsonObject& req)
{
    const QJsonValue id = req.value(QStringLiteral("id"));
    const QString op = req.value(QStringLiteral("op")).toString();
    const qint64 timeoutMs = req.value(QStringLiteral("timeout_ms")).toInteger(10000);

    if (op == QLatin1String("estado")) {
        reply(socket, id, true, state());
    } else if (op == QLatin1String("foco")) {
        m_view->forceActiveFocus(Qt::OtherFocusReason);
        reply(socket, id, true, m_view->hasActiveFocus());
    } else if (op == QLatin1String("teclas")) {
        const QJsonArray seq = req.value(QStringLiteral("seq")).toArray();
        for (const QJsonValue& k : seq) {
            sendSequence(k.toString());
        }
        reply(socket, id, true, static_cast<int>(seq.size()));
    } else if (op == QLatin1String("digitar")) {
        typeText(req.value(QStringLiteral("texto")).toString());
        reply(socket, id, true, true);
    } else if (op == QLatin1String("esperar_grade")) {
        const QString text = req.value(QStringLiteral("contem")).toString();
        const bool absent = req.value(QStringLiteral("ausente")).toBool(false);
        addWaiter(
            socket, id, [this, text, absent] { return gridContains(text) != absent; },
            [this] { return QJsonObject{{QStringLiteral("estado"), state()}}; }, timeoutMs);
    } else if (op == QLatin1String("esperar_popup")) {
        const bool visible = req.value(QStringLiteral("visivel")).toBool(true);
        addWaiter(
            socket, id,
            [this, visible] {
                return m_view->popupVisible() == visible && (!visible || !m_view->popupItems().isEmpty());
            },
            [this] { return QJsonObject{{QStringLiteral("estado"), state()}}; }, timeoutMs);
    } else if (op == QLatin1String("esperar_anexado")) {
        addWaiter(
            socket, id, [this] { return m_view->attached() && !m_view->gridRows().isEmpty(); },
            [this] { return QJsonObject{{QStringLiteral("estado"), state()}}; }, timeoutMs);
    } else if (op == QLatin1String("esperar_quieto")) {
        QPointer<QLocalSocket> s(socket);
        afterQuiet(req.value(QStringLiteral("quieto_ms")).toInt(150),
                   [this, s, id] { reply(s, id, true, true); });
    } else if (op == QLatin1String("captura")) {
        const QString path = req.value(QStringLiteral("caminho")).toString();
        const QImage image = m_window->grabWindow();
        const bool saved = image.save(path);
        reply(socket, id, saved,
              QJsonObject{{QStringLiteral("largura"), image.width()},
                          {QStringLiteral("altura"), image.height()},
                          {QStringLiteral("dpr"), image.devicePixelRatio()},
                          {QStringLiteral("estado"), state()}},
              QStringLiteral("captura não gravada"));
    } else if (op == QLatin1String("nvim")) {
        QPointer<QLocalSocket> s(socket);
        const QVariantList args = fromJson(req.value(QStringLiteral("args"))).toList();
        m_view->rpc()->request(req.value(QStringLiteral("metodo")).toString(), args,
                               [this, s, id](const QVariant& err, const QVariant& res) {
                                   if (err.isValid()) {
                                       const QJsonValue e = toJson(err);
                                       const QJsonArray detail = e.isArray() ? e.toArray() : QJsonArray{e};
                                       reply(s, id, false, {},
                                             QString::fromUtf8(QJsonDocument(detail).toJson(QJsonDocument::Compact)));
                                   } else {
                                       reply(s, id, true, toJson(res));
                                   }
                               });
    } else if (op == QLatin1String("digitacao")) {
        startTyping(socket, id, req.value(QStringLiteral("teclas")).toInt(40),
                    req.value(QStringLiteral("quieto_ms")).toInt(150), req.value(QStringLiteral("timeout_ms")).toInteger(120000));
    } else if (op == QLatin1String("abertura")) {
        startOpening(socket, id, req.value(QStringLiteral("path")).toString(),
                     req.value(QStringLiteral("esperado")).toString(), timeoutMs);
    } else if (op == QLatin1String("ime")) {
        QObject* target = m_window->focusObject();
        int sent = 0;
        for (const QJsonValue& e : req.value(QStringLiteral("eventos")).toArray()) {
            const QJsonObject ev = e.toObject();
            const QString preedit = ev.value(QStringLiteral("preedit")).toString();
            QList<QInputMethodEvent::Attribute> attrs;
            if (!preedit.isEmpty()) {
                attrs.append({QInputMethodEvent::Cursor, static_cast<int>(preedit.size()), 1});
            }
            QInputMethodEvent ime(preedit, attrs);
            ime.setCommitString(ev.value(QStringLiteral("commit")).toString());
            if (target != nullptr) {
                QCoreApplication::sendEvent(target, &ime);
                ++sent;
            }
        }
        const QRectF cursorRect = m_view->inputMethodQuery(Qt::ImCursorRectangle).toRectF();
        reply(socket, id, target != nullptr,
              QJsonObject{{QStringLiteral("enviados"), sent},
                          {QStringLiteral("alvo_e_o_editor"), target == m_view},
                          {QStringLiteral("cursor_rect"),
                           QJsonArray{cursorRect.x(), cursorRect.y(), cursorRect.width(), cursorRect.height()}}},
              QStringLiteral("sem objeto com foco"));
    } else if (op == QLatin1String("a11y")) {
        QAccessibleInterface* iface = QAccessible::queryAccessibleInterface(m_view);
        QJsonObject r;
        if (iface != nullptr) {
            r.insert(QStringLiteral("papel"), static_cast<int>(iface->role()));
            r.insert(QStringLiteral("editavel"), iface->role() == QAccessible::EditableText);
            r.insert(QStringLiteral("nome"), iface->text(QAccessible::Name));
            r.insert(QStringLiteral("valor"), iface->text(QAccessible::Value));
            r.insert(QStringLiteral("focado"), static_cast<bool>(iface->state().focused));
            if (QAccessibleTextInterface* t = iface->textInterface()) {
                r.insert(QStringLiteral("cursor"), t->cursorPosition());
                r.insert(QStringLiteral("caracteres"), t->characterCount());
                r.insert(QStringLiteral("texto"), t->text(0, t->characterCount()));
            }
        }
        // O editor está na árvore que um leitor de tela percorre a partir da janela?
        std::function<bool(QAccessibleInterface*, int)> reach = [&](QAccessibleInterface* node, int depth) {
            if (node == nullptr || depth > 8) {
                return false;
            }
            if (node->object() == m_view) {
                return true;
            }
            for (int i = 0; i < node->childCount(); ++i) {
                if (reach(node->child(i), depth + 1)) {
                    return true;
                }
            }
            return false;
        };
        r.insert(QStringLiteral("na_arvore_da_janela"), reach(QAccessible::queryAccessibleInterface(m_window), 0));
        // A raiz que a AT-SPI publica é a aplicação: as janelas de topo são os
        // filhos dela.
        QAccessibleInterface* app = QAccessible::queryAccessibleInterface(qApp);
        r.insert(QStringLiteral("filhos_da_aplicacao"), app != nullptr ? app->childCount() : -1);
        r.insert(QStringLiteral("na_arvore_da_aplicacao"), reach(app, 0));
        QJsonArray janelas;
        for (QWindow* w : QGuiApplication::topLevelWindows()) {
            QAccessibleInterface* raiz = w->accessibleRoot();
            janelas.append(QStringLiteral("%1 visível=%2 raiz=%3")
                               .arg(QString::fromLatin1(w->metaObject()->className()))
                               .arg(w->isVisible())
                               .arg(raiz != nullptr && raiz->object() != nullptr
                                        ? QString::fromLatin1(raiz->object()->metaObject()->className())
                                        : QStringLiteral("nenhuma")));
        }
        r.insert(QStringLiteral("janelas_de_topo"), janelas);
        reply(socket, id, iface != nullptr, r, QStringLiteral("sem interface acessível"));
    } else if (op == QLatin1String("tinta")) {
        // O desenho contra a grade: em cada célula, há tinta (um pixel longe
        // da cor que domina a célula)? O driver compara com o texto da grade.
        const QImage image = m_window->grabWindow().convertToFormat(QImage::Format_RGB32);
        const qreal cw = m_view->cellWidth();
        const qreal ch = m_view->cellHeight();
        QJsonArray rows;
        const int nrows = static_cast<int>(m_view->gridRows().size());
        for (int r = 0; r < nrows; ++r) {
            QString line;
            for (int c = 0; c < m_view->gridCols(); ++c) {
                const int x0 = static_cast<int>(std::round(c * cw));
                const int x1 = std::min(image.width(), static_cast<int>(std::round((c + 1) * cw)));
                const int y0 = static_cast<int>(r * ch);
                const int y1 = std::min(image.height(), static_cast<int>((r + 1) * ch));
                QHash<QRgb, int> count;
                for (int y = y0; y < y1; ++y) {
                    const auto* px = reinterpret_cast<const QRgb*>(image.constScanLine(y));
                    for (int x = x0; x < x1; ++x) {
                        count[px[x]]++;
                    }
                }
                QRgb mode = 0;
                int best = -1;
                for (auto it = count.cbegin(); it != count.cend(); ++it) {
                    if (it.value() > best) {
                        best = it.value();
                        mode = it.key();
                    }
                }
                bool ink = false;
                for (auto it = count.cbegin(); it != count.cend() && !ink; ++it) {
                    const int d = std::abs(qRed(it.key()) - qRed(mode)) + std::abs(qGreen(it.key()) - qGreen(mode)) +
                                  std::abs(qBlue(it.key()) - qBlue(mode));
                    ink = d > 60;
                }
                line.append(ink ? QLatin1Char('#') : QLatin1Char('.'));
            }
            rows.append(line);
        }
        reply(socket, id, true, QJsonObject{{QStringLiteral("tinta"), rows}, {QStringLiteral("estado"), state()}});
    } else if (op == QLatin1String("area_transferencia")) {
        if (req.contains(QStringLiteral("texto"))) {
            QGuiApplication::clipboard()->setText(req.value(QStringLiteral("texto")).toString());
        }
        reply(socket, id, true, QGuiApplication::clipboard()->text());
    } else if (op == QLatin1String("sair")) {
        reply(socket, id, true, true);
        QTimer::singleShot(0, qApp, &QCoreApplication::quit);
    } else {
        reply(socket, id, false, {}, QStringLiteral("operação desconhecida: %1").arg(op));
    }
}

} // namespace prova
