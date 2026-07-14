#include "documentation.h"

#include <QFile>
#include <QIODevice>

namespace kinein {

Documentation::Documentation(QObject* parent) : QObject(parent)
{
    QFile manual(QStringLiteral(":/KineinVectis/MANUAL.md"));
    if (manual.open(QIODevice::ReadOnly | QIODevice::Text)) {
        m_manualMarkdown = QString::fromUtf8(manual.readAll());
    }
    else {
        m_manualMarkdown = QStringLiteral(
            "# Manual indisponível\n\nO recurso `MANUAL.md` não foi incluído nesta build.");
    }
}

QString Documentation::manualMarkdown() const
{
    return m_manualMarkdown;
}

} // namespace kinein
