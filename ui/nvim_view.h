// O editor Neovim dentro da cena Qt Quick.
//
// Um QQuickItem que fala a UI remota do Neovim (ext_linegrid, ext_popupmenu)
// pelo próprio socket e desenha a grade pelo scene graph: um nó por linha,
// só as linhas sujas refeitas. O texto do documento não mora aqui: a grade é
// o que o Neovim mandou desenhar, e o dono do texto é o Neovim.
#pragma once

#include "nvim_rpc.h"

#include <QColor>
#include <QFont>
#include <QHash>
#include <QPointF>
#include <QQuickItem>
#include <QtQml/qqmlregistration.h>
#include <QStringList>
#include <QVariantList>
#include <QVector>

namespace prova {

class NvimView : public QQuickItem
{
    Q_OBJECT
    QML_ELEMENT
    Q_PROPERTY(QString socketPath READ socketPath WRITE setSocketPath NOTIFY socketPathChanged)
    Q_PROPERTY(bool attached READ attached NOTIFY attachedChanged)
    Q_PROPERTY(QString mode READ mode NOTIFY modeChanged)
    Q_PROPERTY(QVariantList popupItems READ popupItems NOTIFY popupChanged)
    Q_PROPERTY(int popupSelected READ popupSelected NOTIFY popupChanged)
    Q_PROPERTY(bool popupVisible READ popupVisible NOTIFY popupChanged)
    Q_PROPERTY(QPointF popupPos READ popupPos NOTIFY popupChanged)
    Q_PROPERTY(qreal cellHeight READ cellHeight CONSTANT)

public:
    struct HlAttr {
        QColor fg;
        QColor bg;
        QColor sp;
        bool bold = false;
        bool italic = false;
        bool underline = false;
        bool undercurl = false;
        bool strikethrough = false;
        bool reverse = false;
    };
    struct Cell {
        QString text = QStringLiteral(" ");
        int hl = 0;
    };

    explicit NvimView(QQuickItem* parent = nullptr);

    [[nodiscard]] QString socketPath() const { return m_socketPath; }
    void setSocketPath(const QString& path);
    [[nodiscard]] bool attached() const { return m_attached; }
    [[nodiscard]] QString mode() const { return m_mode; }
    [[nodiscard]] QVariantList popupItems() const { return m_popupItems; }
    [[nodiscard]] int popupSelected() const { return m_popupSelected; }
    [[nodiscard]] bool popupVisible() const { return m_popupVisible; }
    [[nodiscard]] QPointF popupPos() const { return m_popupPos; }
    [[nodiscard]] qreal cellHeight() const { return m_cellH; }

    [[nodiscard]] NvimRpc* rpc() { return &m_rpc; }
    [[nodiscard]] QStringList gridRows() const;
    [[nodiscard]] int cursorRow() const { return m_cursorRow; }
    [[nodiscard]] int cursorCol() const { return m_cursorCol; }
    [[nodiscard]] const QFont& font() const { return m_font; }
    [[nodiscard]] qreal cellWidth() const { return m_cellW; }
    [[nodiscard]] int gridCols() const { return m_cols; }

    // O que a acessibilidade lê: a linha do cursor e a coluna, em caracteres,
    // vindas do Neovim pelo `kinein_cursor`.
    [[nodiscard]] QString a11yLine() const { return m_a11yLine; }
    [[nodiscard]] int a11yColumn() const { return m_a11yCol; }
    void requestCursorColumn(int column);

    [[nodiscard]] QVariant inputMethodQuery(Qt::InputMethodQuery query) const override;

signals:
    void socketPathChanged();
    void attachedChanged();
    void modeChanged();
    void popupChanged();
    void flushed();
    void a11yChanged();

protected:
    QSGNode* updatePaintNode(QSGNode* old, UpdatePaintNodeData* data) override;
    void geometryChange(const QRectF& newGeometry, const QRectF& oldGeometry) override;
    void keyPressEvent(QKeyEvent* event) override;
    void inputMethodEvent(QInputMethodEvent* event) override;
    void mousePressEvent(QMouseEvent* event) override;
    void mouseMoveEvent(QMouseEvent* event) override;
    void mouseReleaseEvent(QMouseEvent* event) override;
    void wheelEvent(QWheelEvent* event) override;
    void focusInEvent(QFocusEvent* event) override;
    void focusOutEvent(QFocusEvent* event) override;

private:
    void onConnected();
    void onNotification(const QString& method, const QVariantList& params);
    void onRequest(qint64 id, const QString& method, const QVariantList& params);
    void handleRedraw(const QString& name, const QVariantList& args);
    void resizeGrid(int cols, int rows);
    void markAllDirty();
    void input(const QString& keys);
    void mouse(const QString& button, const QString& action, Qt::KeyboardModifiers mods,
               QPointF pos);
    [[nodiscard]] QString translateKey(const QKeyEvent* event) const;
    [[nodiscard]] QPoint cellAt(QPointF pos) const;
    [[nodiscard]] QColor fgOf(int hl) const;
    [[nodiscard]] QColor bgOf(int hl) const;
    [[nodiscard]] const QFont& fontOf(int hl) const;
    [[nodiscard]] int gridColsForWidth() const;
    [[nodiscard]] int gridRowsForHeight() const;

    NvimRpc m_rpc;
    QString m_socketPath;
    bool m_attached = false;

    QFont m_font;
    QFont m_fontBold;
    QFont m_fontItalic;
    QFont m_fontBoldItalic;
    qreal m_cellW = 8;
    qreal m_cellH = 16;
    qreal m_ascent = 12;

    int m_cols = 0;
    int m_rows = 0;
    QVector<Cell> m_cells;
    QVector<bool> m_dirty;
    bool m_layoutChanged = true;
    QHash<int, HlAttr> m_hl;
    QColor m_defaultFg = QColor(0xd4, 0xd4, 0xd4);
    QColor m_defaultBg = QColor(0x1e, 0x1f, 0x22);
    QColor m_defaultSp = QColor(0xff, 0x55, 0x55);

    int m_cursorRow = 0;
    int m_cursorCol = 0;
    QString m_mode;
    QVariantList m_modeInfo;
    QString m_cursorShape = QStringLiteral("block");
    int m_cursorPercent = 100;
    QString m_preedit;

    QVariantList m_popupItems;
    int m_popupSelected = -1;
    bool m_popupVisible = false;
    QPointF m_popupPos;

    QString m_a11yLine;
    int m_a11yCol = 0;
    bool m_dragging = false;
};

void installNvimAccessibility();

} // namespace prova
