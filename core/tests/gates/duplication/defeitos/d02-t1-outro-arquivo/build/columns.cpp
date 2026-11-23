// Corpus do gate de duplicação (P20), defeito d02: T1 em outro arquivo do
// mesmo módulo, escondido num namespace anônimo para "não mexer no cabeçalho".
#include "corpus/build/diagnostics.hpp"

#include "corpus/text/lines.hpp"

#include <array>
#include <cstddef>
#include <optional>
#include <string_view>

namespace corpus::build {
namespace {

std::optional<Fields> split_columns(std::string_view text) {  // reprova: split_fields
    std::array<std::string_view, 4> heads{};
    std::size_t start{0};
    for (auto& head : heads) {
        const auto colon = text.find(':', start);
        if (colon == std::string_view::npos) {
            return std::nullopt;
        }
        head = text.substr(start, colon - start);
        start = colon + 1;
    }
    return Fields{
        .file = heads[0],
        .line = heads[1],
        .column = heads[2],
        .severity = text::trim(heads[3]),
        .message = text::trim(text.substr(start)),
    };
}

}  // namespace

std::string_view file_of(std::string_view text) {
    const auto fields = split_columns(text);
    return fields ? fields->file : std::string_view{};
}

}  // namespace corpus::build
