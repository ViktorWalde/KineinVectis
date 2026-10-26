// USB DFU discovery: the consumer correlates success and failure by request id.
#include "core_client.h"

namespace kinein {

qint64 CoreClient::dfuList()
{
    return sendRequest(QStringLiteral("dfu.list"), QJsonObject{});
}

bool CoreClient::dispatchDfuResult(const QString& method, const QJsonObject& result,
                                   qint64 requestId)
{
    if (method != QStringLiteral("dfu.list")) {
        return false;
    }
    emit dfuListResolved(result.toVariantMap(), requestId);
    return true;
}

} // namespace kinein
