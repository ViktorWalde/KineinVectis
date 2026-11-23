// Corpus do gate de duplicação (P20), forma correta c13: segunda unidade.
#include "corpus/text/columns.hpp"

#include <cstddef>
#include <string_view>

namespace corpus::text {

bool fits(std::string_view text, std::size_t width) {
    return display_columns(text, 8) <= width;
}

}  // namespace corpus::text
