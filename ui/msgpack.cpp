#include "msgpack.h"

#include <QString>
#include <QVariantList>
#include <QVariantMap>

#include <cstring>

namespace prova::msgpack {
namespace {

void putBE(QByteArray& out, quint64 v, int bytes)
{
    for (int i = bytes - 1; i >= 0; --i) {
        out.append(static_cast<char>((v >> (8 * i)) & 0xff));
    }
}

void encodeInt(QByteArray& out, qint64 v)
{
    if (v >= 0) {
        const auto u = static_cast<quint64>(v);
        if (u < 128) {
            out.append(static_cast<char>(u));
        } else if (u <= 0xff) {
            out.append(static_cast<char>(0xcc));
            putBE(out, u, 1);
        } else if (u <= 0xffff) {
            out.append(static_cast<char>(0xcd));
            putBE(out, u, 2);
        } else if (u <= 0xffffffffULL) {
            out.append(static_cast<char>(0xce));
            putBE(out, u, 4);
        } else {
            out.append(static_cast<char>(0xcf));
            putBE(out, u, 8);
        }
        return;
    }
    if (v >= -32) {
        out.append(static_cast<char>(v));
    } else if (v >= -128) {
        out.append(static_cast<char>(0xd0));
        putBE(out, static_cast<quint64>(v), 1);
    } else if (v >= -32768) {
        out.append(static_cast<char>(0xd1));
        putBE(out, static_cast<quint64>(v), 2);
    } else if (v >= -2147483648LL) {
        out.append(static_cast<char>(0xd2));
        putBE(out, static_cast<quint64>(v), 4);
    } else {
        out.append(static_cast<char>(0xd3));
        putBE(out, static_cast<quint64>(v), 8);
    }
}

void encodeBytes(QByteArray& out, const QByteArray& bytes, bool asStr)
{
    const auto n = static_cast<quint64>(bytes.size());
    if (asStr && n < 32) {
        out.append(static_cast<char>(0xa0 | n));
    } else if (n <= 0xff) {
        out.append(static_cast<char>(asStr ? 0xd9 : 0xc4));
        putBE(out, n, 1);
    } else if (n <= 0xffff) {
        out.append(static_cast<char>(asStr ? 0xda : 0xc5));
        putBE(out, n, 2);
    } else {
        out.append(static_cast<char>(asStr ? 0xdb : 0xc6));
        putBE(out, n, 4);
    }
    out.append(bytes);
}

void encodeHeader(QByteArray& out, quint64 n, quint8 fix, quint8 fixMax, quint8 c16, quint8 c32)
{
    if (n <= fixMax) {
        out.append(static_cast<char>(fix | n));
    } else if (n <= 0xffff) {
        out.append(static_cast<char>(c16));
        putBE(out, n, 2);
    } else {
        out.append(static_cast<char>(c32));
        putBE(out, n, 4);
    }
}

// Leitor com bandeira de "faltam bytes": evita exceções e mantém `pos` intacto
// quando a mensagem ainda está chegando.
struct Reader {
    const QByteArray& data;
    qsizetype pos;
    bool short_ = false;

    bool need(qsizetype n)
    {
        if (pos + n > data.size()) {
            short_ = true;
            return false;
        }
        return true;
    }
    quint64 be(int bytes)
    {
        if (!need(bytes)) {
            return 0;
        }
        quint64 v = 0;
        for (int i = 0; i < bytes; ++i) {
            v = (v << 8) | static_cast<quint8>(data[pos + i]);
        }
        pos += bytes;
        return v;
    }
    QByteArray take(qsizetype n)
    {
        if (!need(n)) {
            return {};
        }
        QByteArray b = data.mid(pos, n);
        pos += n;
        return b;
    }

    QVariant value();
    QVariant array(quint64 n);
    QVariant map(quint64 n);
    QVariant ext(qsizetype n);
};

QVariant Reader::array(quint64 n)
{
    QVariantList list;
    list.reserve(static_cast<qsizetype>(n));
    for (quint64 i = 0; i < n && !short_; ++i) {
        list.append(value());
    }
    return list;
}

QVariant Reader::map(quint64 n)
{
    QVariantMap m;
    for (quint64 i = 0; i < n && !short_; ++i) {
        const QVariant k = value();
        const QVariant v = value();
        m.insert(k.toString(), v);
    }
    return m;
}

QVariant Reader::ext(qsizetype n)
{
    if (!need(1 + n)) {
        return {};
    }
    NvimExt e;
    e.type = static_cast<qint8>(data[pos]);
    ++pos;
    const QByteArray payload = take(n);
    qsizetype inner = 0;
    Reader r{payload, inner};
    e.handle = r.value().toLongLong();
    return QVariant::fromValue(e);
}

QVariant Reader::value()
{
    if (!need(1)) {
        return {};
    }
    const auto b = static_cast<quint8>(data[pos]);
    ++pos;
    if (b <= 0x7f) {
        return static_cast<qint64>(b);
    }
    if (b >= 0xe0) {
        return static_cast<qint64>(static_cast<qint8>(b));
    }
    if ((b & 0xf0) == 0x80) {
        return map(b & 0x0f);
    }
    if ((b & 0xf0) == 0x90) {
        return array(b & 0x0f);
    }
    if ((b & 0xe0) == 0xa0) {
        return QString::fromUtf8(take(b & 0x1f));
    }
    switch (b) {
    case 0xc0:
        return {};
    case 0xc2:
        return false;
    case 0xc3:
        return true;
    case 0xc4:
        return take(static_cast<qsizetype>(be(1)));
    case 0xc5:
        return take(static_cast<qsizetype>(be(2)));
    case 0xc6:
        return take(static_cast<qsizetype>(be(4)));
    case 0xca: {
        const auto bits = static_cast<quint32>(be(4));
        float f = 0;
        std::memcpy(&f, &bits, sizeof f);
        return static_cast<double>(f);
    }
    case 0xcb: {
        const quint64 bits = be(8);
        double d = 0;
        std::memcpy(&d, &bits, sizeof d);
        return d;
    }
    case 0xcc:
        return static_cast<qint64>(be(1));
    case 0xcd:
        return static_cast<qint64>(be(2));
    case 0xce:
        return static_cast<qint64>(be(4));
    case 0xcf:
        return static_cast<qint64>(be(8));
    case 0xd0:
        return static_cast<qint64>(static_cast<qint8>(be(1)));
    case 0xd1:
        return static_cast<qint64>(static_cast<qint16>(be(2)));
    case 0xd2:
        return static_cast<qint64>(static_cast<qint32>(be(4)));
    case 0xd3:
        return static_cast<qint64>(be(8));
    case 0xd4:
        return ext(1);
    case 0xd5:
        return ext(2);
    case 0xd6:
        return ext(4);
    case 0xd7:
        return ext(8);
    case 0xd8:
        return ext(16);
    case 0xc7:
        return ext(static_cast<qsizetype>(be(1)));
    case 0xc8:
        return ext(static_cast<qsizetype>(be(2)));
    case 0xc9:
        return ext(static_cast<qsizetype>(be(4)));
    case 0xd9:
        return QString::fromUtf8(take(static_cast<qsizetype>(be(1))));
    case 0xda:
        return QString::fromUtf8(take(static_cast<qsizetype>(be(2))));
    case 0xdb:
        return QString::fromUtf8(take(static_cast<qsizetype>(be(4))));
    case 0xdc:
        return array(be(2));
    case 0xdd:
        return array(be(4));
    case 0xde:
        return map(be(2));
    case 0xdf:
        return map(be(4));
    default:
        return {};
    }
}

} // namespace

void encode(QByteArray& out, const QVariant& value)
{
    if (value.metaType().id() == qMetaTypeId<NvimExt>()) {
        const auto e = value.value<NvimExt>();
        QByteArray payload;
        encodeInt(payload, e.handle);
        out.append(static_cast<char>(0xc7));
        putBE(out, static_cast<quint64>(payload.size()), 1);
        out.append(static_cast<char>(e.type));
        out.append(payload);
        return;
    }
    switch (value.typeId()) {
    case QMetaType::UnknownType:
    case QMetaType::Nullptr:
        out.append(static_cast<char>(0xc0));
        return;
    case QMetaType::Bool:
        out.append(static_cast<char>(value.toBool() ? 0xc3 : 0xc2));
        return;
    case QMetaType::Int:
    case QMetaType::LongLong:
    case QMetaType::UInt:
    case QMetaType::ULongLong:
        encodeInt(out, value.toLongLong());
        return;
    case QMetaType::Double: {
        out.append(static_cast<char>(0xcb));
        const double d = value.toDouble();
        quint64 bits = 0;
        std::memcpy(&bits, &d, sizeof bits);
        putBE(out, bits, 8);
        return;
    }
    case QMetaType::QString:
        encodeBytes(out, value.toString().toUtf8(), true);
        return;
    case QMetaType::QByteArray:
        encodeBytes(out, value.toByteArray(), false);
        return;
    case QMetaType::QVariantList:
    case QMetaType::QStringList: {
        const QVariantList list = value.toList();
        encodeHeader(out, static_cast<quint64>(list.size()), 0x90, 15, 0xdc, 0xdd);
        for (const QVariant& v : list) {
            encode(out, v);
        }
        return;
    }
    case QMetaType::QVariantMap: {
        const QVariantMap m = value.toMap();
        encodeHeader(out, static_cast<quint64>(m.size()), 0x80, 15, 0xde, 0xdf);
        for (auto it = m.cbegin(); it != m.cend(); ++it) {
            encodeBytes(out, it.key().toUtf8(), true);
            encode(out, it.value());
        }
        return;
    }
    default:
        encodeBytes(out, value.toString().toUtf8(), true);
        return;
    }
}

std::optional<QVariant> decode(const QByteArray& data, qsizetype& pos)
{
    Reader r{data, pos};
    QVariant v = r.value();
    if (r.short_) {
        return std::nullopt;
    }
    pos = r.pos;
    return v;
}

} // namespace prova::msgpack
