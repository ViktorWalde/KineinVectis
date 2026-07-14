// Bundled product documentation exposed read-only to QML.

#pragma once

#include <QObject>
#include <QString>
#include <QtQml/qqmlregistration.h>

namespace kinein {

class Documentation : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    QML_SINGLETON
    Q_PROPERTY(QString manualMarkdown READ manualMarkdown CONSTANT)

public:
    explicit Documentation(QObject* parent = nullptr);

    [[nodiscard]] QString manualMarkdown() const;

private:
    QString m_manualMarkdown;
};

} // namespace kinein
