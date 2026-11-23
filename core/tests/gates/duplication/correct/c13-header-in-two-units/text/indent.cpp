// Corpus of gate.duplication: correct form c13, second unit.
//
// Documented in: docs/engenharia/gates-e-medida-de-eficacia.md
#include "corpus/text/columns.hpp"

#include <cstddef>
#include <string_view>

namespace corpus::text {

bool fits(std::string_view text, std::size_t width) {
    return display_columns(text, 8) <= width;
}

} // namespace corpus::text
