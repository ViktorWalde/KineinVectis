// Corpus do gate de duplicação (P20), forma correta c13: primeira unidade.
#include "corpus/text/columns.hpp"

#include <cstddef>
#include <string>
#include <string_view>

namespace corpus::text {

std::string ruler(std::string_view text) {
    return std::string(display_columns(text, 4), '-');
}

}  // namespace corpus::text
