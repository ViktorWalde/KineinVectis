// Corpus do gate de duplicação (P20), forma correta c07: sobrecargas que
// delegam a uma implementação só.
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

}  // namespace corpus::text
