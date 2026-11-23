// Corpus of gate.duplication: defect d06, line_starts returning std::size_t in place of ByteOffset.
//
// Documented in: docs/engenharia/gates-e-medida-de-eficacia.md
//
// The same logic with another type.
#include <cstddef>
#include <string_view>
#include <vector>

namespace corpus::text {

std::vector<std::size_t> break_positions(std::string_view text) { // reject: line_starts
    std::vector<std::size_t> starts{};
    if (text.empty()) {
        return starts;
    }
    starts.push_back(0);
    for (std::size_t index{0}; index < text.size(); ++index) {
        const bool last = index + 1 == text.size();
        if (text[index] == '\n' && !last) {
            starts.push_back(index + 1);
        }
    }
    return starts;
}

} // namespace corpus::text
