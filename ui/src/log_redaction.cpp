#include "log_redaction.h"

#include <QJsonArray>
#include <QJsonValue>
#include <QStringList>

namespace kinein {

namespace {

QJsonValue redactValue(const QJsonValue& value);

QJsonObject redactObject(const QJsonObject& object)
{
    QJsonObject redacted;
    for (auto it = object.constBegin(); it != object.constEnd(); ++it) {
        redacted.insert(it.key(), isSecretField(it.key()) ? QJsonValue(QStringLiteral("***"))
                                                          : redactValue(it.value()));
    }
    return redacted;
}

QJsonValue redactValue(const QJsonValue& value)
{
    if (value.isObject()) {
        return redactObject(value.toObject());
    }
    if (value.isArray()) {
        QJsonArray redacted;
        const QJsonArray original = value.toArray();
        for (const auto item : original) {
            redacted.append(redactValue(item));
        }
        return redacted;
    }
    return value;
}

} // namespace

bool isSecretField(const QString& key)
{
    static const QStringList kSecretKeys{
        QStringLiteral("password"), QStringLiteral("passwd"),     QStringLiteral("secret"),
        QStringLiteral("token"),    QStringLiteral("credential"),
    };
    return kSecretKeys.contains(key, Qt::CaseInsensitive);
}

QJsonObject redactSecrets(const QJsonObject& request)
{
    return redactObject(request);
}

} // namespace kinein
