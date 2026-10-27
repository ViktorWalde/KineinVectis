// RP2040 BOOTSEL queries: response ids belong to the current consumer.
#include "core_client.h"

namespace kinein {

qint64 CoreClient::picotoolList()
{
    return sendRequest(QStringLiteral("picotool.list"), QJsonObject{});
}

bool CoreClient::dispatchPicotoolResult(const QString& method, const QJsonObject& result,
                                        qint64 requestId)
{
    if (method != QStringLiteral("picotool.list")) {
        return false;
    }
    emit picotoolListResolved(result.toVariantMap(), requestId);
    return true;
}

} // namespace kinein
