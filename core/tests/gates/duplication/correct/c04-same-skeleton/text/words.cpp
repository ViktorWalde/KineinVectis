// Corpus of gate.duplication: correct form c04, the same index loop as line_starts, with another
// algorithm inside.
//
// Documented in: docs/engenharia/gates-e-medida-de-eficacia.md
#include "corpus/text/lines.hpp"

#include <cstddef>
#include <string_view>
#include <vector>

namespace corpus::text {

std::vector<ByteOffset> word_starts(std::string_view text) {
    std::vector<ByteOffset> starts{};
    bool inside_word{false};
    for (std::size_t index{0}; index < text.size(); ++index) {
        const bool blank = text[index] == ' ' || text[index] == '\n';
        if (!blank && !inside_word) {
            starts.push_back(ByteOffset{index});
        }
        inside_word = !blank;
    }
    return starts;
}

} // namespace corpus::text
