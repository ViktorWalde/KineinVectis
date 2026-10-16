#include "web_support.h"

#include <QCoreApplication>

namespace kinein {

WebSupport::WebSupport(QObject* parent) : QObject(parent) {}

bool WebSupport::contextShared()
{
    return QCoreApplication::testAttribute(Qt::AA_ShareOpenGLContexts);
}

} // namespace kinein
