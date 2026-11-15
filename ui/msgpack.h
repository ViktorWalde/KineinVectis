// Codec msgpack mínimo para o msgpack-RPC do Neovim, sobre QVariant.
//
// Só o que o protocolo do Neovim usa: nil, bool, inteiros, float, str, bin,
// array, map e ext (Buffer, Window e Tabpage chegam como ext com um inteiro
// dentro). str vira QString; bin vira QByteArray; ext vira prova::NvimExt.
#pragma once

#include <QByteArray>
#include <QMetaType>
#include <QVariant>

#include <optional>

namespace prova {

struct NvimExt {
    qint8 type = 0;
    qint64 handle = 0;
};

namespace msgpack {

void encode(QByteArray& out, const QVariant& value);

// Decodifica um valor a partir de `pos`. Devolve nullopt se os bytes ainda
// não chegaram inteiros; nesse caso `pos` não anda.
std::optional<QVariant> decode(const QByteArray& data, qsizetype& pos);

} // namespace msgpack
} // namespace prova

Q_DECLARE_METATYPE(prova::NvimExt)
