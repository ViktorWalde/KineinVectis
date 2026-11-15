#include "nvim_view.h"

#include <QAccessible>
#include <QAccessibleObject>
#include <QClipboard>
#include <QFontDatabase>
#include <QFontMetricsF>
#include <QGuiApplication>
#include <QInputMethodEvent>
#include <QKeyEvent>
#include <QMouseEvent>
#include <QQuickWindow>
#include <QSGNode>
#include <QSGRectangleNode>
#include <QSGTextNode>
#include <QTextLayout>
#include <QWheelEvent>

#include <cmath>

namespace prova {
namespace {

// A raiz guarda o fundo, uma linha por nó e, por cima, o cursor e a
// composição do IME. Só as linhas sujas são refeitas a cada quadro.
class GridNode : public QSGNode
{
public:
    QSGRectangleNode* background = nullptr;
    QSGNode* rowsParent = nullptr;
    QVector<QSGNode*> rows;
    QSGRectangleNode* cursor = nullptr;
    QSGNode* overlay = nullptr;
};

void deleteChildren(QSGNode* node)
{
    while (QSGNode* child = node->firstChild()) {
        node->removeChildNode(child);
        delete child;
    }
}

bool isSimpleCell(const QString& text)
{
    return text.size() == 1 && text.at(0).unicode() >= 0x20 && text.at(0).unicode() < 0x7f;
}

QColor rgb(const QVariant& v)
{
    return v.isValid() ? QColor::fromRgb(static_cast<QRgb>(v.toLongLong() & 0xffffff)) : QColor();
}

QString specialKeyName(int key)
{
    switch (key) {
    case Qt::Key_Return:
    case Qt::Key_Enter:
        return QStringLiteral("CR");
    case Qt::Key_Backspace:
        return QStringLiteral("BS");
    case Qt::Key_Tab:
    case Qt::Key_Backtab:
        return QStringLiteral("Tab");
    case Qt::Key_Escape:
        return QStringLiteral("Esc");
    case Qt::Key_Delete:
        return QStringLiteral("Del");
    case Qt::Key_Insert:
        return QStringLiteral("Insert");
    case Qt::Key_Home:
        return QStringLiteral("Home");
    case Qt::Key_End:
        return QStringLiteral("End");
    case Qt::Key_PageUp:
        return QStringLiteral("PageUp");
    case Qt::Key_PageDown:
        return QStringLiteral("PageDown");
    case Qt::Key_Up:
        return QStringLiteral("Up");
    case Qt::Key_Down:
        return QStringLiteral("Down");
    case Qt::Key_Left:
        return QStringLiteral("Left");
    case Qt::Key_Right:
        return QStringLiteral("Right");
    case Qt::Key_Space:
        return QStringLiteral("Space");
    default:
        break;
    }
    if (key >= Qt::Key_F1 && key <= Qt::Key_F35) {
        return QStringLiteral("F%1").arg(key - Qt::Key_F1 + 1);
    }
    return {};
}

QString escapeInput(QString text)
{
    return text.replace(QLatin1Char('<'), QStringLiteral("<lt>"));
}

QString mouseModifier(Qt::KeyboardModifiers mods)
{
    QString m;
    if (mods & Qt::ControlModifier) {
        m += QLatin1Char('C');
    }
    if (mods & Qt::ShiftModifier) {
        m += QLatin1Char('S');
    }
    if (mods & Qt::AltModifier) {
        m += QLatin1Char('A');
    }
    return m;
}

} // namespace

NvimView::NvimView(QQuickItem* parent) : QQuickItem(parent)
{
    setFlag(ItemHasContents);
    setFlag(ItemAcceptsInputMethod);
    setFlag(ItemIsFocusScope, false);
    setAcceptedMouseButtons(Qt::AllButtons);

    // A fonte do editor de hoje (Theme.monoFont e fontSizeEditor): a família
    // genérica, que o fontconfig resolve para uma monoespaçada de verdade. A
    // "fonte fixa do sistema" não serve: sem tema de plataforma (offscreen),
    // ela cai numa proporcional.
    m_font = QFont(QStringLiteral("monospace"));
    bool ok = false;
    const int size = qEnvironmentVariable("KINEIN_PROVA_FONT_PX").toInt(&ok);
    m_font.setPixelSize(ok && size > 0 ? size : 14);
    m_font.setStyleHint(QFont::Monospace);
    m_font.setFixedPitch(true);
    m_font.setKerning(false);
    m_fontBold = m_font;
    m_fontBold.setBold(true);
    m_fontItalic = m_font;
    m_fontItalic.setItalic(true);
    m_fontBoldItalic = m_fontBold;
    m_fontBoldItalic.setItalic(true);
    const QFontMetricsF fm(m_font);
    m_cellW = fm.horizontalAdvance(QLatin1Char('M'));
    m_cellH = std::ceil(fm.lineSpacing());
    m_ascent = fm.ascent();

    connect(&m_rpc, &NvimRpc::connected, this, &NvimView::onConnected);
    connect(&m_rpc, &NvimRpc::disconnected, this, [this] {
        m_attached = false;
        emit attachedChanged();
    });
    connect(&m_rpc, &NvimRpc::notificationReceived, this, &NvimView::onNotification);
    connect(&m_rpc, &NvimRpc::requestReceived, this, &NvimView::onRequest);
}

void NvimView::setSocketPath(const QString& path)
{
    if (path == m_socketPath && m_rpc.isConnected()) {
        return;
    }
    m_socketPath = path;
    emit socketPathChanged();
    m_attached = false;
    emit attachedChanged();
    if (!path.isEmpty()) {
        m_rpc.connectTo(path);
    }
}

QStringList NvimView::gridRows() const
{
    QStringList rows;
    for (int r = 0; r < m_rows; ++r) {
        QString line;
        for (int c = 0; c < m_cols; ++c) {
            line += m_cells.at(r * m_cols + c).text;
        }
        rows.append(line);
    }
    return rows;
}

int NvimView::gridColsForWidth() const
{
    return std::max(20, static_cast<int>(width() / m_cellW));
}

int NvimView::gridRowsForHeight() const
{
    return std::max(5, static_cast<int>(height() / m_cellH));
}

void NvimView::onConnected()
{
    m_rpc.request(QStringLiteral("nvim_get_api_info"), {}, [this](const QVariant& err, const QVariant& res) {
        if (err.isValid()) {
            return;
        }
        const qint64 chan = res.toList().value(0).toLongLong();
        m_rpc.request(QStringLiteral("nvim_set_var"), {QStringLiteral("kinein_ui_chan"), chan});
        const QVariantMap options{{QStringLiteral("rgb"), true},
                                  {QStringLiteral("ext_linegrid"), true},
                                  {QStringLiteral("ext_popupmenu"), true}};
        const int cols = width() > 0 ? gridColsForWidth() : 100;
        const int rows = height() > 0 ? gridRowsForHeight() : 30;
        m_rpc.request(QStringLiteral("nvim_ui_attach"), {cols, rows, options},
                      [this](const QVariant& attachErr, const QVariant&) {
                          m_attached = !attachErr.isValid();
                          emit attachedChanged();
                          if (hasActiveFocus()) {
                              m_rpc.request(QStringLiteral("nvim_ui_set_focus"), {true});
                          }
                      });
    });
}

void NvimView::onNotification(const QString& method, const QVariantList& params)
{
    if (method == QLatin1String("redraw")) {
        for (const QVariant& batch : params) {
            const QVariantList event = batch.toList();
            if (event.isEmpty()) {
                continue;
            }
            const QString name = event.at(0).toString();
            for (qsizetype i = 1; i < event.size(); ++i) {
                handleRedraw(name, event.at(i).toList());
            }
        }
        return;
    }
    if (method == QLatin1String("kinein_cursor") && params.size() >= 3) {
        m_a11yCol = params.at(1).toInt();
        m_a11yLine = params.at(2).toString();
        emit a11yChanged();
        if (QAccessible::isActive()) {
            QAccessibleTextCursorEvent ev(this, m_a11yCol);
            QAccessible::updateAccessibility(&ev);
        }
    }
}

void NvimView::onRequest(qint64 id, const QString& method, const QVariantList& params)
{
    QClipboard* clipboard = QGuiApplication::clipboard();
    if (method == QLatin1String("kinein_clipboard_set")) {
        QStringList lines;
        for (const QVariant& l : params.value(0).toList()) {
            lines.append(l.toString());
        }
        clipboard->setText(lines.join(QLatin1Char('\n')));
        m_rpc.respond(id, {}, true);
        return;
    }
    if (method == QLatin1String("kinein_clipboard_get")) {
        QVariantList lines;
        for (const QString& l : clipboard->text().split(QLatin1Char('\n'))) {
            lines.append(l);
        }
        m_rpc.respond(id, {}, QVariantList{lines, QStringLiteral("v")});
        return;
    }
    m_rpc.respond(id, QStringLiteral("método desconhecido da UI: %1").arg(method), {});
}

void NvimView::resizeGrid(int cols, int rows)
{
    QVector<Cell> cells(static_cast<qsizetype>(cols) * rows);
    for (int r = 0; r < std::min(rows, m_rows); ++r) {
        for (int c = 0; c < std::min(cols, m_cols); ++c) {
            cells[r * cols + c] = m_cells.at(r * m_cols + c);
        }
    }
    m_cols = cols;
    m_rows = rows;
    m_cells = std::move(cells);
    m_layoutChanged = true;
    markAllDirty();
}

void NvimView::markAllDirty()
{
    m_dirty.fill(true, m_rows);
}

void NvimView::handleRedraw(const QString& name, const QVariantList& args)
{
    if (name == QLatin1String("grid_line")) {
        const int row = args.value(1).toInt();
        int col = args.value(2).toInt();
        if (row < 0 || row >= m_rows) {
            return;
        }
        int hl = 0;
        for (const QVariant& cv : args.value(3).toList()) {
            const QVariantList c = cv.toList();
            const QString text = c.value(0).toString();
            if (c.size() > 1) {
                hl = c.at(1).toInt();
            }
            const int repeat = c.size() > 2 ? c.at(2).toInt() : 1;
            for (int i = 0; i < repeat && col < m_cols; ++i, ++col) {
                m_cells[row * m_cols + col] = Cell{text, hl};
            }
        }
        m_dirty[row] = true;
    } else if (name == QLatin1String("grid_cursor_goto")) {
        if (m_cursorRow >= 0 && m_cursorRow < m_rows) {
            m_dirty[m_cursorRow] = true;
        }
        m_cursorRow = args.value(1).toInt();
        m_cursorCol = args.value(2).toInt();
    } else if (name == QLatin1String("grid_scroll")) {
        const int top = args.value(1).toInt();
        const int bot = args.value(2).toInt();
        const int left = args.value(3).toInt();
        const int right = args.value(4).toInt();
        const int rows = args.value(5).toInt();
        auto copyRow = [&](int to, int from) {
            for (int c = left; c < right; ++c) {
                m_cells[to * m_cols + c] = m_cells.at(from * m_cols + c);
            }
        };
        if (rows > 0) {
            for (int r = top; r < bot - rows; ++r) {
                copyRow(r, r + rows);
            }
        } else {
            for (int r = bot - 1; r >= top - rows; --r) {
                copyRow(r, r + rows);
            }
        }
        for (int r = top; r < bot && r < m_rows; ++r) {
            m_dirty[r] = true;
        }
    } else if (name == QLatin1String("flush")) {
        update();
        emit flushed();
    } else if (name == QLatin1String("hl_attr_define")) {
        const QVariantMap a = args.value(1).toMap();
        HlAttr h;
        h.fg = rgb(a.value(QStringLiteral("foreground")));
        h.bg = rgb(a.value(QStringLiteral("background")));
        h.sp = rgb(a.value(QStringLiteral("special")));
        h.bold = a.value(QStringLiteral("bold")).toBool();
        h.italic = a.value(QStringLiteral("italic")).toBool();
        h.underline = a.value(QStringLiteral("underline")).toBool();
        h.undercurl = a.value(QStringLiteral("undercurl")).toBool();
        h.strikethrough = a.value(QStringLiteral("strikethrough")).toBool();
        h.reverse = a.value(QStringLiteral("reverse")).toBool();
        m_hl.insert(args.value(0).toInt(), h);
    } else if (name == QLatin1String("default_colors_set")) {
        const QColor fg = rgb(args.value(0));
        const QColor bg = rgb(args.value(1));
        const QColor sp = rgb(args.value(2));
        if (args.value(0).toLongLong() >= 0 && fg.isValid()) {
            m_defaultFg = fg;
        }
        if (args.value(1).toLongLong() >= 0 && bg.isValid()) {
            m_defaultBg = bg;
        }
        if (args.value(2).toLongLong() >= 0 && sp.isValid()) {
            m_defaultSp = sp;
        }
        markAllDirty();
    } else if (name == QLatin1String("grid_resize")) {
        resizeGrid(args.value(1).toInt(), args.value(2).toInt());
    } else if (name == QLatin1String("grid_clear")) {
        m_cells.fill(Cell{});
        markAllDirty();
    } else if (name == QLatin1String("mode_info_set")) {
        m_modeInfo = args.value(1).toList();
    } else if (name == QLatin1String("mode_change")) {
        m_mode = args.value(0).toString();
        const QVariantMap info = m_modeInfo.value(args.value(1).toInt()).toMap();
        m_cursorShape = info.value(QStringLiteral("cursor_shape"), QStringLiteral("block")).toString();
        m_cursorPercent = info.value(QStringLiteral("cell_percentage"), 100).toInt();
        emit modeChanged();
    } else if (name == QLatin1String("popupmenu_show")) {
        m_popupItems.clear();
        for (const QVariant& item : args.value(0).toList()) {
            const QVariantList i = item.toList();
            m_popupItems.append(QVariantMap{{QStringLiteral("word"), i.value(0)},
                                            {QStringLiteral("kind"), i.value(1)},
                                            {QStringLiteral("menu"), i.value(2)}});
        }
        m_popupSelected = args.value(1).toInt();
        m_popupPos = QPointF(args.value(3).toInt() * m_cellW, (args.value(2).toInt() + 1) * m_cellH);
        m_popupVisible = true;
        emit popupChanged();
    } else if (name == QLatin1String("popupmenu_select")) {
        m_popupSelected = args.value(0).toInt();
        emit popupChanged();
    } else if (name == QLatin1String("popupmenu_hide")) {
        m_popupVisible = false;
        emit popupChanged();
    }
}

QColor NvimView::fgOf(int hl) const
{
    const HlAttr a = m_hl.value(hl);
    const QColor fg = a.fg.isValid() ? a.fg : m_defaultFg;
    const QColor bg = a.bg.isValid() ? a.bg : m_defaultBg;
    return a.reverse ? bg : fg;
}

QColor NvimView::bgOf(int hl) const
{
    const HlAttr a = m_hl.value(hl);
    const QColor fg = a.fg.isValid() ? a.fg : m_defaultFg;
    const QColor bg = a.bg.isValid() ? a.bg : m_defaultBg;
    return a.reverse ? fg : bg;
}

const QFont& NvimView::fontOf(int hl) const
{
    const HlAttr a = m_hl.value(hl);
    if (a.bold && a.italic) {
        return m_fontBoldItalic;
    }
    if (a.bold) {
        return m_fontBold;
    }
    return a.italic ? m_fontItalic : m_font;
}

QSGNode* NvimView::updatePaintNode(QSGNode* old, UpdatePaintNodeData*)
{
    QQuickWindow* w = window();
    auto* root = static_cast<GridNode*>(old);
    if (root == nullptr) {
        root = new GridNode;
        root->background = w->createRectangleNode();
        root->appendChildNode(root->background);
        root->rowsParent = new QSGNode;
        root->appendChildNode(root->rowsParent);
        root->cursor = w->createRectangleNode();
        root->appendChildNode(root->cursor);
        root->overlay = new QSGNode;
        root->appendChildNode(root->overlay);
        m_layoutChanged = true;
    }
    root->background->setRect(boundingRect());
    root->background->setColor(m_defaultBg);

    if (m_layoutChanged) {
        deleteChildren(root->rowsParent);
        root->rows.clear();
        for (int r = 0; r < m_rows; ++r) {
            auto* row = new QSGNode;
            root->rowsParent->appendChildNode(row);
            root->rows.append(row);
        }
        markAllDirty();
        m_layoutChanged = false;
    }

    auto addText = [&](QSGNode* parent, const QString& text, int hl, qreal x, qreal y) {
        QSGTextNode* node = w->createTextNode();
        node->setColor(fgOf(hl));
        QTextLayout layout(text, fontOf(hl));
        layout.setCacheEnabled(true);
        layout.beginLayout();
        QTextLine line = layout.createLine();
        line.setLineWidth(1.0e6);
        layout.endLayout();
        node->addTextLayout(QPointF(x, y + (m_cellH - line.height()) / 2.0), &layout);
        parent->appendChildNode(node);
        const HlAttr a = m_hl.value(hl);
        if (a.underline || a.undercurl || a.strikethrough) {
            QSGRectangleNode* deco = w->createRectangleNode();
            const qreal width = line.naturalTextWidth();
            const qreal yy = a.strikethrough ? y + m_cellH / 2.0 : y + m_cellH - 1.5;
            deco->setRect(QRectF(x, yy, width, 1.0));
            deco->setColor(a.undercurl ? (a.sp.isValid() ? a.sp : m_defaultSp) : fgOf(hl));
            parent->appendChildNode(deco);
        }
    };

    for (int r = 0; r < m_rows && r < root->rows.size(); ++r) {
        if (!m_dirty.value(r)) {
            continue;
        }
        QSGNode* row = root->rows.at(r);
        deleteChildren(row);
        const qreal y = r * m_cellH;
        for (int c = 0; c < m_cols;) {
            const QColor bg = bgOf(m_cells.at(r * m_cols + c).hl);
            const int start = c;
            while (c < m_cols && bgOf(m_cells.at(r * m_cols + c).hl) == bg) {
                ++c;
            }
            if (bg != m_defaultBg) {
                QSGRectangleNode* rect = w->createRectangleNode();
                rect->setRect(QRectF(start * m_cellW, y, (c - start) * m_cellW, m_cellH));
                rect->setColor(bg);
                row->appendChildNode(rect);
            }
        }
        for (int c = 0; c < m_cols;) {
            const Cell& cell = m_cells.at(r * m_cols + c);
            if (cell.text.isEmpty() || cell.text == QLatin1String(" ")) {
                ++c;
                continue;
            }
            const int start = c;
            QString run = cell.text;
            ++c;
            if (isSimpleCell(cell.text)) {
                while (c < m_cols && m_cells.at(r * m_cols + c).hl == cell.hl &&
                       isSimpleCell(m_cells.at(r * m_cols + c).text)) {
                    run += m_cells.at(r * m_cols + c).text;
                    ++c;
                }
            }
            addText(row, run, cell.hl, start * m_cellW, y);
        }
    }
    m_dirty.fill(false, m_rows);

    const qreal cx = m_cursorCol * m_cellW;
    const qreal cy = m_cursorRow * m_cellH;
    QRectF cursorRect(cx, cy, m_cellW, m_cellH);
    if (m_cursorShape == QLatin1String("vertical")) {
        cursorRect.setWidth(std::max(1.0, m_cellW * m_cursorPercent / 100.0));
    } else if (m_cursorShape == QLatin1String("horizontal")) {
        const qreal h = std::max(1.0, m_cellH * m_cursorPercent / 100.0);
        cursorRect = QRectF(cx, cy + m_cellH - h, m_cellW, h);
    }
    QColor cursorColor = m_defaultFg;
    cursorColor.setAlphaF(m_cursorShape == QLatin1String("block") ? 0.5F : 1.0F);
    root->cursor->setRect(m_preedit.isEmpty() ? cursorRect : QRectF());
    root->cursor->setColor(cursorColor);

    deleteChildren(root->overlay);
    if (!m_preedit.isEmpty()) {
        const qreal width = QFontMetricsF(m_font).horizontalAdvance(m_preedit);
        QSGRectangleNode* back = w->createRectangleNode();
        back->setRect(QRectF(cx, cy, width, m_cellH));
        back->setColor(m_defaultBg);
        root->overlay->appendChildNode(back);
        addText(root->overlay, m_preedit, 0, cx, cy);
        QSGRectangleNode* underline = w->createRectangleNode();
        underline->setRect(QRectF(cx, cy + m_cellH - 1.5, width, 1.0));
        underline->setColor(m_defaultFg);
        root->overlay->appendChildNode(underline);
    }
    return root;
}

void NvimView::geometryChange(const QRectF& newGeometry, const QRectF& oldGeometry)
{
    QQuickItem::geometryChange(newGeometry, oldGeometry);
    if (m_attached && newGeometry.size() != oldGeometry.size()) {
        m_rpc.request(QStringLiteral("nvim_ui_try_resize"), {gridColsForWidth(), gridRowsForHeight()});
    }
}

void NvimView::input(const QString& keys)
{
    // Notificação, não pedido: a tecla não espera resposta, e a ordem no canal
    // é a mesma.
    if (!keys.isEmpty() && m_rpc.isConnected()) {
        m_rpc.notify(QStringLiteral("nvim_input"), {keys});
    }
}

// Pelo ponto de código, não pela unidade UTF-16: um emoji chega como par
// substituto, e a primeira metade sozinha não é imprimível.
static bool startsPrintable(const QString& text)
{
    const QList<uint> codes = text.toUcs4();
    return !codes.isEmpty() && QChar::isPrint(static_cast<char32_t>(codes.first()));
}

QString NvimView::translateKey(const QKeyEvent* event) const
{
    const Qt::KeyboardModifiers mods = event->modifiers();
    const int key = event->key();
    const bool ctrl = mods & Qt::ControlModifier;
    const bool alt = mods & Qt::AltModifier;
    const bool meta = mods & Qt::MetaModifier;
    const bool shift = (mods & Qt::ShiftModifier) || key == Qt::Key_Backtab;

    const QString special = specialKeyName(key);
    if (!special.isEmpty()) {
        QString prefix;
        prefix += ctrl ? QStringLiteral("C-") : QString();
        prefix += shift ? QStringLiteral("S-") : QString();
        prefix += alt ? QStringLiteral("M-") : QString();
        prefix += meta ? QStringLiteral("D-") : QString();
        if (prefix.isEmpty() && key == Qt::Key_Space) {
            return QStringLiteral(" ");
        }
        return QStringLiteral("<%1%2>").arg(prefix, special);
    }
    if (key == Qt::Key_Shift || key == Qt::Key_Control || key == Qt::Key_Alt || key == Qt::Key_Meta ||
        key == Qt::Key_AltGr) {
        return {};
    }
    if (ctrl || alt || meta) {
        QString base;
        if (key >= Qt::Key_A && key <= Qt::Key_Z) {
            base = QChar(QLatin1Char('a').unicode() + (key - Qt::Key_A));
        } else if (key >= Qt::Key_0 && key <= Qt::Key_9) {
            base = QChar(QLatin1Char('0').unicode() + (key - Qt::Key_0));
        } else if (key < 0x7f && key > 0x20) {
            base = QChar(key);
        } else if (startsPrintable(event->text())) {
            base = event->text();
        } else {
            return {};
        }
        if (base == QLatin1String("<")) {
            base = QStringLiteral("lt");
        }
        QString prefix;
        prefix += ctrl ? QStringLiteral("C-") : QString();
        // Shift só conta para letra: num símbolo ele já mudou o caractere.
        prefix += (shift && key >= Qt::Key_A && key <= Qt::Key_Z) ? QStringLiteral("S-") : QString();
        prefix += alt ? QStringLiteral("M-") : QString();
        prefix += meta ? QStringLiteral("D-") : QString();
        return QStringLiteral("<%1%2>").arg(prefix, base);
    }
    const QString text = event->text();
    if (!startsPrintable(text)) {
        return {};
    }
    return escapeInput(text);
}

void NvimView::keyPressEvent(QKeyEvent* event)
{
    const bool paste = (event->modifiers() == Qt::ControlModifier && event->key() == Qt::Key_V) ||
                       (event->modifiers() == Qt::ShiftModifier && event->key() == Qt::Key_Insert);
    if (paste) {
        // O caminho de colar das GUIs do Neovim: nvim_paste, sem recuo
        // automático e com o cursor depois do texto; na seleção, substitui.
        m_rpc.request(QStringLiteral("nvim_paste"), {QGuiApplication::clipboard()->text(), true, -1});
        event->accept();
        return;
    }
    const QString keys = translateKey(event);
    if (keys.isEmpty()) {
        event->ignore();
        return;
    }
    input(keys);
    event->accept();
}

void NvimView::inputMethodEvent(QInputMethodEvent* event)
{
    if (!event->commitString().isEmpty()) {
        input(escapeInput(event->commitString()));
    }
    m_preedit = event->preeditString();
    update();
    event->accept();
}

QVariant NvimView::inputMethodQuery(Qt::InputMethodQuery query) const
{
    switch (query) {
    case Qt::ImEnabled:
        return true;
    case Qt::ImCursorRectangle:
        return QRectF(m_cursorCol * m_cellW, m_cursorRow * m_cellH, m_cellW, m_cellH);
    case Qt::ImFont:
        return m_font;
    case Qt::ImCursorPosition:
        return m_a11yCol;
    case Qt::ImSurroundingText:
        return m_a11yLine;
    case Qt::ImHints:
        return static_cast<int>(Qt::ImhNone);
    default:
        return QQuickItem::inputMethodQuery(query);
    }
}

QPoint NvimView::cellAt(QPointF pos) const
{
    return {std::clamp(static_cast<int>(pos.x() / m_cellW), 0, std::max(0, m_cols - 1)),
            std::clamp(static_cast<int>(pos.y() / m_cellH), 0, std::max(0, m_rows - 1))};
}

void NvimView::mouse(const QString& button, const QString& action, Qt::KeyboardModifiers mods,
                     QPointF pos)
{
    const QPoint cell = cellAt(pos);
    m_rpc.notify(QStringLiteral("nvim_input_mouse"),
                 {button, action, mouseModifier(mods), 0, cell.y(), cell.x()});
}

void NvimView::mousePressEvent(QMouseEvent* event)
{
    forceActiveFocus(Qt::MouseFocusReason);
    const QString button = event->button() == Qt::RightButton    ? QStringLiteral("right")
                           : event->button() == Qt::MiddleButton ? QStringLiteral("middle")
                                                                 : QStringLiteral("left");
    m_dragging = true;
    mouse(button, QStringLiteral("press"), event->modifiers(), event->position());
    event->accept();
}

void NvimView::mouseMoveEvent(QMouseEvent* event)
{
    if (m_dragging) {
        mouse(QStringLiteral("left"), QStringLiteral("drag"), event->modifiers(), event->position());
    }
    event->accept();
}

void NvimView::mouseReleaseEvent(QMouseEvent* event)
{
    m_dragging = false;
    mouse(QStringLiteral("left"), QStringLiteral("release"), event->modifiers(), event->position());
    event->accept();
}

void NvimView::wheelEvent(QWheelEvent* event)
{
    const int dy = event->angleDelta().y();
    if (dy != 0) {
        mouse(QStringLiteral("wheel"), dy > 0 ? QStringLiteral("up") : QStringLiteral("down"),
              event->modifiers(), event->position());
    }
    event->accept();
}

void NvimView::focusInEvent(QFocusEvent* event)
{
    QQuickItem::focusInEvent(event);
    if (m_attached) {
        m_rpc.request(QStringLiteral("nvim_ui_set_focus"), {true});
    }
}

void NvimView::focusOutEvent(QFocusEvent* event)
{
    QQuickItem::focusOutEvent(event);
    if (m_attached) {
        m_rpc.request(QStringLiteral("nvim_ui_set_focus"), {false});
    }
}

void NvimView::requestCursorColumn(int column)
{
    m_rpc.request(QStringLiteral("nvim_exec_lua"),
                  {QStringLiteral("return require('kinein').set_cursor_char(...)"), QVariantList{column}});
}

namespace {

// A acessibilidade do editor: o texto da linha do cursor e a posição, lidos do
// Neovim. É o mínimo que um leitor de tela precisa para acompanhar a edição.
class NvimAccessible : public QAccessibleObject, public QAccessibleTextInterface
{
public:
    explicit NvimAccessible(NvimView* view) : QAccessibleObject(view) {}

    [[nodiscard]] NvimView* view() const { return static_cast<NvimView*>(object()); }

    [[nodiscard]] QAccessibleInterface* parent() const override
    {
        // O pai na árvore acessível é a janela, como no QAccessibleQuickItem de
        // um item sem ancestral acessível.
        QQuickWindow* w = view()->window();
        return w != nullptr ? QAccessible::queryAccessibleInterface(w) : nullptr;
    }
    [[nodiscard]] QAccessibleInterface* child(int) const override { return nullptr; }
    [[nodiscard]] int childCount() const override { return 0; }
    [[nodiscard]] int indexOfChild(const QAccessibleInterface*) const override { return -1; }
    [[nodiscard]] QString text(QAccessible::Text t) const override
    {
        if (t == QAccessible::Name) {
            return QStringLiteral("Editor");
        }
        return t == QAccessible::Value ? view()->a11yLine() : QString();
    }
    [[nodiscard]] QAccessible::Role role() const override { return QAccessible::EditableText; }
    [[nodiscard]] QAccessible::State state() const override
    {
        QAccessible::State s;
        s.focusable = true;
        s.focused = view()->hasActiveFocus();
        s.editable = true;
        s.multiLine = true;
        return s;
    }
    [[nodiscard]] QRect rect() const override
    {
        const QRectF r = view()->mapRectToScene(view()->boundingRect());
        QWindow* w = view()->window();
        return w ? QRect(w->mapToGlobal(r.topLeft().toPoint()), r.size().toSize()) : QRect();
    }
    void* interface_cast(QAccessible::InterfaceType t) override
    {
        if (t == QAccessible::TextInterface) {
            return static_cast<QAccessibleTextInterface*>(this);
        }
        return QAccessibleObject::interface_cast(t);
    }

    void selection(int, int* start, int* end) const override { *start = *end = 0; }
    [[nodiscard]] int selectionCount() const override { return 0; }
    void addSelection(int, int) override {}
    void removeSelection(int) override {}
    void setSelection(int, int, int) override {}
    [[nodiscard]] int cursorPosition() const override { return view()->a11yColumn(); }
    void setCursorPosition(int position) override { view()->requestCursorColumn(position); }
    [[nodiscard]] QString text(int start, int end) const override
    {
        return view()->a11yLine().mid(start, end - start);
    }
    [[nodiscard]] int characterCount() const override
    {
        return static_cast<int>(view()->a11yLine().size());
    }
    [[nodiscard]] QRect characterRect(int) const override { return {}; }
    [[nodiscard]] int offsetAtPoint(const QPoint&) const override { return -1; }
    void scrollToSubstring(int, int) override {}
    [[nodiscard]] QString attributes(int, int* start, int* end) const override
    {
        *start = *end = 0;
        return {};
    }
};

QAccessibleInterface* nvimAccessibleFactory(const QString& className, QObject* object)
{
    if (className == QLatin1String("prova::NvimView") && object != nullptr) {
        return new NvimAccessible(static_cast<NvimView*>(object));
    }
    return nullptr;
}

} // namespace

void installNvimAccessibility()
{
    QAccessible::installFactory(nvimAccessibleFactory);
}

} // namespace prova
