#include "log_redaction.h"

#include <QJsonArray>
#include <QJsonValue>
#include <QStringList>

namespace kinein {

namespace {

QJsonValue redactValue(const QJsonValue& value);

QJsonObject redactObject(const QJsonObject& object)
{
    QJsonObject limpo;
    for (auto it = object.constBegin(); it != object.constEnd(); ++it) {
        limpo.insert(it.key(), isSecretField(it.key()) ? QJsonValue(QStringLiteral("***"))
                                                       : redactValue(it.value()));
    }
    return limpo;
}

QJsonValue redactValue(const QJsonValue& value)
{
    if (value.isObject()) {
        return redactObject(value.toObject());
    }
    if (value.isArray()) {
        QJsonArray limpo;
        const QJsonArray original = value.toArray();
        for (const auto item : original) {
            limpo.append(redactValue(item));
        }
        return limpo;
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
