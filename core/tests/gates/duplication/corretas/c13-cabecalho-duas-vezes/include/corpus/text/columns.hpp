// Corpus do gate de duplicação (P20), forma correta c13: função inline num
// cabeçalho incluído por duas unidades novas.
#ifndef CORPUS_TEXT_COLUMNS_HPP
#define CORPUS_TEXT_COLUMNS_HPP

#include <cstddef>
#include <string_view>

namespace corpus::text {

[[nodiscard]] inline std::size_t display_columns(std::string_view text, std::size_t tab) {
    std::size_t columns{0};
    for (const char character : text) {
        if (character == '\t') {
            columns += tab - columns % tab;
        } else if ((static_cast<unsigned char>(character) & 0xC0U) != 0x80U) {
            ++columns;
        }
    }
    return columns;
}

}  // namespace corpus::text

#endif
