// Corpus of gate.duplication: correct form c07, overloads that delegate to a single implementation.
//
// Documented in: docs/engenharia/gates-e-medida-de-eficacia.md
#include "corpus/text/lines.hpp"

#include <cstddef>
#include <string>
#include <string_view>
#include <vector>

namespace corpus::text {

std::size_t count_lines(const std::string& text) {
    return count_lines(std::string_view{text});
}

std::size_t count_lines(const char* text) {
    return count_lines(std::string_view{text});
}

std::vector<ByteOffset> line_starts(const std::string& text) {
    return line_starts(std::string_view{text});
}

std::vector<ByteOffset> line_starts(const char* text) {
    return line_starts(std::string_view{text});
}

} // namespace corpus::text
