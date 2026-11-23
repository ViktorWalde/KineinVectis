// Corpus do gate de duplicação (P20): o módulo de texto da árvore base.
// (docs/engenharia/gates-e-medida-de-eficacia.md)
#ifndef CORPUS_TEXT_LINES_HPP
#define CORPUS_TEXT_LINES_HPP

#include <cstddef>
#include <string>
#include <string_view>
#include <vector>

namespace corpus::text {

enum class ByteOffset : std::size_t {};

[[nodiscard]] std::size_t count_lines(std::string_view text);
[[nodiscard]] std::vector<ByteOffset> line_starts(std::string_view text);
[[nodiscard]] std::string_view trim(std::string_view text);
[[nodiscard]] std::vector<std::string> wrap_words(std::string_view text, std::size_t width);

// Definição em cabeçalho incluído por duas unidades: a ferramenta vê a mesma
// função duas vezes, e isso não é cópia.
[[nodiscard]] inline bool is_blank(std::string_view text) {
    for (const char character : text) {
        const bool space = character == ' ' || character == '\t' || character == '\n'
                           || character == '\r';
        if (!space) {
            return false;
        }
    }
    return true;
}

}  // namespace corpus::text

#endif
