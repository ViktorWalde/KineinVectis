// Corpus of gate.duplication: correct form c13, first unit.
//
// Documented in: docs/engenharia/gates-e-medida-de-eficacia.md
#include "corpus/text/columns.hpp"

#include <cstddef>
#include <string>
#include <string_view>

namespace corpus::text {

std::string ruler(std::string_view text) {
    return std::string(display_columns(text, 4), '-');
}

} // namespace corpus::text
